#![allow(unused)]
pub mod gp_reg_constants {
    pub const ZERO : usize = 0;
    pub const RA : usize = 1;
    pub const SP : usize = 2;
    pub const GP : usize = 3;
    pub const TP : usize = 4;
    pub const T0 : usize = 5;
    pub const T1 : usize = 6;
    pub const T2 : usize = 7;
    pub const S0 : usize = 8;
    pub const S1 : usize = 9;
    pub const A0 : usize = 10;
    pub const A1 : usize = 11;
    pub const A2 : usize = 12;
    pub const A3 : usize = 13;
    pub const A4 : usize = 14;
    pub const A5 : usize = 15;
    pub const A6 : usize = 16;
    pub const A7 : usize = 17;
    pub const S2 : usize = 18;
    pub const S3 : usize = 19;
    pub const S4 : usize = 20;
    pub const S5 : usize = 21;
    pub const S6 : usize = 22;
    pub const S7 : usize = 23;
    pub const S8 : usize = 24;
    pub const S9 : usize = 25;
    pub const S10 : usize = 26;
    pub const S11 : usize = 27;
    pub const T3 : usize = 28;
    pub const T4 : usize = 29;
    pub const T5 : usize = 30;
    pub const T6 : usize = 31;
}

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

/* SysReg for Guest */
#[repr(C)]
pub struct SysRegs {
    pub huvsstatus: u64,
    pub huvsip: u64,
    pub huvsie: u64,
    pub huvstvec: u64,
    pub huvsscratch: u64,
    pub huvsepc: u64,
    pub huvscause: u64,
    pub huvstval: u64,
    pub huvsatp: u64,
}

impl SysRegs {
    pub fn new() -> Self {
        Self {
            huvsstatus: 0,
            huvsip: 0,
            huvsie: 0,
            huvstvec: 0,
            huvsscratch: 0,
            huvsepc: 0,
            huvscause: 0,
            huvstval: 0,
            huvsatp: 0,
        }
    }
}

#[repr(C)]
pub struct HypRegs {
    pub hustatus: u64,
    pub huedeleg: u64,
    pub huideleg: u64,
    pub huie: u64, 

    /* TODO: scounteren & hucounteren */
    pub hucounteren: u64,
    pub hutval: u64,
    pub huvip: u64,
    pub huip: u64,
    /* TODO: hip & hie in doc */

    /* TODO: In doc: Direct IRQ to VM, not needed in HU-mode? */
    pub hugeip: u64,

    /* TODO: In doc: Direct IRQ to VM, not needed in HU-mode? */
    pub hugeie: u64,

    pub hutimedelta: u64,
    pub hutimedeltah: u64,
    pub hutinst: u64,
    pub hugatp: u64,
    pub utvec: u64,
    pub uepc: u64, /* for sepc */
    pub uscratch: u64, /* for sscratch */
    pub utval: u64, /* for stval */
    pub ucause: u64, /* for scause */
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

/* Context for both ULH & VM */
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