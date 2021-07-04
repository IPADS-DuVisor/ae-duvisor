use crate::vm::virtualmachine;
use crate::irq::virq;
use crate::vcpu::vcpucontext;
use std::sync::{Arc, Mutex};
use vcpucontext::*;
use crate::mm::utils::*;
use crate::mm::gstagemmu::*;
use crate::plat::uhe::ioctl::ioctl_constants::*;
use crate::irq::delegation::delegation_constants::*;
use crate::plat::uhe::csr::csr_constants;
use csr_constants::*;
use crate::plat::opensbi;
use crate::vcpu::utils::*;
use crate::irq::plic::Plic;
use std::lazy::SyncOnceCell;
use crate::devices::tty::Tty;

#[allow(unused)]
mod errno_constants {
    pub const EFAILED: i32 = -1;
    pub const ENOPERMIT: i32 = -2;
    pub const ENOMAPPING: i32 = -3;
}
pub use errno_constants::*;

pub const ECALL_VM_TEST_END: u64 = 0xFF;

pub enum ExitReason {
    ExitUnknown,
    ExitEaccess,
    ExitMmio,
    ExitIntr,
    ExitSystemEvent,
    ExitRiscvSbi,
}

#[allow(unused)]
#[link(name = "enter_guest")]
extern "C" {
    fn enter_guest(vcpuctx: u64) -> i32;
    fn exit_guest();
}

#[allow(unused)]
extern "C"
{
    fn hypervisor_load(target_addr: u64) -> u64;
}

#[allow(unused)]
extern "C"
{
    fn vcpu_ecall_exit();
    fn vcpu_ecall_exit_end();
    fn vcpu_add_all_gprs();
    fn vcpu_add_all_gprs_end();
    fn vmem_ld_mapping();
    fn vmem_ld_mapping_end();
    fn vmem_W_Ro();
    fn vmem_W_Ro_end();
    fn vmem_X_nonX();
    fn vmem_X_nonX_end();
    fn vmem_ld_sd_over_loop();
    fn vmem_ld_sd_over_loop_end();
}

pub struct VirtualCpu {
    pub vcpu_id: u32,
    pub vm: Arc<Mutex<virtualmachine::VmSharedState>>,
    pub vcpu_ctx: Mutex<VcpuCtx>,
    pub virq: Mutex<virq::VirtualInterrupt>,
    // Cell for late init
    // TODO: replace plic with irq_chip abstraction
    pub plic: SyncOnceCell<Arc<Plic>>,
    /* TODO: irq_pending with shared memory */
    pub exit_reason: Mutex<ExitReason>,
    pub console: Arc<Mutex<Tty>>,
}

impl VirtualCpu {
    pub fn new(vcpu_id: u32,
            vm_mutex_ptr: Arc<Mutex<virtualmachine::VmSharedState>>,
            console: Arc<Mutex<Tty>>) -> Self {
        let vcpu_ctx = Mutex::new(VcpuCtx::new());
        let virq = Mutex::new(virq::VirtualInterrupt::new());
        let exit_reason = Mutex::new(ExitReason::ExitUnknown);
        let plic = SyncOnceCell::new();

        Self {
            vcpu_id,
            vm: vm_mutex_ptr,
            vcpu_ctx,
            virq,
            plic,
            exit_reason,
            console,
        }
    }

    /* For test case: test_vm_run */
    pub fn test_change_guest_ctx(&self) -> u32 {
        /* Change guest context */
        self.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10] += 10;
        self.vcpu_ctx.lock().unwrap().guest_ctx.sys_regs.huvsscratch += 11;
        self.vcpu_ctx.lock().unwrap().guest_ctx.hyp_regs.hutinst += 12;

        /* Increse vm_id in vm_state */
        self.vm.lock().unwrap().vm_id += 100;

