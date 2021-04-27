use crate::vm::virtualmachine;
use crate::irq::virq;
use crate::irq::vtimer;
use crate::vcpu::vcpucontext;
use std::sync::{Arc, Mutex};
use vcpucontext::*;
use crate::mm::gstagemmu::*;
use crate::plat::uhe::ioctl::ioctl_constants::*;
use crate::irq::delegation::delegation_constants::*;
use core::ffi::c_void;

global_asm!(include_str!("vm_code.S"));

mod errno_constants {
    pub const EFAILED: i32 = -1;
    pub const ENOPERMIT: i32 = -2;
    pub const ENOMAPPING: i32 = -3;
}
pub use errno_constants::*;

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
    // int enter_guest(struct VcpuCtx *ctx);
    fn enter_guest(vcpuctx: u64) -> i32;

    // void set_hugatp(uint64_t hugatp)
    fn set_hugatp(hugatp: u64);

    // void set_utvec()
    fn set_utvec();
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

    fn vm_code();
}

pub struct VirtualCpu {
    pub vcpu_id: u32,
    pub vm: Arc<Mutex<virtualmachine::VmSharedState>>,
    pub vcpu_ctx: VcpuCtx,
    pub virq: virq::VirtualInterrupt,
    pub vtimer: vtimer::VirtualTimer,
    // TODO: irq_pending with shared memory
    pub exit_reason: ExitReason,
}

impl VirtualCpu {
    pub fn new(vcpu_id: u32,
            vm_mutex_ptr: Arc<Mutex<virtualmachine::VmSharedState>>) -> Self {
        let vcpu_ctx = VcpuCtx::new();
        let virq = virq::VirtualInterrupt::new();
        let vtimer = vtimer::VirtualTimer::new(0, 0);
        let exit_reason = ExitReason::ExitUnknown;

        Self {
            vcpu_id,
            vm: vm_mutex_ptr,
            vcpu_ctx,
            virq,
            vtimer,
            exit_reason,
        }
    }

    // For test case: test_vm_run
    pub fn test_change_guest_ctx(&mut self) -> u32 {
        // Change guest context
        self.vcpu_ctx.guest_ctx.gp_regs.x_reg[10] += 10;
        self.vcpu_ctx.guest_ctx.sys_regs.huvsscratch += 11;
        self.vcpu_ctx.guest_ctx.hyp_regs.hutinst += 12;

        // Increse vm_id in vm_state
        self.vm.lock().unwrap().vm_id += 100;

        0
    }

    fn config_hugatp(&mut self) -> u64 {
        let pt_pfn: u64 = self.vm.lock().unwrap().gsmmu.page_table.paddr >> 12;
        let hugatp: u64 = pt_pfn | (9 << 60);

        self.vcpu_ctx.host_ctx.hyp_regs.hugatp = hugatp;

        unsafe {
            set_hugatp(hugatp);
        }

        hugatp
    }
    
    fn virtual_inst_fault(&mut self) -> i32 {
        let ret = 0;
        let utval = self.vcpu_ctx.host_ctx.hyp_regs.utval;
        println!("virtual_inst_fault: insn = {:x}", utval);
        
        ret
    }

