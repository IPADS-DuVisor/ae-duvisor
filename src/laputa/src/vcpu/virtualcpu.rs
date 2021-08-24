use crate::vm::virtualmachine;
use crate::irq::virq;
use crate::vcpu::vcpucontext;
use std::sync::{Arc, Mutex, RwLock};
use vcpucontext::*;
use crate::mm::utils::*;
use crate::mm::gstagemmu::*;
use crate::plat::uhe::ioctl::ioctl_constants::*;
use crate::irq::delegation::delegation_constants::*;
use crate::plat::uhe::csr::csr_constants;
use csr_constants::*;
use crate::plat::opensbi;
use crate::vcpu::utils::*;
use std::lazy::SyncOnceCell;
use crate::devices::tty::Tty;

extern crate irq_util;
use irq_util::IrqChip;

extern crate devices;
extern crate sys_util;
use sys_util::GuestMemory;

#[allow(unused)]
mod errno_constants {
    pub const EFAILED: i32 = -1;
    pub const ENOPERMIT: i32 = -2;
    pub const ENOMAPPING: i32 = -3;
}
pub use errno_constants::*;

mod inst_parsing_constants {
    pub const INST_OPCODE_MASK: u32 =   0x007c;
    pub const INST_OPCODE_SHIFT: u32 =  2;
    pub const INST_OPCODE_SYSTEM: u32 = 28;
    
    pub const INST_MASK_WFI: u32 =	0xffffff00;
    pub const INST_MATCH_WFI: u32 =	0x10500000;
    
    pub const INST_MATCH_LB: u32 =	0x3;
    pub const INST_MASK_LB: u32 =	0x707f;
    pub const INST_MATCH_LH: u32 =	0x1003;
    pub const INST_MASK_LH: u32 =	0x707f;
    pub const INST_MATCH_LW: u32 =	0x2003;
    pub const INST_MASK_LW: u32 =	0x707f;
    pub const INST_MATCH_LD: u32 =	0x3003;
    pub const INST_MASK_LD: u32 =	0x707f;
    pub const INST_MATCH_LBU: u32 =	0x4003;
    pub const INST_MASK_LBU: u32 =	0x707f;
    pub const INST_MATCH_LHU: u32 =	0x5003;
    pub const INST_MASK_LHU: u32 =	0x707f;
    pub const INST_MATCH_LWU: u32 =	0x6003;
    pub const INST_MASK_LWU: u32 =	0x707f;
    pub const INST_MATCH_SB: u32 =	0x23;
    pub const INST_MASK_SB: u32 =	0x707f;
    pub const INST_MATCH_SH: u32 =	0x1023;
    pub const INST_MASK_SH: u32 =	0x707f;
    pub const INST_MATCH_SW: u32 =	0x2023;
    pub const INST_MASK_SW: u32 =	0x707f;
    pub const INST_MATCH_SD: u32 =	0x3023;
    pub const INST_MASK_SD: u32 =	0x707f;
    
    pub const INST_MATCH_C_LD: u32 =	0x6000;
    pub const INST_MASK_C_LD: u32 =	0xe003;
    pub const INST_MATCH_C_SD: u32 =	0xe000;
    pub const INST_MASK_C_SD: u32 =	0xe003;
    pub const INST_MATCH_C_LW: u32 =	0x4000;
    pub const INST_MASK_C_LW: u32 =	0xe003;
    pub const INST_MATCH_C_SW: u32 =	0xc000;
    pub const INST_MASK_C_SW: u32 =	0xe003;
    pub const INST_MATCH_C_LDSP: u32 =	0x6002;
    pub const INST_MASK_C_LDSP: u32 =	0xe003;
    pub const INST_MATCH_C_SDSP: u32 =	0xe002;
    pub const INST_MASK_C_SDSP: u32 =	0xe003;
    pub const INST_MATCH_C_LWSP: u32 =	0x4002;
    pub const INST_MASK_C_LWSP: u32 =	0xe003;
    pub const INST_MATCH_C_SWSP: u32 =	0xc002;
    pub const INST_MASK_C_SWSP: u32 =	0xe003;
}
pub use inst_parsing_constants::*;

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
    pub virq: virq::VirtualInterrupt,
    /* Cell for late init */
    pub irqchip: SyncOnceCell<Arc<dyn IrqChip>>,
    /* TODO: irq_pending with shared memory */
    pub exit_reason: Mutex<ExitReason>,
    pub console: Arc<Mutex<Tty>>,
    pub guest_mem: GuestMemory,
    pub mmio_bus: Arc<RwLock<devices::Bus>>,
}