        0
    }

    fn config_hugatp(&self) -> u64 {
        let pt_pfn: u64 = 
            self.vm.lock().unwrap().gsmmu.page_table.paddr >> PAGE_SIZE_SHIFT;
        let hugatp: u64 = pt_pfn | HUGATP_MODE_SV48;

        self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hugatp = hugatp;

        unsafe { csrw!(HUGATP, hugatp); }

        dbgprintln!("set hugatp {:x}", hugatp);

        hugatp
    }
    
    fn handle_virtual_inst_fault(&self) -> i32 {
        let ret = 0;

        self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc += 4;
        
        ret
    }

    fn handle_u_vtimer_irq(&self) -> i32 {
        /* insert or clear tty irq on each vtimer irq */
        let cnt = self.console.lock().unwrap().cnt;

        if cnt > 0 {
            unsafe {
                csrs!(HUVIP, 1 << IRQ_TTY);
            }
        } else {
            unsafe {
                csrc!(HUVIP, 1 << IRQ_TTY);
            }
        }

        /* set virtual timer */
        self.virq.lock().unwrap().set_pending_irq(IRQ_VS_TIMER);
        unsafe {
            /* 
             * FIXME: There may be unexpected pending bit IRQ_U_VTIMER when 
             * traped to kernel disable timer.
             */
            csrc!(VTIMECTL, 1 << VTIMECTL_ENABLE);

            /* Clear U VTIMER bit. Its counterpart in ARM is GIC EOI.  */
            csrc!(HUIP, 1 << IRQ_U_VTIMER);
        }
        return 0;
    }

    /* TODO: H(U)LV/H(U)LVX.HU problems on qemu */
    fn get_vm_inst_by_uepc(_uepc: u64) -> u64 {
        return 0;
    }

    /* TODO: Cannot get the instruction for now */
    fn inst_parse(_inst: u64) -> Option<(u64, u64)> {
        return None;
    }

    fn store_emulation(&self, fault_addr: u64, target_reg: u64,
                bit_width: u64) -> i32 {
        let ret: i32;
        let bit_mask: u64 = (1 << bit_width) - 1;
        let data: u64 = self.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs
                .x_reg[target_reg as usize] & bit_mask;

        if fault_addr >= 0x3f8 && fault_addr < 0x400 { /* ttyS0-3F8 */
            ret = Tty::store_emulation(&self, fault_addr, data as u8);
        } else {
            dbgprintln!("Unknown mmio (store)");
            ret = 1;
        }

        return ret;
    }

    fn load_emulation(&self, fault_addr: u64, target_reg: u64,
                _bit_width: u64) -> i32 {
        let ret: i32;

        if fault_addr >= 0x3f8 && fault_addr < 0x400 { /* ttyS0-3F8 */
            let data: u64 = Tty::load_emulation(&self, fault_addr) as u64;
            self.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[target_reg as usize] = data;
            ret = 0;
        } else {
            dbgprintln!("Unknown mmio (load) fault_addr: 0x{:x}", fault_addr);
            ret = 1;
        }

        return ret;
    }

    /* 
     * Handlers for mmio require the follow info at least:
     * - fault address: the fault address
     * - instruction: the instruction which caused the trap
     *   - data bit width: for example, SD/LD or SW/LW
     *   - target register: the register which the data should be stored or 
     *     loaded
     * - data access type: load or store (get from ucause or inst)
     *
     * TODO: the HLV instructions got some problems on qemu for now.
     * Take the load inst as 'lb a0, 0x0(a0)' 
     * and the store inst as 'sb a2, 0x0(a1)'
     */
    fn handle_mmio(&self, fault_addr: u64) -> i32 {
        let ucause = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.ucause;
        let uepc = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc;
        let hutinst = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hutinst;
        let inst: u64;
        let target_reg: u64;
        let bit_width: u64;
        let mut inst_width: u64 = 4;
        let ret: i32;

        // ffffffe0002e568a sw a3
        // ffffffe0002e53ba sw a2
        // ffffffe0002e57d6 sw a4
        // ffffffe0002e55c0 sw a4
        // ffffffe000713502 sw s1
        // ffffffe000713514 lw a5
        // ffffffe000713522 sw s1
        // ffffffe0007134f8 lw a3
        // ffffffe0002e543e sw a5
        let is_plic_mmio = |fault_addr: u64| -> bool {
            if 0xc000000 <= fault_addr && fault_addr < (0xc000000 + 0x1000000) {
                return true;
            } else {
                return false;
            }
        };
        
        if is_plic_mmio(fault_addr) {
            println!("handle_mmio: plic_toggle uepc {:x} ucause {:x}, hutinst {:x}", 
                uepc, ucause, hutinst);
        }

        if hutinst == 0x0 {
            /* The implementation has not support the function of hutinst */
            inst = VirtualCpu::get_vm_inst_by_uepc(uepc);
        } else {
            inst = hutinst;
        }

        let inst_res = VirtualCpu::inst_parse(inst);
        if inst_res.is_none() {
            /* linux use a0 for load and a2 for store in ttyS0-3f8 */
            if ucause == EXC_LOAD_GUEST_PAGE_FAULT {
                /* lb a0, 0x0(a0) */
                target_reg = 10;
                bit_width = 8;
            } else {
                /* sb a2, 0x0(a1) */
                target_reg = 12;
                bit_width = 8;
            }
        } else {
            let (_target_reg, _bit_width) = inst_res.unwrap();
            target_reg = _target_reg;
            bit_width = _bit_width;
        }

        if ucause == EXC_LOAD_GUEST_PAGE_FAULT {
            /* load */
            if is_plic_mmio(fault_addr) {
                let mut data: u32 = 0;
                self.plic.get().unwrap().mmio_callback(fault_addr, &mut data, false);
                if (uepc == 0xffffffe0007134f8) {
                    self.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[13] = data as u64;
                } else {
                    self.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[15] = data as u64;
                }
                ret = 0;
                inst_width = 2;
            } else {
            /* check the input and update huvip */
            self.console.lock().unwrap().update_huvip();

            ret = self.load_emulation(fault_addr, target_reg, bit_width);
            }
        } else if ucause == EXC_STORE_GUEST_PAGE_FAULT {
            /* store */
            if is_plic_mmio(fault_addr) {
                let mut data: u32 = 0;
                if uepc == 0xffffffe0002e543e {
                    data = self.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[15] as u32;
                } else {
                    data = self.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[9] as u32;
                }
                self.plic.get().unwrap().mmio_callback(fault_addr, &mut data, true);
                ret = 0;
                inst_width = 2;
            } else {
            ret = self.store_emulation(fault_addr, target_reg, bit_width);
            }
        } else {
            ret = 1;
        }

        self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc = uepc + inst_width;

        return ret;
    }

    fn handle_stage2_page_fault(&self) -> i32 {
        let hutval = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hutval;
        let utval = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.utval;
        let mut fault_addr = (hutval << 2) | (utval & 0x3);
        let mut ret;

        //if self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc == 0xffffffe0002e543e ||
        //    (fault_addr >= 0xc000000 && fault_addr < 0xc000000 + 0x4000000) {
        //    println!("gstage fault: hutval: {:x}, utval: {:x}, fault_addr: {:x}",
        //        hutval, utval, fault_addr);
        //}

        dbgprintln!("gstage fault: hutval: {:x}, utval: {:x}, fault_addr: {:x}",
            hutval, utval, fault_addr);
        
        let gpa_check = self.vm.lock().unwrap().gsmmu.check_gpa(fault_addr);
        if !gpa_check {
            /* Maybe mmio or illegal gpa */
            let mmio_check = self.vm.lock().unwrap().gsmmu.check_mmio(fault_addr);

            //if !mmio_check {
            //    panic!("Invalid gpa!");
            //}

            ret = self.handle_mmio(fault_addr);

            return ret;
        }

        fault_addr &= !PAGE_SIZE_MASK;

        /* map_query */
        let query = self.vm.lock().unwrap().gsmmu.map_query(fault_addr);
        if query.is_some() {
            let i = query.unwrap();

            dbgprintln!("Query PTE offset {}, value {}, level {}", i.offset, 
                i.value, i.level);

            if i.is_leaf() {
                ret = ENOPERMIT;
            } else {
                dbgprintln!("QUERY is some but ENOMAPPING");

                ret = ENOMAPPING;
            }
        } else {
            ret = ENOMAPPING;
        }
        match ret {
            ENOPERMIT => {
                *self.exit_reason.lock().unwrap() = ExitReason::ExitEaccess;
                dbgprintln!("Query return ENOPERMIT: {}", ret);
            }
            ENOMAPPING => {
                dbgprintln!("Query return ENOMAPPING: {}", ret);
                /* find hpa by fault_addr */
                let fault_hpa_query = self.vm.lock().unwrap().gsmmu
                    .gpa_block_query(fault_addr);

                if fault_hpa_query.is_some() {
                    /* fault gpa is already in a gpa_block and it is valid */
                    let fault_hpa = fault_hpa_query.unwrap();
                    let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE
                        | PTE_EXECUTE;
                        
                    dbgprintln!("map gpa: {:x} to hpa: {:x}",
                        fault_addr, fault_hpa);
                    self.vm.lock().unwrap().gsmmu.map_page(
                        fault_addr, fault_hpa, flag);

                    ret = 0;
                } else {
                    /* fault gpa is not in a gpa_block and it is valid */
                    let len = PAGE_SIZE;
                    let res = self.vm.lock().unwrap().gsmmu
                        .gpa_block_add(fault_addr, len);

                    if res.is_ok() {
                        /* map new page to VM if the region exists */
                        let (_hva, hpa) = res.unwrap();
                        let flag: u64 = PTE_USER | PTE_VALID | PTE_READ 
                            | PTE_WRITE | PTE_EXECUTE;

                        self.vm.lock().unwrap().gsmmu.map_page(
                            fault_addr, hpa, flag);

                        ret = 0;
                    } else {
                        panic!("Create gpa_block for fault addr {:x} failed!",
                            fault_addr);
                    }
                }
            }
            _ => {
                *self.exit_reason.lock().unwrap() = ExitReason::ExitEaccess;
                dbgprintln!("Invalid query result: {}", ret);
            }
        }

        ret
    }

    fn handle_supervisor_ecall(&self) -> i32 {
        let ret: i32;
        let mut vcpu_ctx = self.vcpu_ctx.lock().unwrap();
        let a0 = vcpu_ctx.guest_ctx.gp_regs.x_reg[10]; // a0: 0th arg/ret 1
        let a1 = vcpu_ctx.guest_ctx.gp_regs.x_reg[11]; // a1: 1st arg/ret 2
        let a2 = vcpu_ctx.guest_ctx.gp_regs.x_reg[11]; // a1: 2nd arg 
        let a3 = vcpu_ctx.guest_ctx.gp_regs.x_reg[11]; // a1: 3rd arg 
        let a4 = vcpu_ctx.guest_ctx.gp_regs.x_reg[11]; // a1: 4th arg 
        let a5 = vcpu_ctx.guest_ctx.gp_regs.x_reg[11]; // a1: 5th arg 
        let a6 = vcpu_ctx.guest_ctx.gp_regs.x_reg[16]; // a6: FID
        let a7 = vcpu_ctx.guest_ctx.gp_regs.x_reg[17]; // a7: EID

        /* FIXME: for test cases */
        if a7 == ECALL_VM_TEST_END {
            ret = 0xdead;

            vcpu_ctx.host_ctx.gp_regs.x_reg[0] = ret as u64;
        
            return ret as i32;
        }
        
        let mut target_ecall = opensbi::emulation::Ecall::new();
        target_ecall.ext_id = a7;
        target_ecall.func_id = a6;
        target_ecall.arg[0] = a0;
        target_ecall.arg[1] = a1;
        target_ecall.arg[2] = a2;
        target_ecall.arg[3] = a3;
        target_ecall.arg[4] = a4;
        target_ecall.arg[5] = a5;
        target_ecall.ret[0] = a0;
        target_ecall.ret[1] = a1;

        /* Part of SBIs should emulated via IOCTL */
        let fd = self.vm.lock().unwrap().gsmmu.allocator.ioctl_fd as i32;
        ret = target_ecall.ecall_handler(fd, &self);

        /* save the result */
        vcpu_ctx.guest_ctx.gp_regs.x_reg[10] = target_ecall.ret[0];
        vcpu_ctx.guest_ctx.gp_regs.x_reg[11] = target_ecall.ret[1];

        /* add uepc to start vm on next instruction */
        vcpu_ctx.host_ctx.hyp_regs.uepc += 4;

        ret
    }

    fn handle_vcpu_exit(&self) -> i32 {
        let mut ret: i32 = -1;
        let ucause = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.ucause;
        *self.exit_reason.lock().unwrap() = ExitReason::ExitUnknown;
        
        if self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc == 0xffffffe0002e543e {
            println!("vmexit: ucause: {:x}", ucause);
        }

        if (ucause & EXC_IRQ_MASK) != 0 {
            *self.exit_reason.lock().unwrap() = ExitReason::ExitIntr;
            let ucause = ucause & (!EXC_IRQ_MASK);
            match ucause {
                IRQ_U_VTIMER => {
                    dbgprintln!("handler U VTIMER: {}, current pc is {:x}.", ucause, self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc);
                    ret = self.handle_u_vtimer_irq();
                }
                _ => {
                    dbgprintln!("Invalid IRQ ucause: {}", ucause);
                    ret = 1;
                }
            }
            return ret;
        }

        match ucause {
            EXC_VIRTUAL_INST_FAULT => {
                self.handle_virtual_inst_fault();
                ret = 0;
            }
            EXC_INST_GUEST_PAGE_FAULT | EXC_LOAD_GUEST_PAGE_FAULT |
                EXC_STORE_GUEST_PAGE_FAULT => {
                ret = self.handle_stage2_page_fault();
            }
            EXC_VIRTUAL_SUPERVISOR_SYSCALL => {
                ret = self.handle_supervisor_ecall();
            }
            _ => {
                dbgprintln!("Invalid EXCP ucause: {}", ucause);
            }
        }

        if ret < 0 {
            dbgprintln!("ERROR: handle_vcpu_exit ret: {}", ret);

            /* FIXME: save the exit reason in HOST_A0 before the vcpu down */
            self.vcpu_ctx.lock().unwrap().host_ctx.gp_regs.x_reg[0] = (0 - ret) as u64;
        }

        ret
    }

    pub fn thread_vcpu_run(&self) -> i32 {
        let fd = self.vm.lock().unwrap().gsmmu.allocator.ioctl_fd;
        let mut _res;

        self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hustatus = ((1 << HUSTATUS_SPV_SHIFT)
            | (1 << HUSTATUS_SPVP_SHIFT)) | (1 << HUSTATUS_UPIE_SHIFT) as u64;

        unsafe {
            /* register vcpu thread to the kernel */
            _res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
            dbgprintln!("IOCTL_LAPUTA_REGISTER_VCPU : {}", _res);

            /* set hugatp */
            let _hugatp = self.config_hugatp();
            dbgprintln!("Config hugatp: {:x}", _hugatp);

            /* set trap handler */
            csrw!(UTVEC, exit_guest as u64);

            /* enable timer irq */
            csrw!(HUIE, 1 << IRQ_U_VTIMER);

            /* TODO: redesign scounteren register */
            /* allow VM to directly access time register */

            /* TODO: introduce RUST feature to distinguish between rv64 and rv32 */
            let delta_time :i64 = csrr!(TIME) as i64;
            csrw!(HUTIMEDELTA, -delta_time as u64);
        }
        // FIXME: deadlock if ptr & ptr_u64 are not declared independently
        let vcpu_ctx_ptr: *const VcpuCtx;
        let vcpu_ctx_ptr_u64: u64;
        vcpu_ctx_ptr = &*self.vcpu_ctx.lock().unwrap() as *const VcpuCtx;
        vcpu_ctx_ptr_u64 = vcpu_ctx_ptr as u64;
        
        let mut ret: i32 = 0;
        while ret == 0 {
            // Flush pending irqs into HUVIP
            self.virq.lock().unwrap().flush_pending_irq();

            unsafe {
                enter_guest(vcpu_ctx_ptr_u64);
            }

            ret = self.handle_vcpu_exit();
        }
        
        unsafe {
            _res = libc::ioctl(fd, IOCTL_LAPUTA_UNREGISTER_VCPU);
            dbgprintln!("IOCTL_LAPUTA_UNREGISTER_VCPU : {}", _res);
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use rusty_fork::rusty_fork_test;
    use crate::debug::utils::configtest::test_vm_config_create;
    use crate::devices::tty::tty_uart_constants::*;

    rusty_fork_test! {
        #[test]
        fn test_handle_stage2_page_fault() { 
            let vcpu_id = 0;
            let vm_config = test_vm_config_create();
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let fd = vm.vm_state.lock().unwrap().ioctl_fd;
            let vm_mutex = vm.vm_state;
            let console = Arc::new(Mutex::new(Tty::new()));
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console);
            let mut res;
            let version: u64 = 0;
            let test_buf: u64;
            let test_buf_pfn: u64;
            let test_buf_size: usize = 32 << 20;
            let mut hugatp: u64;

            unsafe { 
                let version_ptr = (&version) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_GET_API_VERSION, version_ptr);
                println!("IOCTL_LAPUTA_GET_API_VERSION -  version : {:x}", 
                    version);

                let addr = 0 as *mut libc::c_void;
                let mmap_ptr = libc::mmap(addr, test_buf_size, 
                    libc::PROT_READ | libc::PROT_WRITE, 
                    libc::MAP_SHARED, fd, 0);
                assert_ne!(mmap_ptr, libc::MAP_FAILED);

                test_buf = mmap_ptr as u64;
                test_buf_pfn = test_buf;
                let test_buf_pfn_ptr = (&test_buf_pfn) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_QUERY_PFN, test_buf_pfn_ptr);
                println!("IOCTL_LAPUTA_QUERY_PFN -  test_buf_pfn : {:x}", 
                    test_buf_pfn);

                vcpu.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[17] = ECALL_VM_TEST_END;

                let mut test_buf_ptr = test_buf as *mut i32;
                *test_buf_ptr = 0x73; /* ecall */
                test_buf_ptr = (test_buf + 4) as *mut i32;
                *test_buf_ptr = 0xa001; /* loop */

                /* 512G 1-level direct mapping */
                hugatp = test_buf + PAGE_SIZE * 4;
                let pte_ptr = (hugatp + 8 * ((test_buf_pfn << PAGE_SIZE_SHIFT)
                     >> 30)) as *mut u64;
                *pte_ptr = (((test_buf_pfn << PAGE_SIZE_SHIFT) >> 30) << 28) | 
                    0x1f;
                println!("PTE : {:x}", *pte_ptr);

                /* delegate vs-ecall and guest page fault */
                virtualmachine::VirtualMachine::hu_delegation(fd);

                res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
                println!("IOCTL_LAPUTA_REGISTER_VCPU : {}", res);
            }

            let mut uepc: u64 = 0;
            let mut utval: u64 = 0;
            let mut ucause: u64 = 0;

            // FIXME: deadlock if ptr & ptr_u64 are not declared independently
            let ptr: *const VcpuCtx;
            let ptr_u64: u64;
            ptr = &*vcpu.vcpu_ctx.lock().unwrap() as *const VcpuCtx;
            ptr_u64 = ptr as u64;
            println!("test_handle_stage2_page_fault - ptr_u64: {:x}", ptr_u64);
            let mut ret: i32 = 0;

            let target_code = (test_buf_pfn << PAGE_SIZE_SHIFT) as u64;
            vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc = target_code;

            hugatp = (test_buf_pfn + 2) | (8 << 60);
            vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hugatp = hugatp;

            while ret == 0 {
                unsafe {
                    csrw!(HUGATP, vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hugatp);
                    println!("HUGATP : {:x}", 
                        vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hugatp);

                    /* hustatus.SPP=1 .SPVP=1 uret to VS mode */
                    vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hustatus = 
                        ((1 << HUSTATUS_SPV_SHIFT) 
                        | (1 << HUSTATUS_SPVP_SHIFT)) as u64;

                    csrw!(UTVEC, exit_guest as u64);
                    enter_guest(ptr_u64);

                    uepc = vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc;
                    utval = vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.utval;
                    ucause = vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.ucause;

                    println!("guest hyp uepc 0x{:x}", uepc);
                    println!("guest hyp utval 0x{:x}", utval);
                    println!("guest hyp ucause 0x{:x}", ucause);

                    if ucause == 20 {
                        vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hugatp = 
                            (test_buf_pfn + 4) | HUGATP_MODE_SV39;
                    }
                }
                ret = vcpu.handle_vcpu_exit();
            }

            unsafe {
                res = libc::ioctl(fd, IOCTL_LAPUTA_UNREGISTER_VCPU);
                println!("IOCTL_LAPUTA_UNREGISTER_VCPU : {}", res);

                let addr = test_buf as *mut libc::c_void;
                libc::munmap(addr, test_buf_size); 

                let test_buf_pfn_ptr = (&test_buf_pfn) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_RELEASE_PFN, test_buf_pfn_ptr);
                println!("IOCTL_LAPUTA_RELEASE_PFN - test_buf_pfn : {:x}", 
                    test_buf_pfn);
            }

            assert_eq!(uepc, test_buf_pfn << PAGE_SIZE_SHIFT);
            assert_eq!(utval, 0);
            assert_eq!(ucause, 10);
        }

        /* Check the correctness of vcpu new() */
        #[test]
        fn test_vcpu_new() { 
            let vcpu_id = 20;
            let vm_config = test_vm_config_create();
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let vm_mutex = vm.vm_state;
            let console = Arc::new(Mutex::new(Tty::new()));
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console);

            assert_eq!(vcpu.vcpu_id, vcpu_id);
        }

        /* Check the init state of the vcpu */  
        #[test]
        fn test_vcpu_ctx_init() { 
            let vcpu_id = 1;
            let vm_config = test_vm_config_create();
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let vm_mutex = vm.vm_state;
            let console = Arc::new(Mutex::new(Tty::new()));
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console);

            let tmp = vcpu.vcpu_ctx.lock().unwrap().host_ctx.gp_regs.x_reg[10];
            assert_eq!(tmp, 0);

            let tmp = vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hutinst;
            assert_eq!(tmp, 0);

            let tmp = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10];
            assert_eq!(tmp, 0);

            let tmp = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.hyp_regs.hutinst;
            assert_eq!(tmp, 0);

            let tmp = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.sys_regs.huvsatp;
            assert_eq!(tmp, 0);
        }

        /* Check the rw permission of vcpu ctx */
        #[test]
        fn test_vcpu_set_ctx() {  
            let vcpu_id = 1;
            let vm_config = test_vm_config_create();
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let vm_mutex = vm.vm_state;
            let console = Arc::new(Mutex::new(Tty::new()));
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console);
            let ans = 17;

            /* guest ctx */
            vcpu.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10] = ans;
            let tmp = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10];
            assert_eq!(tmp, ans);

            vcpu.vcpu_ctx.lock().unwrap().guest_ctx.sys_regs.huvsatp = ans;
            let tmp = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.sys_regs.huvsatp;
            assert_eq!(tmp, ans);

            vcpu.vcpu_ctx.lock().unwrap().guest_ctx.hyp_regs.hutinst = ans;
            let tmp = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.hyp_regs.hutinst;
            assert_eq!(tmp, ans);

            /* host ctx */
            vcpu.vcpu_ctx.lock().unwrap().host_ctx.gp_regs.x_reg[10] = ans;
            let tmp = vcpu.vcpu_ctx.lock().unwrap().host_ctx.gp_regs.x_reg[10];
            assert_eq!(tmp, ans);

            vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hutinst = ans;
            let tmp = vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hutinst;
            assert_eq!(tmp, ans);
        }

        /* Check the Arc<Mutex<>> data access. */
        #[test]
        fn test_vcpu_run() {
            let vcpu_num = 4;
            let mut vm_config = test_vm_config_create();
            vm_config.vcpu_count = vcpu_num;
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);
            let mut vcpu_handle: Vec<thread::JoinHandle<()>> = Vec::new();
            let mut handle: thread::JoinHandle<()>;

            for i in &mut vm.vcpus {
                /* Get a clone for the closure */
                let vcpu = i.clone();

                /* Start vcpu threads! */
                handle = thread::spawn(move || {
                    /* TODO: thread_vcpu_run */
                    vcpu.test_change_guest_ctx();
                });

                vcpu_handle.push(handle);
            }

            /* All the vcpu thread finish */
            for i in vcpu_handle {
                i.join().unwrap();
            }

            /* Check the guest contexxt */
            let gpreg;
            let sysreg;
            let hypreg;

            gpreg = vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs
                .x_reg[10];
            sysreg = vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.sys_regs
                .huvsscratch;
            hypreg = vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.hyp_regs
                .hutinst;

            assert_eq!(gpreg, 10);
            assert_eq!(sysreg, 11);
            assert_eq!(hypreg, 12);

            /* 
             * The result should be 400 to prove the main thread can get the 
             * correct value.
             */
            let result = vm.vm_state.lock().unwrap().vm_id;
            assert_eq!(result, vcpu_num * 100);
        }

        #[test]
        fn test_vcpu_ecall_exit() { 
            let vcpu_id = 0;
            let vm_config = test_vm_config_create();
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let fd = vm.vm_state.lock().unwrap().ioctl_fd;
            let vm_mutex = vm.vm_state;
            let console = Arc::new(Mutex::new(Tty::new()));
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console);
            let res;
            let version: u64 = 0;
            let test_buf: u64;
            let test_buf_pfn: u64;
            let test_buf_size: usize = 64 << 20;
            let mut hugatp: u64;

            println!("---test_vcpu_ecall_exit---");

            unsafe {
                /* ioctl */
                let version_ptr = (&version) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_GET_API_VERSION, version_ptr);
                println!("IOCTL_LAPUTA_GET_API_VERSION -  version : {:x}", 
                    version);

                let addr = 0 as *mut libc::c_void;
                let mmap_ptr = libc::mmap(addr, test_buf_size, 
                    libc::PROT_READ | libc::PROT_WRITE, 
                    libc::MAP_SHARED, fd, 0);
                assert_ne!(mmap_ptr, libc::MAP_FAILED);
                
                test_buf = mmap_ptr as u64; /* va */
                test_buf_pfn = test_buf; /* pa.pfn */
                let test_buf_pfn_ptr = (&test_buf_pfn) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_QUERY_PFN, test_buf_pfn_ptr);
                println!("IOCTL_LAPUTA_QUERY_PFN -  test_buf_pfn : {:x}", 
                    test_buf_pfn);
                
                /* set test code */
                let start = vcpu_ecall_exit as u64;
                let end = vcpu_ecall_exit_end as u64;
                let code_buf = test_buf + PAGE_TABLE_REGION_SIZE;

                std::ptr::copy_nonoverlapping(vcpu_ecall_exit as *const u8,
                    code_buf as *mut u8, (end - start) as usize);

                /* set hugatp */
                hugatp = test_buf;
                let pte_ptr = (hugatp + 8 * (((test_buf_pfn << PAGE_SIZE_SHIFT)
                     + PAGE_TABLE_REGION_SIZE) >> 30)) as *mut u64;

                let pte_ptr_value = pte_ptr as u64;
                println!("pte_ptr_value {}", pte_ptr_value);

                /* 512G 1-level direct mapping */
                *pte_ptr = (((test_buf_pfn << PAGE_SIZE_SHIFT) >> 30) << 28)
                    | 0x1f;
                println!("PTE : {:x}", *pte_ptr);

                /* delegate vs-ecall and guest page fault */
                virtualmachine::VirtualMachine::hu_delegation(fd);

                res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
                println!("IOCTL_LAPUTA_REGISTER_VCPU : {}", res);
            }

            let uepc: u64;
            let utval: u64;
            let ucause: u64;

            // FIXME: deadlock if ptr & ptr_u64 are not declared independently
            let ptr: *const VcpuCtx;
            let ptr_u64: u64;
            ptr = &*vcpu.vcpu_ctx.lock().unwrap() as *const VcpuCtx;
            ptr_u64 = ptr as u64;
            println!("the ptr is {:x}", ptr_u64);

            let target_code = ((test_buf_pfn << PAGE_SHIFT) 
                + PAGE_TABLE_REGION_SIZE) as u64;
            vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc = target_code;
                

            hugatp = test_buf_pfn | (8 << 60);
            vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hugatp = hugatp;

            unsafe {
                csrw!(HUGATP, hugatp);
                /* set hugatp */
                println!("HUGATP : 0x{:x}", hugatp);
                /* hustatus.SPP=1 .SPVP=1 uret to VS mode */
                vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hustatus = 
                    ((1 << HUSTATUS_SPV_SHIFT)
                    | (1 << HUSTATUS_SPVP_SHIFT)) as u64;

                /* set utvec to trap handler */
                csrw!(UTVEC, exit_guest as u64);
                enter_guest(ptr_u64);

                uepc = vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc;
                utval = vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.utval;
                ucause = vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.ucause;

                let a7 = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[17];

                println!("guest hyp uepc 0x{:x}", uepc);
                println!("guest hyp utval 0x{:x}", utval);
                println!("guest hyp ucause 0x{:x}", ucause);
                println!("guest hyp a7 0x{:x}", a7);
            }

            assert_eq!(uepc, ((test_buf_pfn << PAGE_SIZE_SHIFT)
                + PAGE_TABLE_REGION_SIZE) + 4);
            assert_eq!(utval, 0);
            assert_eq!(ucause, 10);
        }

        #[test]
        fn test_vcpu_add_all_gprs() { 
            let vcpu_id = 0;
            let vm_config = test_vm_config_create();
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let fd = vm.vm_state.lock().unwrap().ioctl_fd;
            let vm_mutex = vm.vm_state;
            let console = Arc::new(Mutex::new(Tty::new()));
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console);
            let res;
            let version: u64 = 0;
            let test_buf: u64;
            let test_buf_pfn: u64;
            let test_buf_size: usize = 64 << 20; /* 64 MB */
            let size: u64;
            let mut hugatp: u64;

            println!("---test_vcpu_add_all_gprs---");

            unsafe {
                /* ioctl */
                let version_ptr = (&version) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_GET_API_VERSION, version_ptr);
                println!("IOCTL_LAPUTA_GET_API_VERSION -  version : {:x}",
                    version);

                let addr = 0 as *mut libc::c_void;
                let mmap_ptr = libc::mmap(addr, test_buf_size, 
                    libc::PROT_READ | libc::PROT_WRITE, 
                    libc::MAP_SHARED, fd, 0);
                assert_ne!(mmap_ptr, libc::MAP_FAILED);
                
                test_buf = mmap_ptr as u64; /* va */
                test_buf_pfn = test_buf; /* pa.pfn */
                let test_buf_pfn_ptr = (&test_buf_pfn) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_QUERY_PFN, test_buf_pfn_ptr);
                println!("IOCTL_LAPUTA_QUERY_PFN -  test_buf_pfn : {:x}",
                    test_buf_pfn);
                
                /* set test code */
                let start = vcpu_add_all_gprs as u64;
                let end = vcpu_add_all_gprs_end as u64;
                size = end - start;
                let code_buf = test_buf + PAGE_TABLE_REGION_SIZE;

                std::ptr::copy_nonoverlapping(vcpu_add_all_gprs as *const u8,
                    code_buf as *mut u8, size as usize);

                /* set hugatp */
                hugatp = test_buf;
                let pte_ptr = (hugatp + 8 * (((test_buf_pfn << PAGE_SIZE_SHIFT)
                    + PAGE_TABLE_REGION_SIZE) >> 30)) as *mut u64;

                let pte_ptr_value = pte_ptr as u64;
                println!("pte_ptr_value {}", pte_ptr_value);

                /* 512G 1-level direct mapping */
                *pte_ptr = (((test_buf_pfn << PAGE_SIZE_SHIFT) >> 30) << 28)
                    | 0x1f;
                println!("PTE : {:x}", *pte_ptr);

                /* delegate vs-ecall and guest page fault */
                virtualmachine::VirtualMachine::hu_delegation(fd);

                res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
                println!("IOCTL_LAPUTA_REGISTER_VCPU : {}", res);
            }

            let uepc: u64;
            let utval: u64;
            let ucause: u64;

            // FIXME: deadlock if ptr & ptr_u64 are not declared independently
            let ptr: *const VcpuCtx;
            let ptr_u64: u64;
            ptr = &*vcpu.vcpu_ctx.lock().unwrap() as *const VcpuCtx;
            ptr_u64 = ptr as u64;
            println!("the ptr is {:x}", ptr_u64);

            let target_code = ((test_buf_pfn << PAGE_SHIFT) 
                + PAGE_TABLE_REGION_SIZE) as u64;
            vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc = target_code;

            hugatp = test_buf_pfn | (8 << 60);
            vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hugatp = hugatp;

            let mut sum = 0;
            let len = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg.len();
            for i in 0..len {
                vcpu.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[i] = i as u64;
                sum += i as u64;
            }

            sum += 10 - 1;
            println!("sum {}", sum);

            unsafe {
                csrw!(HUGATP, hugatp);
                /* set hugatp */
                println!("HUGATP : 0x{:x}", hugatp);
                /* hustatus.SPP=1 .SPVP=1 uret to VS mode */
                vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hustatus = 
                    ((1 << HUSTATUS_SPV_SHIFT) 
                    | (1 << HUSTATUS_SPVP_SHIFT)) as u64;
                /* set utvec to trap handler */
                csrw!(UTVEC, exit_guest as u64);
                enter_guest(ptr_u64);

                uepc = vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc;
                utval = vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.utval;
                ucause = vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.ucause;

                let a7 = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[17];

                println!("guest hyp uepc 0x{:x}", uepc);
                println!("guest hyp utval 0x{:x}", utval);
                println!("guest hyp ucause 0x{:x}", ucause);
                println!("guest hyp a7 0x{:x}", a7);
            }

            assert_eq!(sum, vcpu.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]);
            assert_eq!(uepc, ((test_buf_pfn << PAGE_SIZE_SHIFT)
                + PAGE_TABLE_REGION_SIZE) + size - 4);
            assert_eq!(utval, 0);
            assert_eq!(ucause, 10);
        }

        #[test]
        fn test_tty_sd() { 
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/tty_store.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                = entry_point;
            
            vm.vm_run();

            /* Answer */
            let ans_dlm = 0xfe;
            let ans_fcr = 0x6;
            let ans_lcr = 0x80;
            let ans_mcr = 0x8;
            let ans_scr = 0x0;
            let ans_ier = 0xf;

            /* test data */
            let dlm = vm.console.lock().unwrap().value[UART_DLM];
            let fcr = vm.console.lock().unwrap().value[UART_FCR];
            let lcr = vm.console.lock().unwrap().value[UART_LCR];
            let mcr = vm.console.lock().unwrap().value[UART_MCR];
            let scr = vm.console.lock().unwrap().value[UART_SCR];
            let ier = vm.console.lock().unwrap().value[UART_IER];

            vm.vm_destroy();

            assert_eq!(dlm, ans_dlm);
            assert_eq!(fcr, ans_fcr);
            assert_eq!(lcr, ans_lcr);
            assert_eq!(mcr, ans_mcr);
            assert_eq!(scr, ans_scr);
            assert_eq!(ier, ans_ier);
        }

        #[test]
        fn test_tty_ld() { 
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/tty_load.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            /* Answer will be saved at 0x3000(gpa) */
            let retval: u64;

            /* 
             * Answer should be: 
             * 0x3f8 = 0x0
             * 0x3f9 = 0x0
             * 0x3fa = 0xc0 = UART_IIR_TYPE_BITS
             * 0x3fb = 0x0
             * 0x3fc = 0x08 = UART_MCR_OUT2
             * 0x3fd = 0x60 = UART_LSR_TEMT | UART_LSR_THRE
             * 0x3fe = 0xb0 = UART_MSR_DCD | UART_MSR_DSR | UART_MSR_CTS
             * 0x3ff = 0x0
             */
            let answer: u64 = 0xb0600800c00000;

            vm.vm_init();

            /* the return value will be stored on this gpa */
            let target_address = 0x3000;

            /* set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            let res = vm.vm_state.lock().unwrap()
                .gsmmu.gpa_block_add(target_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!");
            }

            /* get the hva of 0x3000(gpa) */
            let (hva, hpa) = res.unwrap();
            dbgprintln!("hva {:x}, hpa {:x}", hva, hpa);

            /* map the page on g-stage */
            let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                    | PTE_EXECUTE;
            vm.vm_state.lock().unwrap().gsmmu.map_page(target_address, hpa, 
                    flag);

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            
            vm.vm_run();

            /* check the return value store by the vm */
            unsafe {
                retval = *(hva as *mut u64);
                dbgprintln!("retval 0x{:x}", retval);
                assert_eq!(answer, retval);
            }

            vm.vm_destroy();
        }
    }
}

