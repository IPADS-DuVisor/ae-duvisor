use crate::vm::virtualmachine;
use crate::irq::virq;
use crate::irq::vtimer;
use crate::vcpu::vcpucontext;
use std::sync::{Arc, Mutex};
use vcpucontext::*;

global_asm!(include_str!("vm_code.S"));

mod vm_exception_constants {
    pub const EXC_SUPERVISOR_SYSCALL: u64 = 10;
    pub const EXC_INST_GUEST_PAGE_FAULT: u64 = 20;
    pub const EXC_LOAD_GUEST_PAGE_FAULT: u64 = 21;
    pub const EXC_VIRTUAL_INST_FAULT: u64 = 22;
    pub const EXC_STORE_GUEST_PAGE_FAULT: u64 = 23;
    pub const EXC_IRQ_MASK: u64 = 1 << 63;
}
pub use vm_exception_constants::*;

mod errno_constants {
    pub const ENOPERMIT: i32 = -1;
    pub const ENOMAPPING: i32 = -2;
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
    
    fn virtual_inst_fault(&mut self) -> i32 {
        let ret = 0;
        let utval = self.vcpu_ctx.host_ctx.hyp_regs.utval;
        println!("virtual_inst_fault: insn = {:x}", utval);
        
        ret
    }

    fn stage2_page_fault(&mut self) -> i32 {
        let hutval = self.vcpu_ctx.host_ctx.hyp_regs.hutval;
        let utval = self.vcpu_ctx.host_ctx.hyp_regs.utval;
        let fault_addr = (hutval << 2) | (utval & 0x3);
        println!("gstage_page_fault: fault_addr = {:x}", fault_addr);

        let mut ret = 0;
        // map_query
        match ret {
            ENOPERMIT => {
                self.exit_reason = ExitReason::ExitEaccess;
                eprintln!("Query return ENOPERMIT: {}", ret);
                ret = ENOPERMIT
            }
            ENOMAPPING => {
                println!("Query return ENOMAPPING: {}", ret);
                // find gpa region by fault_addr
                // map new region to VM if the region exists
                // handle MMIO otherwise
            }
            _ => {
                self.exit_reason = ExitReason::ExitUnknown;
                eprintln!("Invalid query result: {}", ret);
            }
        }

        ret
    }

    fn supervisor_ecall(&mut self) -> i32 {
        let mut ret = 0;
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
        //let gsmmu = &self.vm.lock().unwrap().gsmmu;
        self.vcpu_ctx.host_ctx.hyp_regs.hugatp = 
            (self.vm.lock().unwrap().gsmmu.page_table.region.base_address >> 12) | 
            (8 << 60);
        unsafe {
            set_hugatp(self.vcpu_ctx.host_ctx.hyp_regs.hugatp);
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

    #[test]
    fn test_stage2_page_fault() { 
        let vcpu_id = 0;
        let vm_state = virtualmachine::VmSharedState::new();
        let vm_mutex = Arc::new(Mutex::new(vm_state));
        let mut vcpu = VirtualCpu::new(vcpu_id, vm_mutex);
        let fd;
        let mut res;
        let file_path = CString::new("/dev/laputa_dev").unwrap();

        let version: u64 = 0;
        let mut test_buf: u64 = 0;
        let mut test_buf_pfn: u64 = 0;
        let test_buf_size: usize = 32 << 20;
        unsafe { 
            fd = libc::open(file_path.as_ptr(), libc::O_RDWR); 

            // ioctl(fd_ioctl, IOCTL_LAPUTA_GET_API_VERSION, &tmp_buf_pfn) // 0x80086b01
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

    // Check the correctness of vcpu new()
    #[test]
    fn test_vcpu_new() { 
        let vcpu_id = 20;
        let vm_state = virtualmachine::VmSharedState::new();
        let vm_mutex = Arc::new(Mutex::new(vm_state));
        let vcpu = VirtualCpu::new(vcpu_id, vm_mutex);

        assert_eq!(vcpu.vcpu_id, vcpu_id);
    }

    // Check the init state of the vcpu  
    #[test]
    fn test_vcpu_ctx_init() { 
        let vcpu_id = 1;
        let vm_state = virtualmachine::VmSharedState::new();
        let vm_mutex = Arc::new(Mutex::new(vm_state));
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
        let vm_state = virtualmachine::VmSharedState::new();
        let vm_mutex = Arc::new(Mutex::new(vm_state));
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
    
    // Check the correctness of vcpu_exit_handler
    #[test]
    fn test_vcpu_exit_handler() { 
        let vcpu_id = 20;
        let vm_state = virtualmachine::VmSharedState::new();
        let vm_mutex = Arc::new(Mutex::new(vm_state));
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
    }
}