    fn stage2_page_fault(&mut self) -> i32 {
        let hutval = self.vcpu_ctx.host_ctx.hyp_regs.hutval;
        let utval = self.vcpu_ctx.host_ctx.hyp_regs.utval;
        //let fault_addr = (hutval << 2) | (utval & 0x3);
        let fault_addr = utval;
        println!("gstage_page_fault: hutval: {:x}, utval: {:x}, fault_addr: {:x}", 
            hutval, utval, fault_addr);

        let mut ret;
        // map_query
        let query = self.vm.lock().unwrap().gsmmu.map_query(fault_addr);
        if query.is_some() {
            let i = query.unwrap();
            println!("Query PTE offset {}, value {}, level {}", i.offset, i.value, i.level);
            ret = ENOPERMIT;
        } else {
            ret = ENOMAPPING;
        }
        match ret {
            ENOPERMIT => {
                self.exit_reason = ExitReason::ExitEaccess;
                eprintln!("Query return ENOPERMIT: {}", ret);
            }
            ENOMAPPING => {
                println!("Query return ENOMAPPING: {}", ret);
                // find gpa region by fault_addr
                let len = 4096;
                let res = self.vm.lock().unwrap().
                    gsmmu.gpa_region_add(fault_addr, len);
                if res.is_ok() {
                    // map new region to VM if the region exists
                    let (hva, hpa) = res.unwrap();
                    println!("New hpa: {:x}", hpa);
                    unsafe {
                        let ptr = hva as *mut i32;

                        // test case
                        // set test code
                        let start = vcpu_add_all_gprs as u64;
                        let end = vcpu_add_all_gprs_end as u64;
                        let size = end - start;
                        //let code_buf = test_buf + PAGE_TABLE_REGION_SIZE;
                        libc::memcpy(ptr as *mut c_void, vcpu_add_all_gprs as *mut c_void, size as usize);

                        //*ptr = 0x73; // ecall
                    }
                    let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE | PTE_EXECUTE;
                    self.vm.lock().unwrap().gsmmu.map_page(
                        fault_addr, hpa, flag);
                    ret = 0;
                } else {
                    // handle MMIO otherwise
                    self.exit_reason = ExitReason::ExitMmio;
                    ret = EFAILED;
                    eprintln!("MMIO unsupported: {}", ret);
                }
            }
            _ => {
                self.exit_reason = ExitReason::ExitEaccess;
                eprintln!("Invalid query result: {}", ret);
            }
        }

        ret
    }

    fn supervisor_ecall(&mut self) -> i32 {
        let ret;
        let a0 = self.vcpu_ctx.guest_ctx.gp_regs.x_reg[10]; // a0: funcID
        let a1 = self.vcpu_ctx.guest_ctx.gp_regs.x_reg[11]; // a1: 1st arg 
        // ...
        let a7 = self.vcpu_ctx.guest_ctx.gp_regs.x_reg[17]; // a7: 7th arg
        println!("supervisor_ecall: funcID = {:x}, arg1 = {:x}, arg7 = {:x}",
            a0, a1, a7);
        // for test
        ret = 0xdead;
        
        ret
    }

    fn handle_vcpu_exit(&mut self) -> i32 {
        let mut ret: i32 = -1;
        let ucause = self.vcpu_ctx.host_ctx.hyp_regs.ucause;
        self.exit_reason = ExitReason::ExitUnknown;

        if (ucause & EXC_IRQ_MASK) != 0 {
            self.exit_reason = ExitReason::ExitIntr;
            return 1;
        }

        match ucause {
            EXC_VIRTUAL_INST_FAULT => {
                ret = self.virtual_inst_fault();
            }
            EXC_INST_GUEST_PAGE_FAULT | EXC_LOAD_GUEST_PAGE_FAULT |
                EXC_STORE_GUEST_PAGE_FAULT => {
                ret = self.stage2_page_fault();
            }
            EXC_SUPERVISOR_SYSCALL => {
                ret = self.supervisor_ecall();
            }
            _ => {
                eprintln!("Invalid ucause: {}", ucause);
            }
        }

        if ret < 0 {
            eprintln!("ERROR: handle_vcpu_exit ret: {}", ret);
        }

        ret
    }

