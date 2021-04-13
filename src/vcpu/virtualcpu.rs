use crate::vm::virtualmachine;
use crate::irq::virq;
use crate::irq::vtimer;
use std::sync::{Arc, Mutex};

pub struct GpRegs {
    pub x_reg: [u64; 32]
}

impl GpRegs {
    pub fn new() -> Self {
        Self {
            x_reg: [0; 32],
        }
    }
}

pub struct SysRegs { //scounteren?
    pub vsstatus: u64,
    pub vsip: u64,
    pub vsie: u64,
    pub vstec: u64,
    pub vsscratch: u64,
    pub vsepc: u64,
    pub vscause: u64,
    pub vstval: u64,
    pub vsatp: u64,
    pub vscounteren: u64 // save for scounteren
}

impl SysRegs {
    pub fn new() -> Self {
        Self {
            vsstatus: 0,
            vsip: 0,
            vsie: 0,
            vstec: 0,
            vsscratch: 0,
            vsepc: 0,
            vscause: 0,
            vstval: 0,
            vsatp: 0,
            // For scounteren
            vscounteren: 0
        }
    }
}

pub struct HypRegs {
    pub hustatus: u64,
    pub huedeleg: u64,
    pub huideleg: u64,
    pub huvip: u64,
    pub huip: u64,
    pub huie: u64, 
    // TODO: hip & hie in doc

    // TODO: In doc: Direct IRQ to VM, not needed in HU-mode?
    pub hugeip: u64,

    // TODO: In doc: Direct IRQ to VM, not needed in HU-mode?
    pub hugeie: u64,

    pub hucounteren: u64,
    pub hutimedelta: u64,
    pub hutimedeltah: u64,
    pub hutval: u64,
    pub hutinst: u64,
    pub hugatp: u64,
    pub utvec: u64,
    pub uepc: u64, // for sepc
    pub uscratch: u64, // for sscratch
    pub utval: u64, // for stval
    pub ucause: u64, // for scause
    pub scounteren: u64, // move from SysReg to reduce HostCtx
}

impl HypRegs {
    pub fn new() -> Self {
        Self {
            hustatus: 0,
            huedeleg: 0,
            huideleg: 0,
            huvip: 0,
            huip: 0,
            huie: 0, 
            hugeip: 0,
            hugeie: 0,
            hucounteren: 0,
            hutimedelta: 0,
            hutimedeltah: 0,
            hutval: 0,
            hutinst: 0,
            hugatp: 0,
            utvec: 0,
            uepc: 0,
            uscratch: 0,
            utval: 0,
            ucause: 0,
            scounteren: 0,
        }
    }
}

pub struct HostCtx {
    pub gp_regs: GpRegs,
    pub hyp_regs: HypRegs
}

impl HostCtx {
    pub fn new() -> Self {
        let gp_regs = GpRegs::new();
        let hyp_regs = HypRegs::new();

        Self {
            gp_regs,
            hyp_regs
        }
    }
}

pub struct GuestCtx {
    pub gp_regs: GpRegs,
    pub sys_regs: SysRegs,
    pub hyp_regs: HypRegs
}

impl GuestCtx {
    pub fn new() -> Self {
        let gp_regs = GpRegs::new();
        let sys_regs = SysRegs::new();
        let hyp_regs = HypRegs::new();

        Self {
            gp_regs,
            sys_regs,
            hyp_regs
        }
    }
}

// Context for both ULH & VM
pub struct VcpuCtx {
    pub host_ctx: HostCtx,
    pub guest_ctx: GuestCtx
}

impl VcpuCtx {
    pub fn new() -> Self {
        let host_ctx = HostCtx::new();
        let guest_ctx = GuestCtx::new();

        Self {
            host_ctx,
            guest_ctx
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

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
