use crate::vm::virtualmachine;
use crate::irq::virq;
use crate::irq::vtimer;
use crate::vcpu::context;
use std::sync::{Arc, Mutex};
use context::*;

global_asm!(include_str!("../asm_offset.S"));
global_asm!(include_str!("../asm_csr.S"));
global_asm!(include_str!("../fence.S"));

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

#[no_mangle]
pub fn vm_code_print() {
    println!("Enter vm");
}

#[no_mangle]
pub unsafe fn vm_code_ecall() {
    llvm_asm!(".align 2
            ecall":::: "volatile");
}

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub unsafe fn enter_guest_inline(ctx: u64) {
    llvm_asm!(".align 2
            // a0 point to vcpu_ctx
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

            /* a0 = ctx */

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
            //csrrw t3, 64, a0 //debug
            SAVE_HOST_HYP_USCRATCH a0, t3

            // debug
            CSRR_CSR_USCRATCH t3
            SAVE_HOST_GP_X0 a0, t3

            /* debug */
            //mv t6, a0
            //CSRR_CSR_USCRATCH t4
            //CSRW_CSR_USCRATCH t6
            //CSRR_CSR_USCRATCH t3
            //csrr t3, 64
            //SAVE_HOST_HYP_USCRATCH a0, t3

            /* debug */
            //CSRW_CSR_USCRATCH a0
            //CSRR_CSR_USCRATCH t3
            //SAVE_HOST_GP_X30 a0, t3

            /* debug for USCRATCH */
            //li t4, 0x100
            //CSRW_CSR_USCRATCH a0
            //csrrw t4, 64, a0
            //CSRR_CSR_USCRATCH t3
            //SAVE_HOST_GP_X0 a0, t3
            //SAVE_GUEST_GP_X0 a0, t4

            /* set utvec to catch the trap & save UTVEC*/
            la	t4, __vm_exit
            CSRRW_CSR_UTVEC t4, t4
            SAVE_HOST_HYP_UTVEC a0, t4
            //CSRR_CSR_UTVEC t3
            //SAVE_HOST_GP_X6 a0, t3

            /* set UEPC */
            RESTORE_GUEST_HYP_UEPC a0, t0
            la t0, __vm_code // for debug
            CSRW_CSR_UEPC t0
            //CSRR_CSR_UEPC t3
            //SAVE_HOST_GP_X7 a0, t3


            /* set HUGATP */
            RESTORE_GUEST_HYP_HUGATP a0, t0
            CSRW_CSR_HUGATP t0
            //CSRR_CSR_HUGATP t3
            //SAVE_HOST_GP_X8 a0, t3

            //CSRR_CSR_HUVSATP t3
            //SAVE_HOST_GP_X9 a0, t3

            //.insn r 0x73, 0x0, 0x71, x0, x0, x0
            //.insn r 0x73, 0x0, 0x51, x0, x0, x0
            //.word 0x000000510073
            //.word 0x000000710073
            //.word 0x730071000000
            //.word 0x730051000000
            //HUFENCE_VVMA x0, x0, x0
            //HUFENCE_GVMA x0, x0, x0
            .word 0xE2000073
            //.word 0xC2000073
            //.word 0x73007100
            //.word 0x73005100
            //.word 0x00510073
            //.word 0x00710073


            /* debug for huret */
            //CSRW_CSR_UEPC t4
            //SAVE_HOST_HYP_USCRATCH a0, t0
            //la t0, vm_code_ecall
            //SAVE_HOST_GP_X31 a0, t0

            /* debug */
            //mv t5, a0

            // restore guest GP except A0 & X0
            RESTORE_GUEST_GP_X1 a0, x1
            RESTORE_GUEST_GP_X2 a0, x2
            RESTORE_GUEST_GP_X3 a0, x3
            RESTORE_GUEST_GP_X4 a0, x4
            RESTORE_GUEST_GP_X5 a0, x5
            RESTORE_GUEST_GP_X6 a0, x6
            RESTORE_GUEST_GP_X7 a0, x7
            RESTORE_GUEST_GP_X8 a0, x8
            RESTORE_GUEST_GP_X9 a0, x9
            //RESTORE_GUEST_GP_X10 a0, x10 // a0
            RESTORE_GUEST_GP_X11 a0, x11
            RESTORE_GUEST_GP_X12 a0, x12
            RESTORE_GUEST_GP_X13 a0, x13
            RESTORE_GUEST_GP_X14 a0, x14
            RESTORE_GUEST_GP_X15 a0, x15
            RESTORE_GUEST_GP_X16 a0, x16
            RESTORE_GUEST_GP_X17 a0, x17
            RESTORE_GUEST_GP_X18 a0, x18
            RESTORE_GUEST_GP_X19 a0, x19
            RESTORE_GUEST_GP_X20 a0, x20
            RESTORE_GUEST_GP_X21 a0, x21
            RESTORE_GUEST_GP_X22 a0, x22
            RESTORE_GUEST_GP_X23 a0, x23
            RESTORE_GUEST_GP_X24 a0, x24
            RESTORE_GUEST_GP_X25 a0, x25
            RESTORE_GUEST_GP_X26 a0, x26
            RESTORE_GUEST_GP_X27 a0, x27
            RESTORE_GUEST_GP_X28 a0, x28
            RESTORE_GUEST_GP_X29 a0, x29
            RESTORE_GUEST_GP_X30 a0, x30
            RESTORE_GUEST_GP_X31 a0, x31

            /* restore guest A0 */
            RESTORE_GUEST_GP_X10 a0, x10

            /* uret */
            uret
            //.word 0x200073

            .align 2
            __vm_exit:


            /* save guest-a0 in sscratch & get host-a0 */
            CSRRW_CSR_USCRATCH a0, a0
            SAVE_HOST_GP_X0 a0, a0



            /* debug */
            //mv t4, a0
            //mv a0, t5
            //SAVE_GUEST_GP_X0 a0, t4
            //CSRR_CSR_USCRATCH t4
            //SAVE_HOST_GP_X31 a0, t4

            /* a0 = ctx */

            /* save guest gp except A0 & X0 */
            SAVE_GUEST_GP_X1 a0, x1
            SAVE_GUEST_GP_X2 a0, x2
            SAVE_GUEST_GP_X3 a0, x3
            SAVE_GUEST_GP_X4 a0, x4
            SAVE_GUEST_GP_X5 a0, x5
            SAVE_GUEST_GP_X6 a0, x6
            SAVE_GUEST_GP_X7 a0, x7
            SAVE_GUEST_GP_X8 a0, x8
            SAVE_GUEST_GP_X9 a0, x9
            SAVE_GUEST_GP_X11 a0, x11
            SAVE_GUEST_GP_X12 a0, x12
            SAVE_GUEST_GP_X13 a0, x13
            SAVE_GUEST_GP_X14 a0, x14
            SAVE_GUEST_GP_X15 a0, x15
            SAVE_GUEST_GP_X16 a0, x16
            SAVE_GUEST_GP_X17 a0, x17
            SAVE_GUEST_GP_X18 a0, x18
            SAVE_GUEST_GP_X19 a0, x19
            SAVE_GUEST_GP_X20 a0, x20
            SAVE_GUEST_GP_X21 a0, x21
            SAVE_GUEST_GP_X22 a0, x22
            SAVE_GUEST_GP_X23 a0, x23
            SAVE_GUEST_GP_X24 a0, x24
            SAVE_GUEST_GP_X25 a0, x25
            SAVE_GUEST_GP_X26 a0, x26
            SAVE_GUEST_GP_X27 a0, x27
            SAVE_GUEST_GP_X28 a0, x28
            SAVE_GUEST_GP_X29 a0, x29
            SAVE_GUEST_GP_X30 a0, x30
            SAVE_GUEST_GP_X31 a0, x31

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
            RESTORE_HOST_GP_X1 a0, x1
            RESTORE_HOST_GP_X2 a0, x2
            RESTORE_HOST_GP_X3 a0, x3
            RESTORE_HOST_GP_X4 a0, x4
            RESTORE_HOST_GP_X5 a0, x5
            RESTORE_HOST_GP_X6 a0, x6
            RESTORE_HOST_GP_X7 a0, x7
            RESTORE_HOST_GP_X8 a0, x8
            RESTORE_HOST_GP_X9 a0, x9
            RESTORE_HOST_GP_X10 a0, x10
            RESTORE_HOST_GP_X11 a0, x11
            RESTORE_HOST_GP_X12 a0, x12
            RESTORE_HOST_GP_X13 a0, x13
            RESTORE_HOST_GP_X14 a0, x14
            RESTORE_HOST_GP_X15 a0, x15
            RESTORE_HOST_GP_X16 a0, x16
            RESTORE_HOST_GP_X17 a0, x17
            RESTORE_HOST_GP_X18 a0, x18
            RESTORE_HOST_GP_X19 a0, x19
            RESTORE_HOST_GP_X20 a0, x20
            RESTORE_HOST_GP_X21 a0, x21
            RESTORE_HOST_GP_X22 a0, x22
            RESTORE_HOST_GP_X23 a0, x23
            RESTORE_HOST_GP_X24 a0, x24
            RESTORE_HOST_GP_X25 a0, x25
            RESTORE_HOST_GP_X26 a0, x26
            RESTORE_HOST_GP_X27 a0, x27
            //RESTORE_HOST_GP_X28 a0, x28
            //RESTORE_HOST_GP_X29 a0, x29
            //RESTORE_HOST_GP_X30 a0, x30
            //RESTORE_HOST_GP_X31 a0, x31


            la t6, __guest_end
            jr t6


            .align 2
          __vm_code:
            li	t0,	0
	        li  a0, 0
	        li  a0, 0
	        //li	a1,	1
	        //li	a2,	2
	        //mv	t0, a0
	        //mv	t1,	a1
	        //mv	t2,	a2
	        //li	t4, 0x2000
	        //ld	t2,	0(t4)
	        //li	t4, 0x3000
	        //ld	t3, 0(t4)
	        //add	t0,	t0,	t1
	        //mul t2, t2, t0
	        //ecall

            .align 2
            __guest_end:

            " :: "r"(ctx) :"memory", "x5", "x6", "x7", "x10", "x11", "x28", "x29", "x30", "x31": "volatile");

            // Save the key reg
            // UCAUSE / UTVAL
            llvm_asm!(".align 2
                mv a0, $0
                csrr t3, 0x42
                SAVE_HOST_GP_X1 a0, t3
                csrr t3, 0x43
                SAVE_HOST_GP_X2 a0, t3
            " :: "r"(ctx) :"memory", "x10", "x28": "volatile");
            //"memory", "x5", "x6", "x7", "x10", "x28", "x29", "x30", "x31"
            //"memory", "a0", "t0", "t1", "t2", "t3", "t4", "t5", "t6"
}

pub fn print_ctx() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::ffi::CString;
    use crate::mm::gstagemmu;

    #[test]
    fn test_struct_ptr() { 
        let mut vcpuctx = VcpuCtx::new();
        let mut fd;
        let file_path = CString::new("/dev/laputa_dev").unwrap();


        let mut tmp_buf_pfn: u64 = 0;
        unsafe { 
            fd = libc::open(file_path.as_ptr(), libc::O_RDWR); 

            // ioctl(fd_ioctl, IOCTL_LAPUTA_GET_API_VERSION, &tmp_buf_pfn) // 0x80086b01
            let tmp_buf_pfn_ptr = (&tmp_buf_pfn) as *const u64;
            let res1 = libc::ioctl(fd, 0x80086b01, tmp_buf_pfn_ptr);
            println!("tmp_buf_pfn : {:x}", tmp_buf_pfn);

            // ioctl(fd_ioctl, IOCTL_LAPUTA_REQUEST_DELEG, deleg_info)
            let edeleg = ((1<<20) | (1<<21) | (1<<23)) as libc::c_ulong; // guest page fault(sedeleg)
            let ideleg = (1<<0) as libc::c_ulong;
            let deleg = [edeleg,ideleg];
            let deleg_ptr = (&deleg) as *const u64;
            let res2 = libc::ioctl(fd, 1074817795, deleg_ptr);
            println!("ioctl {}", res2);

            let res3 = libc::ioctl(fd, 0x6b04);
            println!("ioctl {}", res3);




        }

        let ptr = &vcpuctx as *const VcpuCtx;
        println!("the ptr is {}", ptr as u64);
        let ptr_u64 = ptr as u64;
        unsafe {
            let gsmmu = gstagemmu::GStageMmu::new();
            let pt_hpa = (tmp_buf_pfn | (1 << 63));
            vcpuctx.guest_ctx.hyp_regs.hugatp = pt_hpa;
            //vcpuctx.guest_ctx.hyp_regs.hugatp = 0x0;
            println!("HUGATP : {:x}", pt_hpa);
            //libc::println

            //hustatus.SPP=1 .SPVP=1 uret to VS mode
            vcpuctx.guest_ctx.hyp_regs.hustatus = ((1 << 8) | (1 << 7)) as u64;
            //VirtualCpu::open_hu_extension(ioctl_fd);

            enter_guest_inline(ptr_u64);

            

            println!("the data 0 {:x}", *(ptr_u64 as *mut u64));
            println!("the data 416 {:x}", *((ptr_u64 + 416) as *mut u64));
            println!("the data HOST_HYP_USCRATCH {:x}", *((ptr_u64 + 384) as *mut u64));
            println!("the data 8 {:x}", *((ptr_u64 + 8) as *mut u64));
            println!("the data 16 {:x}", *((ptr_u64 + 16) as *mut u64));
            println!("the data 24 {:x}", *((ptr_u64 + 24) as *mut u64));
            println!("the data 32 {:x}", *((ptr_u64 + 32) as *mut u64));
            println!("the data 40 {:x}", *((ptr_u64 + 40) as *mut u64));
            println!("the data 48 {:x}", *((ptr_u64 + 48) as *mut u64));
            println!("the data 56 {:x}", *((ptr_u64 + 56) as *mut u64));
            println!("the data 64 {:x}", *((ptr_u64 + 64) as *mut u64));
            println!("the data 72 {:x}", *((ptr_u64 + 72) as *mut u64));
            println!("the data 80 {:x}", *((ptr_u64 + 80) as *mut u64));
            println!("the data 88 {:x}", *((ptr_u64 + 88) as *mut u64));
            println!("the data 96 {:x}", *((ptr_u64 + 96) as *mut u64));
            println!("the data 104 {:x}", *((ptr_u64 + 104) as *mut u64));
            println!("the data 112 {:x}", *((ptr_u64 + 112) as *mut u64));
            println!("the data 120 {:x}", *((ptr_u64 + 120) as *mut u64));
            println!("the data 128 {:x}", *((ptr_u64 + 128) as *mut u64));
            println!("the data 136 {:x}", *((ptr_u64 + 136) as *mut u64));
            println!("the data 144 {:x}", *((ptr_u64 + 144) as *mut u64));
            println!("the data 152 {:x}", *((ptr_u64 + 152) as *mut u64));
            println!("the data 160 {:x}", *((ptr_u64 + 160) as *mut u64));
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

            let res4 = libc::ioctl(fd, 0x6b05);
            println!("ioctl {}", res4);
        }
//
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