    pub fn thread_vcpu_run(&mut self) -> i32 {
        let fd = self.vm.lock().unwrap().gsmmu.allocator.ioctl_fd;
        let mut res;
        self.vcpu_ctx.host_ctx.hyp_regs.uepc = 0x400000;
        self.vcpu_ctx.host_ctx.hyp_regs.hustatus = ((1 << 8) | (1 << 7)) as u64;

        unsafe {
            // register vcpu thread to the kernel
            res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
            println!("IOCTL_LAPUTA_REGISTER_VCPU : {}", res);

            // set hugatp
            let hugatp = self.config_hugatp();
            println!("Config hugatp: {:x}", hugatp);

            // set trap hadnler
            set_utvec();
        }
        
        let vcpu_ctx_ptr = &self.vcpu_ctx as *const VcpuCtx;
        let vcpu_ctx_ptr_u64 = vcpu_ctx_ptr as u64;
        
        let mut ret: i32 = 0;
        while ret == 0 {
            unsafe {
                enter_guest(vcpu_ctx_ptr_u64);
            }

            ret = self.handle_vcpu_exit();
        } 
        
        unsafe {
            res = libc::ioctl(fd, IOCTL_LAPUTA_UNREGISTER_VCPU);
            println!("IOCTL_LAPUTA_UNREGISTER_VCPU : {}", res);
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::ffi::CString;
    use crate::plat::uhe::ioctl::ioctl_constants;
    use crate::plat::uhe::csr::csr_constants;
    use crate::irq::delegation::delegation_constants;
    use ioctl_constants::*;
    use delegation_constants::*;
    use csr_constants::*;

    rusty_fork_test! {


        #[test]
        fn test_stage2_page_fault() { 
            let vcpu_id = 0;
            let vcpu_num = 1;
            let vm = virtualmachine::VirtualMachine::new(vcpu_num);
            let mut fd = vm.vm_state.lock().unwrap().ioctl_fd;
            let vm_mutex = vm.vm_state;
            let mut vcpu = VirtualCpu::new(vcpu_id, vm_mutex);
            let mut res;
            let version: u64 = 0;
            let mut test_buf: u64 = 0;
            let mut test_buf_pfn: u64 = 0;
            let test_buf_size: usize = 32 << 20;

            unsafe { 
                let version_ptr = (&version) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_GET_API_VERSION, version_ptr);
                println!("IOCTL_LAPUTA_GET_API_VERSION -  version : {:x}", version);

                let addr = 0 as *mut libc::c_void;
                let mmap_ptr = libc::mmap(addr, test_buf_size, 
                    libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0);
                assert_ne!(mmap_ptr, libc::MAP_FAILED);

                test_buf = mmap_ptr as u64;
                test_buf_pfn = test_buf;
                let test_buf_pfn_ptr = (&test_buf_pfn) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_QUERY_PFN, test_buf_pfn_ptr);
                println!("IOCTL_LAPUTA_QUERY_PFN -  test_buf_pfn : {:x}", test_buf_pfn);

                let mut test_buf_ptr = test_buf as *mut i32;
                *test_buf_ptr = 0x73;
                test_buf_ptr = (test_buf + 4) as *mut i32;
                *test_buf_ptr = 0xa001;

                let hugatp = test_buf + 4096 * 4;
                let pte_ptr = (hugatp + 8 * ((test_buf_pfn << 12) >> 30)) as *mut u64;
                *pte_ptr = (((test_buf_pfn << 12) >> 30) << 28) | 0x1f; // 512G 1-level direct mapping
                println!("PTE : {:x}", *pte_ptr);

                // ioctl(fd_ioctl, IOCTL_LAPUTA_REQUEST_DELEG, deleg_info)
                let edeleg = ((1<<10)) | ((1<<20) | (1<<21) | (1<<23)) as libc::c_ulong; // guest page fault(sedeleg)
                let ideleg = (1<<0) as libc::c_ulong;
                let deleg = [edeleg,ideleg];
                let deleg_ptr = (&deleg) as *const u64;
                res = libc::ioctl(fd, IOCTL_LAPUTA_REQUEST_DELEG, deleg_ptr);
                println!("IOCTL_LAPUTA_REQUEST_DELEG : {}", res);

                res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
                println!("IOCTL_LAPUTA_REGISTER_VCPU : {}", res);
            }

            let mut uepc: u64 = 0;
            let mut utval: u64 = 0;
            let mut ucause: u64 = 0;

            let ptr = &vcpu.vcpu_ctx as *const VcpuCtx;
            let ptr_u64 = ptr as u64;
            println!("the ptr is {:x}", ptr_u64);
            let mut ret: i32 = 0;

            vcpu.vcpu_ctx.host_ctx.hyp_regs.uepc = (test_buf_pfn << 12) as u64;
            vcpu.vcpu_ctx.host_ctx.hyp_regs.hugatp = (test_buf_pfn + 2) | (8 << 60);

            while ret == 0 {
                unsafe {
                    set_hugatp(vcpu.vcpu_ctx.host_ctx.hyp_regs.hugatp);
                    println!("HUGATP : {:x}", vcpu.vcpu_ctx.host_ctx.hyp_regs.hugatp);

                    //hustatus.SPP=1 .SPVP=1 uret to VS mode
                    vcpu.vcpu_ctx.host_ctx.hyp_regs.hustatus = ((1 << 8) | (1 << 7)) as u64;

                    set_utvec();

                    //enter_guest_inline(ptr_u64);
                    enter_guest(ptr_u64);

                    uepc = vcpu.vcpu_ctx.host_ctx.hyp_regs.uepc;
                    utval = vcpu.vcpu_ctx.host_ctx.hyp_regs.utval;
                    ucause = vcpu.vcpu_ctx.host_ctx.hyp_regs.ucause;

                    println!("guest hyp uepc 0x{:x}", uepc);
                    println!("guest hyp utval 0x{:x}", utval);
                    println!("guest hyp ucause 0x{:x}", ucause);

                    if ucause == 20 {
                        vcpu.vcpu_ctx.host_ctx.hyp_regs.hugatp = (test_buf_pfn + 4) | (8 << 60);
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
                println!("IOCTL_LAPUTA_RELEASE_PFN -  test_buf_pfn : {:x}", test_buf_pfn);
            }

            assert_eq!(uepc, test_buf_pfn << 12);
            assert_eq!(utval, 0);
            assert_eq!(ucause, 10);
        }

        #[test]
        fn test_vcpu_ecall_exit() { 
            let vcpu_id = 0;
            let vcpu_num = 1;
            let vm = virtualmachine::VirtualMachine::new(vcpu_num);
            let mut fd = vm.vm_state.lock().unwrap().ioctl_fd;
            let vm_mutex = vm.vm_state;
            let mut vcpu = VirtualCpu::new(vcpu_id, vm_mutex);
            let mut res;
            let version: u64 = 0;
            let mut test_buf: u64 = 0;
            let mut test_buf_pfn: u64 = 0;
            let test_buf_size: usize = 32 << 20;

            println!("---test_vcpu_ecall_exit---");

            unsafe {
                // ioctl
                let version_ptr = (&version) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_GET_API_VERSION, version_ptr);
                println!("IOCTL_LAPUTA_GET_API_VERSION -  version : {:x}", version);

                let addr = 0 as *mut libc::c_void;
                let mmap_ptr = libc::mmap(addr, test_buf_size, 
                    libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0);
                assert_ne!(mmap_ptr, libc::MAP_FAILED);
                
                test_buf = mmap_ptr as u64; // va
                test_buf_pfn = test_buf; // pa.pfn
                let test_buf_pfn_ptr = (&test_buf_pfn) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_QUERY_PFN, test_buf_pfn_ptr);
                println!("IOCTL_LAPUTA_QUERY_PFN -  test_buf_pfn : {:x}", test_buf_pfn);
                
                // set test code
                let start = vcpu_ecall_exit as u64;
                let end = vcpu_ecall_exit_end as u64;
                let code_buf = test_buf + PAGE_TABLE_REGION_SIZE;
                libc::memcpy(code_buf as *mut c_void, vcpu_ecall_exit as *mut c_void, (end - start) as usize);

                // set hugatp
                let hugatp = test_buf;
                let pte_ptr = (hugatp + 8 * (((test_buf_pfn << 12) + PAGE_TABLE_REGION_SIZE) >> 30)) as *mut u64;

                let pte_ptr_value = pte_ptr as u64;
                println!("pte_ptr_value {}", pte_ptr_value);

                *pte_ptr = (((test_buf_pfn << 12) >> 30) << 28) | 0x1f; // 512G 1-level direct mapping
                println!("PTE : {:x}", *pte_ptr);

                // ioctl(fd_ioctl, IOCTL_LAPUTA_REQUEST_DELEG, deleg_info)
                let edeleg = ((1<<10)) | ((1<<20) | (1<<21) | (1<<23)) as libc::c_ulong; // guest page fault(sedeleg)
                let ideleg = (1<<0) as libc::c_ulong;
                let deleg = [edeleg,ideleg];
                let deleg_ptr = (&deleg) as *const u64;
                res = libc::ioctl(fd, IOCTL_LAPUTA_REQUEST_DELEG, deleg_ptr);
                println!("IOCTL_LAPUTA_REQUEST_DELEG : {}", res);

                res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
                println!("IOCTL_LAPUTA_REGISTER_VCPU : {}", res);
            }

            let mut uepc: u64 = 0;
            let mut utval: u64 = 0;
            let mut ucause: u64 = 0;

            let ptr = &vcpu.vcpu_ctx as *const VcpuCtx;
            let ptr_u64 = ptr as u64;
            println!("the ptr is {:x}", ptr_u64);
            let mut ret: i32 = 0;

            vcpu.vcpu_ctx.host_ctx.hyp_regs.uepc = ((test_buf_pfn << 12) + PAGE_TABLE_REGION_SIZE) as u64;
            vcpu.vcpu_ctx.host_ctx.hyp_regs.hugatp = (test_buf_pfn) | (8 << 60);

            unsafe {
                // set hugatp
                set_hugatp(vcpu.vcpu_ctx.host_ctx.hyp_regs.hugatp);
                println!("HUGATP : 0x{:x}", vcpu.vcpu_ctx.host_ctx.hyp_regs.hugatp);

                //hustatus.SPP=1 .SPVP=1 uret to VS mode
                vcpu.vcpu_ctx.host_ctx.hyp_regs.hustatus = ((1 << 8) | (1 << 7)) as u64;

                // set utvec to trap handler
                set_utvec();

                enter_guest(ptr_u64);

                uepc = vcpu.vcpu_ctx.host_ctx.hyp_regs.uepc;
                utval = vcpu.vcpu_ctx.host_ctx.hyp_regs.utval;
                ucause = vcpu.vcpu_ctx.host_ctx.hyp_regs.ucause;

                let a7 = vcpu.vcpu_ctx.guest_ctx.gp_regs.x_reg[17];

                println!("guest hyp uepc 0x{:x}", uepc);
                println!("guest hyp utval 0x{:x}", utval);
                println!("guest hyp ucause 0x{:x}", ucause);
                println!("guest hyp a7 0x{:x}", a7);
            }

            assert_eq!(uepc, ((test_buf_pfn << 12) + PAGE_TABLE_REGION_SIZE) + 2);
            assert_eq!(utval, 0);
            assert_eq!(ucause, 10);
        }

        #[test]
        fn test_vcpu_add_all_gprs() { 
            let vcpu_id = 0;
            let vcpu_num = 1;
            let vm = virtualmachine::VirtualMachine::new(vcpu_num);
            let mut fd = vm.vm_state.lock().unwrap().ioctl_fd;
            let vm_mutex = vm.vm_state;
            let mut vcpu = VirtualCpu::new(vcpu_id, vm_mutex);
            let mut res;
            let version: u64 = 0;
            let mut test_buf: u64 = 0;
            let mut test_buf_pfn: u64 = 0;
            let test_buf_size: usize = 32 << 20;
            let size: u64;

            println!("---test_vcpu_add_all_gprs---");

            unsafe {
                // ioctl
                let version_ptr = (&version) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_GET_API_VERSION, version_ptr);
                println!("IOCTL_LAPUTA_GET_API_VERSION -  version : {:x}", version);

                let addr = 0 as *mut libc::c_void;
                let mmap_ptr = libc::mmap(addr, test_buf_size, 
                    libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0);
                assert_ne!(mmap_ptr, libc::MAP_FAILED);
                
                test_buf = mmap_ptr as u64; // va
                test_buf_pfn = test_buf; // pa.pfn
                let test_buf_pfn_ptr = (&test_buf_pfn) as *const u64;
                libc::ioctl(fd, IOCTL_LAPUTA_QUERY_PFN, test_buf_pfn_ptr);
                println!("IOCTL_LAPUTA_QUERY_PFN -  test_buf_pfn : {:x}", test_buf_pfn);
                
                // set test code
                let start = vcpu_add_all_gprs as u64;
                let end = vcpu_add_all_gprs_end as u64;
                size = end - start;
                let code_buf = test_buf + PAGE_TABLE_REGION_SIZE;
                libc::memcpy(code_buf as *mut c_void, vcpu_add_all_gprs as *mut c_void, size as usize);

                // set hugatp
                let hugatp = test_buf;
                let pte_ptr = (hugatp + 8 * (((test_buf_pfn << 12) + PAGE_TABLE_REGION_SIZE) >> 30)) as *mut u64;

                let pte_ptr_value = pte_ptr as u64;
                println!("pte_ptr_value {}", pte_ptr_value);

                *pte_ptr = (((test_buf_pfn << 12) >> 30) << 28) | 0x1f; // 512G 1-level direct mapping
                println!("PTE : {:x}", *pte_ptr);

                // ioctl(fd_ioctl, IOCTL_LAPUTA_REQUEST_DELEG, deleg_info)
                let edeleg = ((1<<10)) | ((1<<20) | (1<<21) | (1<<23)) as libc::c_ulong; // guest page fault(sedeleg)
                let ideleg = (1<<0) as libc::c_ulong;
                let deleg = [edeleg,ideleg];
                let deleg_ptr = (&deleg) as *const u64;
                res = libc::ioctl(fd, IOCTL_LAPUTA_REQUEST_DELEG, deleg_ptr);
                println!("IOCTL_LAPUTA_REQUEST_DELEG : {}", res);

                res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
                println!("IOCTL_LAPUTA_REGISTER_VCPU : {}", res);
            }

            let mut uepc: u64 = 0;
            let mut utval: u64 = 0;
            let mut ucause: u64 = 0;

            let ptr = &vcpu.vcpu_ctx as *const VcpuCtx;
            let ptr_u64 = ptr as u64;
            println!("the ptr is {:x}", ptr_u64);
            let mut ret: i32 = 0;

            vcpu.vcpu_ctx.host_ctx.hyp_regs.uepc = ((test_buf_pfn << 12) + PAGE_TABLE_REGION_SIZE) as u64;
            vcpu.vcpu_ctx.host_ctx.hyp_regs.hugatp = (test_buf_pfn) | (8 << 60);

            let mut sum = 0; 
            for i in 0..vcpu.vcpu_ctx.guest_ctx.gp_regs.x_reg.len() {
                vcpu.vcpu_ctx.guest_ctx.gp_regs.x_reg[i] = i as u64;
                sum += i as u64;
            }
            println!("sum {}", sum);

            unsafe {
                // set hugatp
                set_hugatp(vcpu.vcpu_ctx.host_ctx.hyp_regs.hugatp);
                println!("HUGATP : 0x{:x}", vcpu.vcpu_ctx.host_ctx.hyp_regs.hugatp);

                //hustatus.SPP=1 .SPVP=1 uret to VS mode
                vcpu.vcpu_ctx.host_ctx.hyp_regs.hustatus = ((1 << 8) | (1 << 7)) as u64;

                // set utvec to trap handler
                set_utvec();

                enter_guest(ptr_u64);

                uepc = vcpu.vcpu_ctx.host_ctx.hyp_regs.uepc;
                utval = vcpu.vcpu_ctx.host_ctx.hyp_regs.utval;
                ucause = vcpu.vcpu_ctx.host_ctx.hyp_regs.ucause;

                let a7 = vcpu.vcpu_ctx.guest_ctx.gp_regs.x_reg[17];

                println!("guest hyp uepc 0x{:x}", uepc);
                println!("guest hyp utval 0x{:x}", utval);
                println!("guest hyp ucause 0x{:x}", ucause);
                println!("guest hyp a7 0x{:x}", a7);
            }

            assert_eq!(sum, vcpu.vcpu_ctx.guest_ctx.gp_regs.x_reg[10]);
            assert_eq!(uepc, ((test_buf_pfn << 12) + PAGE_TABLE_REGION_SIZE) + size - 4);
            assert_eq!(utval, 0);
            assert_eq!(ucause, 10);
        }

