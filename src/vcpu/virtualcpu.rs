use crate::vm::virtualmachine;
use crate::irq::virq;
use crate::irq::vtimer;
use crate::vcpu::context;
use std::sync::{Arc, Mutex};
use context::*;

global_asm!(include_str!("../asm_offset.S"));
global_asm!(include_str!("../asm_csr.S"));

pub struct VirtualCpu {
    pub vcpu_id: u32,
    pub vm: Arc<Mutex<virtualmachine::VmSharedState>>,
    pub vcpu_ctx: VcpuCtx,
    pub virq: virq::VirtualInterrupt,
    pub vtimer: vtimer::VirtualTimer,
    // TODO: irq_pending with shared memory
}

impl VirtualCpu {
    pub fn new(vcpu_id: u32, vm_mutex_ptr: Arc<Mutex<virtualmachine::VmSharedState>>) -> Self {
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
        self.vcpu_ctx.guest_ctx.sys_regs.vsscratch += 11;
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

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub unsafe fn enter_guest_inline(ctx: u64) {
    llvm_asm!(".align 2
            mv a0, $0
            // save host gp with a0=ctx, except t0-t6 and zero-x0
            //SAVE_HOST_GP_X0 a0, x0 // zero
            SAVE_HOST_GP_X1 a0, x1
            SAVE_HOST_GP_X2 a0, x2
            SAVE_HOST_GP_X3 a0, x3
            SAVE_HOST_GP_X4 a0, x4
            SAVE_HOST_GP_X5 a0, x5 // t0
            SAVE_HOST_GP_X6 a0, x6 // t1
            SAVE_HOST_GP_X7 a0, x7 // t2
            SAVE_HOST_GP_X8 a0, x8
            SAVE_HOST_GP_X9 a0, x9
            SAVE_HOST_GP_X10 a0, x10
            SAVE_HOST_GP_X11 a0, x11
            SAVE_HOST_GP_X12 a0, x12
            SAVE_HOST_GP_X13 a0, x13
            SAVE_HOST_GP_X14 a0, x14
            SAVE_HOST_GP_X15 a0, x15
            SAVE_HOST_GP_X16 a0, x16
            SAVE_HOST_GP_X17 a0, x17
            SAVE_HOST_GP_X18 a0, x18
            SAVE_HOST_GP_X19 a0, x19
            SAVE_HOST_GP_X20 a0, x20
            SAVE_HOST_GP_X21 a0, x21
            SAVE_HOST_GP_X22 a0, x22
            SAVE_HOST_GP_X23 a0, x23
            SAVE_HOST_GP_X24 a0, x24
            SAVE_HOST_GP_X25 a0, x25
            SAVE_HOST_GP_X26 a0, x26
            SAVE_HOST_GP_X27 a0, x27
            SAVE_HOST_GP_X28 a0, x28 // t3
            SAVE_HOST_GP_X29 a0, x29 // t4
            SAVE_HOST_GP_X30 a0, x30 // t5
            SAVE_HOST_GP_X31 a0, x31 // t6
//
//            /* a0 = ctx */
//
//            /* SSTATUS? HUSTATUS got hstatus.SPV & sstatus.SPP*/
//            RESTORE_GUEST_HYP_HUSTATUS a0, t1
//            CSRRW_CSR_HUSTATUS t1, t1
//            li t1, 17
//            SAVE_HOST_HYP_HUSTATUS a0, t1
//
//            /* SCOUNTEREN? HUCOUNTEREN should be 64-bit with hcounteren(32-bit) + scounteren(32-bit)*/
//            RESTORE_GUEST_HYP_HUCOUNTEREN a0, t1
//            CSRRW_CSR_HUCOUNTEREN t1, t1
//            SAVE_HOST_HYP_HUCOUNTEREN a0, t1
//
//            /* save a0-vcpu-ctx in CSR_USCRATCH & save USCRATCH */
//            CSRRW_CSR_USCRATCH t3, a0
//            SAVE_HOST_HYP_USCRATCH a0, t3
//
//            /* set utvec to catch the trap & save UTVEC*/
//            la	t4, __vm_exit
//            CSRRW_CSR_UTVEC t4, t4
//            SAVE_HOST_HYP_UTVEC a0, t4
//
//            /* set UEPC */
//            RESTORE_GUEST_HYP_UEPC a0, t0
//            CSRW_CSR_UEPC t0
//
//            // restore guest GP except A0 & X0
//            RESTORE_GUEST_GP_X1 a0, x1
//            RESTORE_GUEST_GP_X2 a0, x2
//            RESTORE_GUEST_GP_X3 a0, x3
//            RESTORE_GUEST_GP_X4 a0, x4
//            RESTORE_GUEST_GP_X5 a0, x5
//            RESTORE_GUEST_GP_X6 a0, x6
//            RESTORE_GUEST_GP_X7 a0, x7
//            RESTORE_GUEST_GP_X8 a0, x8
//            RESTORE_GUEST_GP_X9 a0, x9
//            //RESTORE_GUEST_GP_X10 a0, x10 // a0
//            RESTORE_GUEST_GP_X11 a0, x11
//            RESTORE_GUEST_GP_X12 a0, x12
//            RESTORE_GUEST_GP_X13 a0, x13
//            RESTORE_GUEST_GP_X14 a0, x14
//            RESTORE_GUEST_GP_X15 a0, x15
//            RESTORE_GUEST_GP_X16 a0, x16
//            RESTORE_GUEST_GP_X17 a0, x17
//            RESTORE_GUEST_GP_X18 a0, x18
//            RESTORE_GUEST_GP_X19 a0, x19
//            RESTORE_GUEST_GP_X20 a0, x20
//            RESTORE_GUEST_GP_X21 a0, x21
//            RESTORE_GUEST_GP_X22 a0, x22
//            RESTORE_GUEST_GP_X23 a0, x23
//            RESTORE_GUEST_GP_X24 a0, x24
//            RESTORE_GUEST_GP_X25 a0, x25
//            RESTORE_GUEST_GP_X26 a0, x26
//            RESTORE_GUEST_GP_X27 a0, x27
//            RESTORE_GUEST_GP_X28 a0, x28
//            RESTORE_GUEST_GP_X29 a0, x29
//            RESTORE_GUEST_GP_X30 a0, x30
//            RESTORE_GUEST_GP_X31 a0, x31

//            /* restore guest A0 */
//            RESTORE_GUEST_GP_X10 a0, x10
//
//            /* uret */
//            //huret
//
            .align 2
            __vm_exit:
//            /* save guest-a0 in sscratch & get host-a0 */
//            CSRRW_CSR_USCRATCH a0, a0
//
//            /* a0 = ctx */

//            /* save guest gp except A0 & X0 */
//            SAVE_GUEST_GP_X1 a0, x1
//            SAVE_GUEST_GP_X2 a0, x2
//            SAVE_GUEST_GP_X3 a0, x3
//            SAVE_GUEST_GP_X4 a0, x4
//            SAVE_GUEST_GP_X5 a0, x5
//            SAVE_GUEST_GP_X6 a0, x6
//            SAVE_GUEST_GP_X7 a0, x7
//            SAVE_GUEST_GP_X8 a0, x8
//            SAVE_GUEST_GP_X9 a0, x9
//            SAVE_GUEST_GP_X11 a0, x11
//            SAVE_GUEST_GP_X12 a0, x12
//            SAVE_GUEST_GP_X13 a0, x13
//            SAVE_GUEST_GP_X14 a0, x14
//            SAVE_GUEST_GP_X15 a0, x15
//            SAVE_GUEST_GP_X16 a0, x16
//            SAVE_GUEST_GP_X17 a0, x17
//            SAVE_GUEST_GP_X18 a0, x18
//            SAVE_GUEST_GP_X19 a0, x19
//            SAVE_GUEST_GP_X20 a0, x20
//            SAVE_GUEST_GP_X21 a0, x21
//            SAVE_GUEST_GP_X22 a0, x22
//            SAVE_GUEST_GP_X23 a0, x23
//            SAVE_GUEST_GP_X24 a0, x24
//            SAVE_GUEST_GP_X25 a0, x25
//            SAVE_GUEST_GP_X26 a0, x26
//            SAVE_GUEST_GP_X27 a0, x27
//            SAVE_GUEST_GP_X28 a0, x28
//            SAVE_GUEST_GP_X29 a0, x29
//            SAVE_GUEST_GP_X30 a0, x30
//            SAVE_GUEST_GP_X31 a0, x31

//            /* save guest A0 with USCRATCH */
//            CSRR_CSR_USCRATCH t1
//            SAVE_GUEST_GP_X10 a0, t1
//
//            /* save guest UEPC */
//            CSRR_CSR_UEPC t0
//            SAVE_GUEST_HYP_UEPC a0, t0
//
//            /* restore host utvec */
//            RESTORE_HOST_HYP_UTVEC a0, t1
//            CSRW_CSR_UTVEC t1
//
//            /* restore host uscratch */
//            RESTORE_HOST_HYP_USCRATCH a0, t2
//            CSRW_CSR_USCRATCH t2
//
//            /* restore host HUCOUNTEREN */
//            RESTORE_HOST_HYP_HUCOUNTEREN a0, t3
//            CSRRW_CSR_HUCOUNTEREN t3, t3
//            SAVE_GUEST_HYP_HUCOUNTEREN a0, t3
//
//            /* restore host HUSTATUS */
//            RESTORE_HOST_HYP_HUSTATUS a0, t4
//            CSRRW_CSR_HUSTATUS t4, t4
//            SAVE_GUEST_HYP_HUSTATUS a0, t4

//            /* restore host gp with a0=ctx, except t0-t6 and zero-x0 */
//            RESTORE_HOST_GP_X1 a0, x1
//            RESTORE_HOST_GP_X2 a0, x2
//            RESTORE_HOST_GP_X3 a0, x3
//            RESTORE_HOST_GP_X4 a0, x4
//            RESTORE_HOST_GP_X5 a0, x5
//            RESTORE_HOST_GP_X6 a0, x6
//            RESTORE_HOST_GP_X7 a0, x7
//            RESTORE_HOST_GP_X8 a0, x8
//            RESTORE_HOST_GP_X9 a0, x9
//            RESTORE_HOST_GP_X10 a0, x10
//            RESTORE_HOST_GP_X11 a0, x11
//            RESTORE_HOST_GP_X12 a0, x12
//            RESTORE_HOST_GP_X13 a0, x13
//            RESTORE_HOST_GP_X14 a0, x14
//            RESTORE_HOST_GP_X15 a0, x15
//            RESTORE_HOST_GP_X16 a0, x16
//            RESTORE_HOST_GP_X17 a0, x17
//            RESTORE_HOST_GP_X18 a0, x18
//            RESTORE_HOST_GP_X19 a0, x19
//            RESTORE_HOST_GP_X20 a0, x20
//            RESTORE_HOST_GP_X21 a0, x21
//            RESTORE_HOST_GP_X22 a0, x22
//            RESTORE_HOST_GP_X23 a0, x23
//            RESTORE_HOST_GP_X24 a0, x24
//            RESTORE_HOST_GP_X25 a0, x25
//            RESTORE_HOST_GP_X26 a0, x26
//            RESTORE_HOST_GP_X27 a0, x27
//            RESTORE_HOST_GP_X28 a0, x28
//            RESTORE_HOST_GP_X29 a0, x29
//            RESTORE_HOST_GP_X30 a0, x30
//            RESTORE_HOST_GP_X31 a0, x31
            " :: "r"(ctx) :"memory", "x5", "x6", "x7", "x10", "x11", "x28", "x29", "x30", "x31": "volatile");
            //"memory", "x5", "x6", "x7", "x10", "x28", "x29", "x30", "x31"
            //"memory", "a0", "t0", "t1", "t2", "t3", "t4", "t5", "t6"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_struct_ptr() { 
        let mut vcpuctx = VcpuCtx::new();

        let ptr = &vcpuctx as *const VcpuCtx;
        println!("the ptr is {}", ptr as u64);
        let ptr_u64 = ptr as u64;
        unsafe {
            //VirtualCpu::open_hu_extension(ioctl_fd);

            enter_guest_inline(ptr_u64);
            println!("the data 0 {}", *(ptr_u64 as *mut u64));
            println!("the data 8 {}", *((ptr_u64 + 8) as *mut u64));
            println!("the data 16 {}", *((ptr_u64 + 16) as *mut u64));
            println!("the data 24 {}", *((ptr_u64 + 24) as *mut u64));
            println!("the data 32 {}", *((ptr_u64 + 32) as *mut u64));
            println!("the data 40 {}", *((ptr_u64 + 40) as *mut u64));
            println!("the data 48 {}", *((ptr_u64 + 48) as *mut u64));
            println!("the data 56 {}", *((ptr_u64 + 56) as *mut u64));
            println!("the data 64 {}", *((ptr_u64 + 64) as *mut u64));
            println!("the data 72 {}", *((ptr_u64 + 72) as *mut u64));
            println!("the data 80 {}", *((ptr_u64 + 80) as *mut u64));
            println!("the data 88 {}", *((ptr_u64 + 88) as *mut u64));
            println!("the data 96 {}", *((ptr_u64 + 96) as *mut u64));
            println!("the data 104 {}", *((ptr_u64 + 104) as *mut u64));
            println!("the data 112 {}", *((ptr_u64 + 112) as *mut u64));
            println!("the data 120 {}", *((ptr_u64 + 120) as *mut u64));
            println!("the data 128 {}", *((ptr_u64 + 128) as *mut u64));
            println!("the data 136 {}", *((ptr_u64 + 136) as *mut u64));
            println!("the data 144 {}", *((ptr_u64 + 144) as *mut u64));
            println!("the data 152 {}", *((ptr_u64 + 152) as *mut u64));
            println!("the data 160 {}", *((ptr_u64 + 160) as *mut u64));
            println!("the data 168 {}", *((ptr_u64 + 168) as *mut u64));
            println!("the data 176 {}", *((ptr_u64 + 176) as *mut u64));
            println!("the data 184 {}", *((ptr_u64 + 184) as *mut u64));
            println!("the data 192 {}", *((ptr_u64 + 192) as *mut u64));
            println!("the data 200 {}", *((ptr_u64 + 200) as *mut u64));
            println!("the data 208 {}", *((ptr_u64 + 208) as *mut u64));
            println!("the data 216 {}", *((ptr_u64 + 216) as *mut u64));
            println!("the data 224 {}", *((ptr_u64 + 224) as *mut u64));
            println!("the data 232 {}", *((ptr_u64 + 232) as *mut u64));
            println!("the data 240 {}", *((ptr_u64 + 240) as *mut u64));
            println!("the data 248 {}", *((ptr_u64 + 248) as *mut u64));
            println!("the data 256 hustatus {}", *((ptr_u64 + 256) as *mut u64));
            
        }

        // Always wrong to get output
        assert_eq!(1, 0);
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
        
        let tmp = vcpu.vcpu_ctx.guest_ctx.sys_regs.vsatp;
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

        vcpu.vcpu_ctx.guest_ctx.sys_regs.vsatp = 17;
        let tmp = vcpu.vcpu_ctx.guest_ctx.sys_regs.vsatp;
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
            sysreg = i.lock().unwrap().vcpu_ctx.guest_ctx.sys_regs.vsscratch;
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
