#[repr(C)]
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

#[repr(C)]
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

#[repr(C)]
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

#[repr(C)]
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

#[repr(C)]
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
#[repr(C)]
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