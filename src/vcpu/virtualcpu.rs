use crate::vm::virtualmachine;
use crate::irq::virq;
use crate::irq::vtimer;
use crate::vcpu::vcpucontext;
use std::sync::{Arc, Mutex};
use vcpucontext::*;

global_asm!(include_str!("vm_code.S"));

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
}

impl VirtualCpu {
    pub fn new(vcpu_id: u32,
            vm_mutex_ptr: Arc<Mutex<virtualmachine::VmSharedState>>) -> Self {
        let vcpu_ctx = VcpuCtx::new();
        let virq = virq::VirtualInterrupt::new();
        let vtimer = vtimer::VirtualTimer::new(0, 0);

        Self {
            vcpu_id,
            vm: vm_mutex_ptr,
            vcpu_ctx,
            virq,
            vtimer,
        }
    }

    // For test case: test_vm_run
    fn test_change_guest_ctx(&mut self) -> u32 {
        // Change guest context
        self.vcpu_ctx.guest_ctx.gp_regs.x_reg[10] += 10;
        self.vcpu_ctx.guest_ctx.sys_regs.huvsscratch += 11;
        self.vcpu_ctx.guest_ctx.hyp_regs.hutinst += 12;

        // Increse vm_id in vm_state
        self.vm.lock().unwrap().vm_id += 100;

        0
    }

    pub fn thread_vcpu_run(&mut self) -> u32 {
        self.test_change_guest_ctx();

        0
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
    fn test_first_uret() { 
        let mut vcpuctx = VcpuCtx::new();
        let fd;
        let mut res;
        let file_path = CString::new("/dev/laputa_dev").unwrap();

        let tmp_buf_pfn: u64 = 0;
        unsafe { 
            fd = libc::open(file_path.as_ptr(), libc::O_RDWR); 

            // ioctl(fd_ioctl, IOCTL_LAPUTA_GET_API_VERSION, &tmp_buf_pfn)
            let tmp_buf_pfn_ptr = (&tmp_buf_pfn) as *const u64;
            libc::ioctl(fd, IOCTL_LAPUTA_GET_API_VERSION, tmp_buf_pfn_ptr);
            println!("IOCTL_LAPUTA_GET_API_VERSION -  tmp_buf_pfn : {:x}", tmp_buf_pfn);

            // ioctl(fd_ioctl, IOCTL_LAPUTA_REQUEST_DELEG, deleg_info)
            // delegate guest page fault
            let edeleg = ((1 << INST_GUEST_PAGE_FAULT) | (1 << LOAD_GUEST_ACCESS_FAULT) 
                | (1 << STORE_GUEST_AMO_ACCESS_FAULT)) as libc::c_ulong;
            let ideleg = (1 << S_SOFT) as libc::c_ulong;
            let deleg = [edeleg,ideleg];
            let deleg_ptr = (&deleg) as *const u64;
            res = libc::ioctl(fd, IOCTL_LAPUTA_REQUEST_DELEG, deleg_ptr);
            println!("IOCTL_LAPUTA_REQUEST_DELEG : {}", res);

            res = libc::ioctl(fd, IOCTL_LAPUTA_REGISTER_VCPU);
            println!("IOCTL_LAPUTA_REGISTER_VCPU : {}", res);
        }

        let uepc: u64;
        let utval: u64;
        let ucause: u64;

        let ptr = &vcpuctx as *const VcpuCtx;
        println!("the ptr is {}", ptr as u64);
        let ptr_u64 = ptr as u64;
        unsafe {
            let pt_hpa = tmp_buf_pfn | (1 << 63);

            set_hugatp(pt_hpa);
            println!("HUGATP : {:x}", pt_hpa);

            set_utvec();
            
            vcpuctx.guest_ctx.hyp_regs.uepc = vm_code as u64;

            //hustatus.SPP=1 .SPVP=1 uret to VS mode
            vcpuctx.guest_ctx.hyp_regs.hustatus = ((1 << HUSTATUS_SPV_SHIFT) 
                | (1 << HUSTATUS_SPVP_SHIFT)) as u64;

            enter_guest(ptr_u64);

            uepc = vcpuctx.guest_ctx.hyp_regs.uepc;
            utval = vcpuctx.guest_ctx.hyp_regs.utval;
            ucause = vcpuctx.guest_ctx.hyp_regs.ucause;

            println!("guest hyp uepc 0x{:x}", uepc);
            println!("guest hyp utval 0x{:x}", utval);
            println!("guest hyp ucause 0x{:x}", ucause);
            
            res = libc::ioctl(fd, IOCTL_LAPUTA_UNREGISTER_VCPU);
            println!("IOCTL_LAPUTA_UNREGISTER_VCPU : {}", res);
        }

        // vm should trap(20) at vm_code
        assert_eq!(uepc, utval);
        assert_eq!(20, ucause);
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
                vcpu_mutex.lock().unwrap().thread_vcpu_run();
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
}