        // Check the correctness of vcpu new()
        #[test]
        fn test_vcpu_new() { 
            let vcpu_id = 20;
            let vm = virtualmachine::VirtualMachine::new(1);
            let vm_mutex = vm.vm_state;
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex);

            assert_eq!(vcpu.vcpu_id, vcpu_id);
        }

        // Check the init state of the vcpu  
        #[test]
        fn test_vcpu_ctx_init() { 
            let vcpu_id = 1;
            let vm = virtualmachine::VirtualMachine::new(1);
            let vm_mutex = vm.vm_state;
            let vcpu = VirtualCpu::new(vcpu_id, vm_mutex);

            let tmp = vcpu.vcpu_ctx.host_ctx.gp_regs.x_reg[10];
            assert_eq!(tmp, 0);

            let tmp = vcpu.vcpu_ctx.host_ctx.hyp_regs.hutinst;
            assert_eq!(tmp, 0);

            let tmp = vcpu.vcpu_ctx.guest_ctx.gp_regs.x_reg[10];
            assert_eq!(tmp, 0);

            let tmp = vcpu.vcpu_ctx.guest_ctx.hyp_regs.hutinst;
            assert_eq!(tmp, 0);

            let tmp = vcpu.vcpu_ctx.guest_ctx.sys_regs.huvsatp;
            assert_eq!(tmp, 0);
        }



        // Check the rw permission of vcpu ctx 
        #[test]
        fn test_vcpu_set_ctx() {  
            let vcpu_id = 1;
            let vm = virtualmachine::VirtualMachine::new(1);
            let vm_mutex = vm.vm_state;
            let mut vcpu = VirtualCpu::new(vcpu_id, vm_mutex);

            // guest ctx
            vcpu.vcpu_ctx.guest_ctx.gp_regs.x_reg[10] = 17;
            let tmp = vcpu.vcpu_ctx.guest_ctx.gp_regs.x_reg[10];
            assert_eq!(tmp, 17);

            vcpu.vcpu_ctx.guest_ctx.sys_regs.huvsatp = 17;
            let tmp = vcpu.vcpu_ctx.guest_ctx.sys_regs.huvsatp;
            assert_eq!(tmp, 17);

            vcpu.vcpu_ctx.guest_ctx.hyp_regs.hutinst = 17;
            let tmp = vcpu.vcpu_ctx.guest_ctx.hyp_regs.hutinst;
            assert_eq!(tmp, 17);

            // host ctx
            vcpu.vcpu_ctx.host_ctx.gp_regs.x_reg[10] = 17;
            let tmp = vcpu.vcpu_ctx.host_ctx.gp_regs.x_reg[10];
            assert_eq!(tmp, 17);

            vcpu.vcpu_ctx.host_ctx.hyp_regs.hutinst = 17;
            let tmp = vcpu.vcpu_ctx.host_ctx.hyp_regs.hutinst;
            assert_eq!(tmp, 17);
        }

        // Check the Arc<Mutex<>> data access.
        #[test]
        fn test_vcpu_run() {
            let vcpu_num = 4;
            let mut vm = virtualmachine::VirtualMachine::new(vcpu_num);
            let mut vcpu_handle: Vec<thread::JoinHandle<()>> = Vec::new();
            let mut handle: thread::JoinHandle<()>;
            let mut vcpu_mutex;

            for i in &mut vm.vcpus {
                // Get a clone for the closure
                vcpu_mutex = i.clone();

                // Start vcpu threads!
                handle = thread::spawn(move || {
                    // TODO: thread_vcpu_run
                    vcpu_mutex.lock().unwrap().test_change_guest_ctx();
                });

                vcpu_handle.push(handle);
            }

            // All the vcpu thread finish
            for i in vcpu_handle {
                i.join().unwrap();
            }

            // Check the guest contexxt
            let mut gpreg;
            let mut sysreg;
            let mut hypreg;
            for i in &vm.vcpus {
                gpreg = i.lock().unwrap().vcpu_ctx.guest_ctx.gp_regs.x_reg[10];
                sysreg = i.lock().unwrap().vcpu_ctx.guest_ctx.sys_regs.huvsscratch;
                hypreg = i.lock().unwrap().vcpu_ctx.guest_ctx.hyp_regs.hutinst;
                assert_eq!(gpreg, 10);
                assert_eq!(sysreg, 11);
                assert_eq!(hypreg, 12);
            }

            /* 
             * The result should be 400 to prove the main thread can get the 
             * correct value.
             */
            let result = vm.vm_state.lock().unwrap().vm_id;
            assert_eq!(result, vcpu_num * 100);
        }

        /* // Check the correctness of vcpu_exit_handler
        #[test]
            fn test_vcpu_exit_handler() { 
            let vcpu_id = 20;
            let vm = virtualmachine::VirtualMachine::new(1);
            let vm_mutex = vm.vm_state;
            let mut vcpu = VirtualCpu::new(vcpu_id, vm_mutex);
            let mut res;
 
            vcpu.vcpu_ctx.host_ctx.hyp_regs.ucause = EXC_IRQ_MASK | 0x1;
            res = vcpu.handle_vcpu_exit();
            assert_eq!(res, 1);
 
            vcpu.vcpu_ctx.host_ctx.hyp_regs.ucause = EXC_SUPERVISOR_SYSCALL;
            res = vcpu.handle_vcpu_exit();
            assert_eq!(res, 0xdead);
 
            vcpu.vcpu_ctx.host_ctx.hyp_regs.hutval = 0x8048000;
            vcpu.vcpu_ctx.host_ctx.hyp_regs.utval = 0xf0;
            vcpu.vcpu_ctx.host_ctx.hyp_regs.ucause = EXC_INST_GUEST_PAGE_FAULT;
            res = vcpu.handle_vcpu_exit();
            assert_eq!(res, 0);
 
            vcpu.vcpu_ctx.host_ctx.hyp_regs.hutval = 0x7ff000;
            vcpu.vcpu_ctx.host_ctx.hyp_regs.utval = 0xf;
            vcpu.vcpu_ctx.host_ctx.hyp_regs.ucause = EXC_LOAD_GUEST_PAGE_FAULT;
            res = vcpu.handle_vcpu_exit();
            assert_eq!(res, 0);
 
            vcpu.vcpu_ctx.host_ctx.hyp_regs.utval = 0xa001;
            vcpu.vcpu_ctx.host_ctx.hyp_regs.ucause = EXC_VIRTUAL_INST_FAULT;
            res = vcpu.handle_vcpu_exit();
            assert_eq!(res, 0);
 
            vcpu.vcpu_ctx.host_ctx.hyp_regs.hutval = 0xdead0000;
            vcpu.vcpu_ctx.host_ctx.hyp_regs.utval = 0xfff;
            vcpu.vcpu_ctx.host_ctx.hyp_regs.ucause = EXC_STORE_GUEST_PAGE_FAULT;
            res = vcpu.handle_vcpu_exit();
            assert_eq!(res, 0);
        } */ 
    }
}

