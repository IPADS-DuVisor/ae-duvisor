use crate::vm::virtualmachine;
use crate::irq::virq;
use crate::irq::vtimer;
use crate::vcpu::vcpucontext;
use std::sync::{Arc, Mutex};
use vcpucontext::*;

global_asm!(include_str!("asm_offset.S"));
global_asm!(include_str!("asm_csr.S"));
global_asm!(include_str!("asm_switch.S"));
global_asm!(include_str!("vm_code.S"));
global_asm!(include_str!("save_restore.S"));

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
    pub ioctl_fd: Option<i32>,
    // TODO: irq_pending with shared memory
}

impl VirtualCpu {
    pub fn new(vcpu_id: u32, ioctl_fd: Option<i32>,
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
            ioctl_fd,
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

#[no_mangle]
pub fn vm_code_print() {
    println!("Enter vm");
}

#[no_mangle]
pub unsafe fn vm_code_ecall() {
    llvm_asm!(".align 2
            ecall":::: "volatile");
}

#[allow(unused)]
pub unsafe fn set_hugatp(hugatp: u64) {
    llvm_asm!(".align 2
            mv t0, $0
            CSRW_CSR_HUGATP t0
            " :: "r"(hugatp) :"memory", "x28": "volatile");
}

#[allow(unused)]
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub unsafe fn enter_guest_inline(ctx: u64) {
    llvm_asm!(".align 2
            // a0 point to vcpu_ctx
            mv a0, $0

            // save host gp with a0=ctx, except t0-t6 and zero-x0
            SAVE_HOST_CTX a0

            /* SSTATUS? HUSTATUS got hstatus.SPV & sstatus.SPP */
            RESTORE_GUEST_HYP_HUSTATUS a0, t1
            CSRRW_CSR_HUSTATUS t1, t1
            //CSRR_CSR_HUSTATUS t3
            SAVE_HOST_HYP_HUSTATUS a0, t1

            /* SCOUNTEREN? HUCOUNTEREN should be 64-bit with hcounteren(32-bit) + scounteren(32-bit) */
            RESTORE_GUEST_HYP_HUCOUNTEREN a0, t1
            CSRRW_CSR_HUCOUNTEREN t1, t1
            SAVE_HOST_HYP_HUCOUNTEREN a0, t1

            /* save a0-vcpu-ctx in CSR_USCRATCH & save USCRATCH */ 
            CSRRW_CSR_USCRATCH t3, a0
            SAVE_HOST_HYP_USCRATCH a0, t3

            /* set utvec to catch the trap & save UTVEC */
            la	t4, __vm_exit
            CSRRW_CSR_UTVEC t4, t4
            SAVE_HOST_HYP_UTVEC a0, t4

            /* set UEPC */
            RESTORE_GUEST_HYP_UEPC a0, t0
            CSRRW_CSR_UEPC t0, t0
            SAVE_GUEST_HYP_UEPC a0, t0

            //hufence
            .word 0xE2000073

            // restore guest GP except A0 & X0
            RESTORE_GUEST_CTX a0

            /* huret */
            uret

            .align 2
            __vm_exit:

            /* save guest-a0 in sscratch & get host-a0 */
            CSRRW_CSR_USCRATCH a0, a0
            SAVE_HOST_GP_X0 a0, a0

            /* save guest gp except A0 & X0 */
            SAVE_GUEST_CTX a0

            /* save guest A0 with USCRATCH */
            CSRR_CSR_USCRATCH t1
            SAVE_GUEST_GP_X10 a0, t1

            /* save guest UEPC */
            CSRR_CSR_UEPC t0
            SAVE_GUEST_HYP_UEPC a0, t0

            /* restore host utvec */
            RESTORE_HOST_HYP_UTVEC a0, t1
            CSRW_CSR_UTVEC t1

            /* restore host uscratch */
            RESTORE_HOST_HYP_USCRATCH a0, t2
            CSRW_CSR_USCRATCH t2

            /* restore host HUCOUNTEREN */
            RESTORE_HOST_HYP_HUCOUNTEREN a0, t3
            CSRRW_CSR_HUCOUNTEREN t3, t3
            SAVE_GUEST_HYP_HUCOUNTEREN a0, t3

            /* restore host HUSTATUS */
            RESTORE_HOST_HYP_HUSTATUS a0, t4
            CSRRW_CSR_HUSTATUS t4, t4
            SAVE_GUEST_HYP_HUSTATUS a0, t4

            /* restore host gp with a0=ctx, except t0-t6 and zero-x0 */
            RESTORE_HOST_CTX a0
            " :: "r"(ctx) :"memory", "x5", "x6", "x7", "x10", "x11", "x28", "x29", "x30", "x31": "volatile");

            // Save the key reg for vm exit handler
            // UCAUSE / UTVAL
            llvm_asm!(".align 2
                mv a0, $0
                CSRR_CSR_UCAUSE t3
                SAVE_GUEST_HYP_UCAUSE a0, t3

                CSRR_CSR_UTVAL t3
                SAVE_GUEST_HYP_UTVAL a0, t3
            " :: "r"(ctx) :"memory", "x10", "x28": "volatile");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::ffi::CString;

    #[test]
    fn test_first_uret() { 
        let mut vcpuctx = VcpuCtx::new();
        let fd;
        let mut res;
        let file_path = CString::new("/dev/laputa_dev").unwrap();

        let tmp_buf_pfn: u64 = 0;
        unsafe { 
            fd = libc::open(file_path.as_ptr(), libc::O_RDWR); 

            // ioctl(fd_ioctl, IOCTL_LAPUTA_GET_API_VERSION, &tmp_buf_pfn) // 0x80086b01
            let tmp_buf_pfn_ptr = (&tmp_buf_pfn) as *const u64;
            libc::ioctl(fd, 0x80086b01, tmp_buf_pfn_ptr);
            println!("tmp_buf_pfn : {:x}", tmp_buf_pfn);

            // ioctl(fd_ioctl, IOCTL_LAPUTA_REQUEST_DELEG, deleg_info)
            let edeleg = ((1<<20) | (1<<21) | (1<<23)) as libc::c_ulong; // guest page fault(sedeleg)
            let ideleg = (1<<0) as libc::c_ulong;
            let deleg = [edeleg,ideleg];
            let deleg_ptr = (&deleg) as *const u64;
            res = libc::ioctl(fd, 1074817795, deleg_ptr);
            println!("ioctl 1074817795 : {}", res);

            res = libc::ioctl(fd, 0x6b04);
            println!("ioctl 0x6b04 : {}", res);
        }

        let uepc: u64;
        let utval: u64;
        let ucause: u64;

        let ptr = &vcpuctx as *const VcpuCtx;
        println!("the ptr is {}", ptr as u64);
        let ptr_u64 = ptr as u64;
        unsafe {
            let pt_hpa = tmp_buf_pfn | (1 << 63);
            //vcpuctx.guest_ctx.hyp_regs.hugatp = pt_hpa;
            set_hugatp(pt_hpa);
            println!("HUGATP : {:x}", pt_hpa);
            
            vcpuctx.guest_ctx.hyp_regs.uepc = vm_code as u64;

            //hustatus.SPP=1 .SPVP=1 uret to VS mode
            vcpuctx.guest_ctx.hyp_regs.hustatus = ((1 << 8) | (1 << 7)) as u64;
            //VirtualCpu::open_hu_extension(ioctl_fd);

            enter_guest_inline(ptr_u64);

            uepc = vcpuctx.guest_ctx.hyp_regs.uepc;
            utval = vcpuctx.guest_ctx.hyp_regs.utval;
            ucause = vcpuctx.guest_ctx.hyp_regs.ucause;

            println!("guest hyp uepc 0x{:x}", uepc);
            println!("guest hyp utval 0x{:x}", utval);
            println!("guest hyp ucause 0x{:x}", ucause);
            
            res = libc::ioctl(fd, 0x6b05);
            println!("ioctl 0x6b05 {}", res);
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
        let vcpu = VirtualCpu::new(vcpu_id, None, vm_mutex);

        assert_eq!(vcpu.vcpu_id, vcpu_id);
    }

    // Check the init state of the vcpu  
    #[test]
    fn test_vcpu_ctx_init() { 
        let vcpu_id = 1;
        let vm_state = virtualmachine::VmSharedState::new();
        let vm_mutex = Arc::new(Mutex::new(vm_state));
        let vcpu = VirtualCpu::new(vcpu_id, None, vm_mutex);

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
        let mut vcpu = VirtualCpu::new(vcpu_id, None, vm_mutex);

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