impl VirtualCpu {
    pub fn new(vcpu_id: u32,
            vm_mutex_ptr: Arc<Mutex<virtualmachine::VmSharedState>>,
            console: Arc<Mutex<Tty>>, guest_mem: GuestMemory, 
            mmio_bus: Arc<RwLock<devices::Bus>>) -> Self {
        let vcpu_ctx = Mutex::new(VcpuCtx::new());
        let virq = virq::VirtualInterrupt::new();
        let exit_reason = Mutex::new(ExitReason::ExitUnknown);
        let irqchip = SyncOnceCell::new();

        Self {
            vcpu_id,
            vm: vm_mutex_ptr,
            vcpu_ctx,
            virq,
            irqchip,
            exit_reason,
            console,
            guest_mem,
            mmio_bus,
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
        let hugatp: u64;

        if S2PT_MODE == 3 {
            hugatp = pt_pfn | HUGATP_MODE_SV39;
        } else if S2PT_MODE == 4 {
            hugatp = pt_pfn | HUGATP_MODE_SV48;
        } else {
            panic!("Invalid S2PT_MODE");
        }

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
        /* Set virtual timer */
        self.virq.set_pending_irq(IRQ_VS_TIMER);
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

    fn get_vm_inst_by_uepc(&self, read_insn: bool) -> u32 {
        let uepc = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc;
        let val: u32;

        /* FIXME: why KVM swap HSTATUS & STVEC here? */

        if read_insn {
            unsafe {
                asm!(
                    ".option push",
                    ".option norvc",

                    /* HULVX.HU t0, (t2) */
                    ".word 0x6433c2f3",
                    "andi t1, t0, 3",
                    "addi t1, t1, -3",
                    "bne t1, zero, 2f",
                    "addi t2, t2, 2",

                    /* HULVX.HU t1, (t2) */
                    ".word 0x6433c373",
                    "sll t1, t1, 16",
                    "add t0, t0, t1",
                    "2:",
                    ".option pop",
                    out("t0") val,
                    in("t2") uepc,
                );
            }
            dbgprintln!("HLVX.HU val: {:x}, uepc: {:x}", val, uepc);
        } else {
            /* TODO: HLV.D for IPI ECALL emulation */
            val = 0;
        }
        return val;
    }

    fn parse_load_inst(&self, inst: u32, inst_len: &mut u64, 
        bit_width: &mut u64, target_reg: &mut u64) {
        /* 16BIT_MASK = 0x3 */
        *inst_len = if inst & 0x3 != 0x3 { 2 } else { 4 };
        if *inst_len == 2 {
            /* Compressed instruction */
            let c_lw_mask = 0b11 | (0b111 << 13); 
            let c_lw_match = 0b00 | (0b010 << 13); 
            let c_lw_rd = |inst: u32| -> u32 { ((inst >> 2) & 0x7) + 8 }; 
            
            if (inst & c_lw_mask) == c_lw_match {
                *target_reg = c_lw_rd(inst) as u64;
                *bit_width = 4 * 8;
                dbgprintln!("--- LW: inst {:x}, inst_len {:x}, reg: {}", 
                    inst, inst_len, target_reg);
            } else {
                panic!("parse_load_inst: unsupported inst {:x}, inst_len {:x}", 
                    inst, inst_len);
            }
        } else {
            /* TODO: refactor get_*_reg */
            let i_rd_reg = |inst: u32| -> u32 { (inst >> 7) & 0x1f };
            *target_reg = i_rd_reg(inst) as u64;

            if (inst & INST_MASK_LW) == INST_MATCH_LW {
                *bit_width = 4 * 8;
            } else if (inst & INST_MASK_LB) == INST_MATCH_LB {
                *bit_width = 1 * 8;
            } else {
                panic!("parse_load_inst: unsupported inst {:x}, inst_len {:x}", 
                    inst, inst_len);
            }
        }
    }

    fn parse_store_inst(&self, inst: u32, inst_len: &mut u64, 
        bit_width: &mut u64, target_reg: &mut u64) {
        /* 16BIT_MASK = 0x3 */
        *inst_len = if inst & 0x3 != 0x3 { 2 } else { 4 };
        if *inst_len == 2 {
            /* Compressed instruction */
            let c_sw_mask = 0b11 | (0b111 << 13); 
            let c_sw_match = 0b00 | (0b110 << 13); 
            let c_sw_rs2 = |inst: u32| -> u32 { ((inst >> 2) & 0x7) + 8 }; 
            
            if (inst & c_sw_mask) == c_sw_match {
                *target_reg = c_sw_rs2(inst) as u64;
                *bit_width = 4 * 8;
                dbgprintln!("--- SW: inst {:x}, inst_len {:x}, reg: {}", 
                    inst, inst_len, target_reg);
            } else {
                panic!("parse_store_inst: unsupported inst {:x}, inst_len {:x}", 
                    inst, inst_len);
            }
        } else {
            let s_rs2_reg = |inst: u32| -> u32 { (inst >> 20) & 0x1f };
            *target_reg = s_rs2_reg(inst) as u64;

            if (inst & INST_MASK_SW) == INST_MATCH_SW {
                *bit_width = 4 * 8;
            } else if (inst & INST_MASK_SB) == INST_MATCH_SB {
                *bit_width = 1 * 8;
            } else {
                panic!("parse_store_inst: unsupported inst {:x}, inst_len {:x}", 
                    inst, inst_len);
            }
        }
    }

    fn store_emulation(&self, fault_addr: u64, target_reg: u64,
                bit_width: u64) -> i32 {
        let mut ret: i32 = 0;
        let bit_mask: u64 = (1 << bit_width) - 1;
        let mut data: u32 = (self.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs
                .x_reg[target_reg as usize] & bit_mask) as u32;

        /* TODO: replce with MMIO bus */
        let is_irqchip_mmio = if 0xc000000 <= fault_addr && 
            fault_addr < (0xc000000 + 0x1000000) { true } else { false };

        if is_irqchip_mmio {
            self.irqchip.get().unwrap().mmio_callback(fault_addr, &mut data, true);
        } else if fault_addr >= 0x3f8 && fault_addr < 0x400 { /* TtyS0-3F8 */
            ret = self.console.lock().unwrap()
                .store_emulation(fault_addr, data as u8, &self.irqchip.get().unwrap());
        } else {
            let slice = &mut data.to_le_bytes();
            if self.mmio_bus.read().unwrap().write(fault_addr, slice) {
                ret = 0;
            } else {
                ret = 1;
                panic!("Unknown mmio (store) fault_addr: {:x}, ret {}", 
                    fault_addr, ret);
            }
        }

        return ret;
    }

    fn load_emulation(&self, fault_addr: u64, target_reg: u64,
                bit_width: u64) -> i32 {
        let mut ret: i32 = 0;
        let bit_mask: u64 = (1 << bit_width) - 1;
        let mut data: u32 = 0;

        let is_irqchip_mmio = if 0xc000000 <= fault_addr && 
            fault_addr < (0xc000000 + 0x1000000) { true } else { false };

        if is_irqchip_mmio {
            self.irqchip.get().unwrap().mmio_callback(fault_addr, &mut data, false);
        } else if fault_addr >= 0x3f8 && fault_addr < 0x400 { /* TtyS0-3F8 */
            data = self.console.lock().unwrap().
                load_emulation(fault_addr, &self.irqchip.get().unwrap()) as u32;
        } else {
            let slice = &mut data.to_le_bytes();
            if self.mmio_bus.read().unwrap().read(fault_addr, slice) {
                data = u32::from_le_bytes(*slice);
                ret = 0;
            } else {
                ret = 1;
                panic!("Unknown mmio (load) fault_addr: {:x}, ret {}", fault_addr, ret);
            }
        }
        self.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.
            x_reg[target_reg as usize] = (data as u64) & bit_mask;

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
        let hutinst = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hutinst;
        let inst: u32;
        let mut target_reg: u64 = 0xffff;
        let mut bit_width: u64 = 0;
        let mut inst_len: u64 = 0;
        let ret: i32;

        if hutinst == 0x0 {
            /* The implementation has not support the function of hutinst */
            inst = self.get_vm_inst_by_uepc(true);
        } else {
            inst = hutinst as u32;
        }

        if ucause == EXC_LOAD_GUEST_PAGE_FAULT {
            self.parse_load_inst(inst, &mut inst_len, &mut bit_width, &mut target_reg);
        } else {
            self.parse_store_inst(inst, &mut inst_len, &mut bit_width, &mut target_reg);
        }

        if ucause == EXC_LOAD_GUEST_PAGE_FAULT {
            /* Load */
            ret = self.load_emulation(fault_addr, target_reg, bit_width);
        } else if ucause == EXC_STORE_GUEST_PAGE_FAULT {
            /* Store */
            ret = self.store_emulation(fault_addr, target_reg, bit_width);
        } else {
            ret = 1;
        }

        self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc += inst_len;

        return ret;
    }

    fn handle_stage2_page_fault(&self) -> i32 {
        let hutval = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hutval;
        let utval = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.utval;
        let mut fault_addr = (hutval << 2) | (utval & 0x3);
        let mut ret;

        dbgprintln!("gstage fault: hutval: {:x}, utval: {:x}, fault_addr: {:x}",
            hutval, utval, fault_addr);
        
        let gpa_check = self.vm.lock().unwrap().gsmmu.check_gpa(fault_addr);
        if !gpa_check {
            /* Maybe mmio or illegal gpa */
            let mmio_check = self.vm.lock().unwrap().gsmmu.check_mmio(fault_addr);

            if !mmio_check {
                panic!("Invalid gpa! {:x}", fault_addr);
            }

            ret = self.handle_mmio(fault_addr);

            return ret;
        }

        fault_addr &= !PAGE_SIZE_MASK;

        /* Map query */
        let query = self.vm.lock().unwrap().gsmmu.map_query(fault_addr);
        if query.is_some() {
            let i = query.unwrap();

            dbgprintln!("Query PTE offset {}, value {}, level {}", i.offset, 
                i.value, i.level);

            if i.is_leaf() {
                let ucause 
                    = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.ucause;

                /* No permission */
                if ucause == EXC_LOAD_GUEST_PAGE_FAULT 
                    && (i.value & PTE_READ) == 0 {
                    ret = ENOPERMIT;
                } else if ucause == EXC_STORE_GUEST_PAGE_FAULT 
                    && (i.value & PTE_WRITE) == 0 {
                    ret = ENOPERMIT;
                } else if ucause == EXC_INST_GUEST_PAGE_FAULT 
                    && (i.value & PTE_EXECUTE) == 0 {
                    ret = ENOPERMIT;
                }else {
                    /* S2PT contention with other vcpus */
                    return 0;
                }
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
                /* Find hpa by fault_addr */
                let fault_addr_query = self.vm.lock().unwrap().gsmmu
                    .gpa_block_query(fault_addr);

                if fault_addr_query.is_some() {
                    /* Fault gpa is already in a gpa_block and it is valid */
                    let fault_hva = fault_addr_query.unwrap().0;
                    let fault_hpa = fault_addr_query.unwrap().1;
                    let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE
                        | PTE_EXECUTE;
                        
                    dbgprintln!("map gpa: {:x} to hpa: {:x}",
                        fault_addr, fault_hpa);
                    self.vm.lock().unwrap().gsmmu.map_page(
                        fault_addr, fault_hpa, flag);
                    
                    /* Record the HVA <--> GPA mapping*/
                    self.guest_mem.insert_region(fault_hva, fault_addr, 
                        PAGE_SIZE as usize);

                    ret = 0;
                } else {
                    /* Fault gpa is not in a gpa_block and it is valid */
                    let len = PAGE_SIZE;
                    let res = self.vm.lock().unwrap().gsmmu
                        .gpa_block_add(fault_addr, len);

                    if res.is_ok() {
                        /* Map new page to VM if the region exists */
                        let (hva, hpa) = res.unwrap();
                        let flag: u64 = PTE_USER | PTE_VALID | PTE_READ 
                            | PTE_WRITE | PTE_EXECUTE;

                        self.vm.lock().unwrap().gsmmu.map_page(
                            fault_addr, hpa, flag);

                        /* Record the HVA <--> GPA mapping*/
                        self.guest_mem.insert_region(hva, fault_addr, len as usize);

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
        let a0 = vcpu_ctx.guest_ctx.gp_regs.x_reg[10]; /* A0: 0th arg/ret 1 */
        let a1 = vcpu_ctx.guest_ctx.gp_regs.x_reg[11]; /* A1: 1st arg/ret 2 */
        let a2 = vcpu_ctx.guest_ctx.gp_regs.x_reg[11]; /* A2: 2nd arg  */
        let a3 = vcpu_ctx.guest_ctx.gp_regs.x_reg[11]; /* A3: 3rd arg */ 
        let a4 = vcpu_ctx.guest_ctx.gp_regs.x_reg[11]; /* A4: 4th arg  */
        let a5 = vcpu_ctx.guest_ctx.gp_regs.x_reg[11]; /* A5: 5th arg  */
        let a6 = vcpu_ctx.guest_ctx.gp_regs.x_reg[16]; /* A6: FID */
        let a7 = vcpu_ctx.guest_ctx.gp_regs.x_reg[17]; /* A7: EID */

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

        /* Save the result */
        vcpu_ctx.guest_ctx.gp_regs.x_reg[10] = target_ecall.ret[0];
        vcpu_ctx.guest_ctx.gp_regs.x_reg[11] = target_ecall.ret[1];

        /* Add uepc to start vm on next instruction */
        vcpu_ctx.host_ctx.hyp_regs.uepc += 4;

        ret
    }

    fn handle_vcpu_exit(&self) -> i32 {
        let mut ret: i32 = -1;
        let ucause = self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.ucause;
        *self.exit_reason.lock().unwrap() = ExitReason::ExitUnknown;
        
        if (ucause & EXC_IRQ_MASK) != 0 {
            *self.exit_reason.lock().unwrap() = ExitReason::ExitIntr;
            let ucause = ucause & (!EXC_IRQ_MASK);
            match ucause {
                IRQ_U_VTIMER => {
                    dbgprintln!("handler U VTIMER: {}, current pc is {:x}.", 
                        ucause, self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc);
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

    pub fn thread_vcpu_run(&self, delta_time: i64) -> i32 {
        let fd = self.vm.lock().unwrap().gsmmu.allocator.ioctl_fd;
        let mut _res;

        self.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hustatus = 
            ((1 << HUSTATUS_SPV_SHIFT) | (1 << HUSTATUS_SPVP_SHIFT)) | 
            (1 << HUSTATUS_UPIE_SHIFT) as u64;

        unsafe {
            /* Register vcpu thread to the kernel */
            _res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
            dbgprintln!("IOCTL_LAPUTA_REGISTER_VCPU : {}", _res);

            /* Set hugatp */
            let _hugatp = self.config_hugatp();
            dbgprintln!("Config hugatp: {:x}", _hugatp);

            /* Set trap handler */
            csrw!(UTVEC, exit_guest as u64);

            /* Enable timer irq */
            csrw!(HUIE, 1 << IRQ_U_VTIMER);

            /* TODO: redesign scounteren register */
            /* Allow VM to directly access time register */

            /* TODO: introduce RUST feature to distinguish between rv64 and rv32 */
            csrw!(HUTIMEDELTA, -delta_time as u64);
        }
        /* FIXME: deadlock if ptr & ptr_u64 are not declared independently */
        let vcpu_ctx_ptr: *const VcpuCtx;
        let vcpu_ctx_ptr_u64: u64;
        vcpu_ctx_ptr = &*self.vcpu_ctx.lock().unwrap() as *const VcpuCtx;
        vcpu_ctx_ptr_u64 = vcpu_ctx_ptr as u64;
        
        let mut ret: i32 = 0;
        while ret == 0 {
            /* Insert or clear tty irq on each vtimer irq */
            self.console.lock().unwrap().update_recv(&self.irqchip.get().unwrap());

            /* Flush pending irqs into HUVIP */
            self.virq.flush_pending_irq();

            unsafe {
                enter_guest(vcpu_ctx_ptr_u64);
            }

            /* FIXME: why KVM need this? */
            //self.virq.sync_pending_irq();

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
    use crate::test::utils::configtest::test_vm_config_create;

    rusty_fork_test! {
        #[test]
        fn test_handle_stage2_page_fault() { 
            let vcpu_id = 0;
            let vm_config = test_vm_config_create();
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let fd = vm.vm_state.lock().unwrap().ioctl_fd;
            let vm_mutex = vm.vm_state;
            let console = Arc::new(Mutex::new(Tty::new()));
            let mmio_bus = Arc::new(RwLock::new(devices::Bus::new()));
            let guest_mem = GuestMemory::new().unwrap();
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console, guest_mem, mmio_bus);
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
                *test_buf_ptr = 0x73; /* Ecall */
                test_buf_ptr = (test_buf + 4) as *mut i32;
                *test_buf_ptr = 0xa001; /* Loop */

                /* 512G 1-level direct mapping */
                hugatp = test_buf + PAGE_SIZE * 4;
                let pte_ptr = (hugatp + 8 * ((test_buf_pfn << PAGE_SIZE_SHIFT)
                     >> 30)) as *mut u64;
                *pte_ptr = (((test_buf_pfn << PAGE_SIZE_SHIFT) >> 30) << 28) | 
                    0x1f;
                println!("PTE : {:x}", *pte_ptr);

                /* Delegate vs-ecall and guest page fault */
                virtualmachine::VirtualMachine::hu_delegation(fd);

                res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
                println!("IOCTL_LAPUTA_REGISTER_VCPU : {}", res);
            }

            let mut uepc: u64 = 0;
            let mut utval: u64 = 0;
            let mut ucause: u64 = 0;

            /* FIXME: deadlock if ptr & ptr_u64 are not declared independently */
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

                    /* HUSTATUS.SPP=1 .SPVP=1 uret to VS mode */
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
            let mmio_bus = Arc::new(RwLock::new(devices::Bus::new()));
            let guest_mem = GuestMemory::new().unwrap();
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console, guest_mem, mmio_bus);

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
            let mmio_bus = Arc::new(RwLock::new(devices::Bus::new()));
            let guest_mem = GuestMemory::new().unwrap();
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console, guest_mem, mmio_bus);

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
            let mmio_bus = Arc::new(RwLock::new(devices::Bus::new()));
            let guest_mem = GuestMemory::new().unwrap();
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console, guest_mem, mmio_bus);
            let ans = 17;

            /* Guest ctx */
            vcpu.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10] = ans;
            let tmp = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10];
            assert_eq!(tmp, ans);

            vcpu.vcpu_ctx.lock().unwrap().guest_ctx.sys_regs.huvsatp = ans;
            let tmp = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.sys_regs.huvsatp;
            assert_eq!(tmp, ans);

            vcpu.vcpu_ctx.lock().unwrap().guest_ctx.hyp_regs.hutinst = ans;
            let tmp = vcpu.vcpu_ctx.lock().unwrap().guest_ctx.hyp_regs.hutinst;
            assert_eq!(tmp, ans);

            /* Host ctx */
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
            let mmio_bus = Arc::new(RwLock::new(devices::Bus::new()));
            let guest_mem = GuestMemory::new().unwrap();
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console, guest_mem, mmio_bus);
            let res;
            let version: u64 = 0;
            let test_buf: u64;
            let test_buf_pfn: u64;
            let test_buf_size: usize = 64 << 20;
            let mut hugatp: u64;

            println!("---test_vcpu_ecall_exit---");

            unsafe {
                /* Ioctl */
                let version_ptr = (&version) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_GET_API_VERSION, version_ptr);
                println!("IOCTL_LAPUTA_GET_API_VERSION -  version : {:x}", 
                    version);

                let addr = 0 as *mut libc::c_void;
                let mmap_ptr = libc::mmap(addr, test_buf_size, 
                    libc::PROT_READ | libc::PROT_WRITE, 
                    libc::MAP_SHARED, fd, 0);
                assert_ne!(mmap_ptr, libc::MAP_FAILED);
                
                test_buf = mmap_ptr as u64; /* VA */
                test_buf_pfn = test_buf; /* PA.PFN */
                let test_buf_pfn_ptr = (&test_buf_pfn) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_QUERY_PFN, test_buf_pfn_ptr);
                println!("IOCTL_LAPUTA_QUERY_PFN -  test_buf_pfn : {:x}", 
                    test_buf_pfn);
                
                /* Set test code */
                let start = vcpu_ecall_exit as u64;
                let end = vcpu_ecall_exit_end as u64;
                let code_buf = test_buf + PAGE_TABLE_REGION_SIZE;

                std::ptr::copy_nonoverlapping(vcpu_ecall_exit as *const u8,
                    code_buf as *mut u8, (end - start) as usize);

                /* Set hugatp */
                hugatp = test_buf;
                let pte_ptr = (hugatp + 8 * (((test_buf_pfn << PAGE_SIZE_SHIFT)
                     + PAGE_TABLE_REGION_SIZE) >> 30)) as *mut u64;

                let pte_ptr_value = pte_ptr as u64;
                println!("pte_ptr_value {}", pte_ptr_value);

                /* 512G 1-level direct mapping */
                *pte_ptr = (((test_buf_pfn << PAGE_SIZE_SHIFT) >> 30) << 28)
                    | 0x1f;
                println!("PTE : {:x}", *pte_ptr);

                /* Delegate vs-ecall and guest page fault */
                virtualmachine::VirtualMachine::hu_delegation(fd);

                res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
                println!("IOCTL_LAPUTA_REGISTER_VCPU : {}", res);
            }

            let uepc: u64;
            let utval: u64;
            let ucause: u64;

            /* FIXME: deadlock if ptr & ptr_u64 are not declared independently */
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
                /* Set hugatp */
                println!("HUGATP : 0x{:x}", hugatp);
                /* HUSTATUS.SPP=1 .SPVP=1 uret to VS mode */
                vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hustatus = 
                    ((1 << HUSTATUS_SPV_SHIFT)
                    | (1 << HUSTATUS_SPVP_SHIFT)) as u64;

                /* Set utvec to trap handler */
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
            let mmio_bus = Arc::new(RwLock::new(devices::Bus::new()));
            let guest_mem = GuestMemory::new().unwrap();
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex, console, guest_mem, mmio_bus);
            let res;
            let version: u64 = 0;
            let test_buf: u64;
            let test_buf_pfn: u64;
            let test_buf_size: usize = 64 << 20; /* 64 MB */
            let size: u64;
            let mut hugatp: u64;

            println!("---test_vcpu_add_all_gprs---");

            unsafe {
                /* Ioctl */
                let version_ptr = (&version) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_GET_API_VERSION, version_ptr);
                println!("IOCTL_LAPUTA_GET_API_VERSION -  version : {:x}",
                    version);

                let addr = 0 as *mut libc::c_void;
                let mmap_ptr = libc::mmap(addr, test_buf_size, 
                    libc::PROT_READ | libc::PROT_WRITE, 
                    libc::MAP_SHARED, fd, 0);
                assert_ne!(mmap_ptr, libc::MAP_FAILED);
                
                test_buf = mmap_ptr as u64; /* VA */
                test_buf_pfn = test_buf; /* PA.PFN */
                let test_buf_pfn_ptr = (&test_buf_pfn) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_QUERY_PFN, test_buf_pfn_ptr);
                println!("IOCTL_LAPUTA_QUERY_PFN -  test_buf_pfn : {:x}",
                    test_buf_pfn);
                
                /* Set test code */
                let start = vcpu_add_all_gprs as u64;
                let end = vcpu_add_all_gprs_end as u64;
                size = end - start;
                let code_buf = test_buf + PAGE_TABLE_REGION_SIZE;

                std::ptr::copy_nonoverlapping(vcpu_add_all_gprs as *const u8,
                    code_buf as *mut u8, size as usize);

                /* Set hugatp */
                hugatp = test_buf;
                let pte_ptr = (hugatp + 8 * (((test_buf_pfn << PAGE_SIZE_SHIFT)
                    + PAGE_TABLE_REGION_SIZE) >> 30)) as *mut u64;

                let pte_ptr_value = pte_ptr as u64;
                println!("pte_ptr_value {}", pte_ptr_value);

                /* 512G 1-level direct mapping */
                *pte_ptr = (((test_buf_pfn << PAGE_SIZE_SHIFT) >> 30) << 28)
                    | 0x1f;
                println!("PTE : {:x}", *pte_ptr);

                /* Delegate vs-ecall and guest page fault */
                virtualmachine::VirtualMachine::hu_delegation(fd);

                res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
                println!("IOCTL_LAPUTA_REGISTER_VCPU : {}", res);
            }

            let uepc: u64;
            let utval: u64;
            let ucause: u64;

            /* FIXME: deadlock if ptr & ptr_u64 are not declared independently */
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
                /* Set hugatp */
                println!("HUGATP : 0x{:x}", hugatp);
                /* HUSTATUS.SPP=1 .SPVP=1 uret to VS mode */
                vcpu.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hustatus = 
                    ((1 << HUSTATUS_SPV_SHIFT) 
                    | (1 << HUSTATUS_SPVP_SHIFT)) as u64;
                /* Set utvec to trap handler */
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
        fn test_tty_store() { 
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
            let ans_dlm = 0x0;
            let ans_dll = 0xc;
            let ans_fcr = 0x6;
            let ans_lcr = 0x0;
            let ans_mcr = 0x8;
            let ans_scr = 0x0;
            let ans_ier = 0xf;

            /* Test data */
            let dlm = vm.console.lock().unwrap().dlm;
            let dll = vm.console.lock().unwrap().dll;
            let fcr = vm.console.lock().unwrap().fcr;
            let lcr = vm.console.lock().unwrap().lcr;
            let mcr = vm.console.lock().unwrap().mcr;
            let scr = vm.console.lock().unwrap().scr;
            let ier = vm.console.lock().unwrap().ier;

            vm.vm_destroy();

            assert_eq!(dlm, ans_dlm);
            assert_eq!(dll, ans_dll);
            assert_eq!(fcr, ans_fcr);
            assert_eq!(lcr, ans_lcr);
            assert_eq!(mcr, ans_mcr);
            assert_eq!(scr, ans_scr);
            assert_eq!(ier, ans_ier);
        }

        #[test]
        fn test_tty_load() { 
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
             * 0x3fa = 0xc0 = UART_IIR_TYPE_BITS | UART_IIR_NO_INT
             * 0x3fb = 0x0
             * 0x3fc = 0x08 = UART_MCR_OUT2
             * 0x3fd = 0x60 = UART_LSR_TEMT | UART_LSR_THRE
             * 0x3fe = 0xb0 = UART_MSR_DCD | UART_MSR_DSR | UART_MSR_CTS
             * 0x3ff = 0x0
             */
            let answer: u64 = 0xb0600800c10000;

            vm.vm_init();

            /* The return value will be stored on this gpa */
            let target_address = 0x3000;

            /* Set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            let res = vm.vm_state.lock().unwrap()
                .gsmmu.gpa_block_add(target_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!");
            }

            /* Get the hva of 0x3000(gpa) */
            let (hva, hpa) = res.unwrap();
            dbgprintln!("hva {:x}, hpa {:x}", hva, hpa);

            /* Map the page on g-stage */
            let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                    | PTE_EXECUTE;
            vm.vm_state.lock().unwrap().gsmmu.map_page(target_address, hpa, 
                    flag);

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            
            vm.vm_run();

            /* Check the return value store by the vm */
            unsafe {
                retval = *(hva as *mut u64);
                dbgprintln!("retval 0x{:x}", retval);
                assert_eq!(answer, retval);
            }

            vm.vm_destroy();
        }
    }
}

