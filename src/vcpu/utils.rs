#![feature(llvm_asm)]
#![feature(global_asm)]
#![allow(non_upper_case_globals)]
#![allow(unused)]

pub const ustatus: u64 = 0x000;
pub const uie: u64 = 0x004;
pub const utvec: u64 = 0x005;
pub const uscratch: u64 = 0x040;
pub const uepc: u64 = 0x041;
pub const ucause: u64 = 0x042;
pub const utval: u64 = 0x043;
pub const uip: u64 = 0x044;
pub const fflags: u64 = 0x001;
pub const frm: u64 = 0x002;
pub const fcsr: u64 = 0x003;
pub const cycle: u64 = 0xc00;
pub const time: u64 = 0xc01;
pub const instret: u64 = 0xc02;
pub const hpmcounter3: u64 = 0xc03;
pub const hpmcounter4: u64 = 0xc04;
pub const hpmcounter5: u64 = 0xc05;
pub const hpmcounter6: u64 = 0xc06;
pub const hpmcounter7: u64 = 0xc07;
pub const hpmcounter8: u64 = 0xc08;
pub const hpmcounter9: u64 = 0xc09;
pub const hpmcounter10: u64 = 0xc0a;
pub const hpmcounter11: u64 = 0xc0b;
pub const hpmcounter12: u64 = 0xc0c;
pub const hpmcounter13: u64 = 0xc0d;
pub const hpmcounter14: u64 = 0xc0e;
pub const hpmcounter15: u64 = 0xc0f;
pub const hpmcounter16: u64 = 0xc10;
pub const hpmcounter17: u64 = 0xc11;
pub const hpmcounter18: u64 = 0xc12;
pub const hpmcounter19: u64 = 0xc13;
pub const hpmcounter20: u64 = 0xc14;
pub const hpmcounter21: u64 = 0xc15;
pub const hpmcounter22: u64 = 0xc16;
pub const hpmcounter23: u64 = 0xc17;
pub const hpmcounter24: u64 = 0xc18;
pub const hpmcounter25: u64 = 0xc19;
pub const hpmcounter26: u64 = 0xc1a;
pub const hpmcounter27: u64 = 0xc1b;
pub const hpmcounter28: u64 = 0xc1c;
pub const hpmcounter29: u64 = 0xc1d;
pub const hpmcounter30: u64 = 0xc1e;
pub const hpmcounter31: u64 = 0xc1f;
pub const cycleh: u64 = 0xc80;
pub const timeh: u64 = 0xc81;
pub const instreth: u64 = 0xc82;
pub const hpmcounter3h: u64 = 0xc83;
pub const hpmcounter4h: u64 = 0xc84;
pub const hpmcounter5h: u64 = 0xc85;
pub const hpmcounter6h: u64 = 0xc86;
pub const hpmcounter7h: u64 = 0xc87;
pub const hpmcounter8h: u64 = 0xc88;
pub const hpmcounter9h: u64 = 0xc89;
pub const hpmcounter10h: u64 = 0xc8a;
pub const hpmcounter11h: u64 = 0xc8b;
pub const hpmcounter12h: u64 = 0xc8c;
pub const hpmcounter13h: u64 = 0xc8d;
pub const hpmcounter14h: u64 = 0xc8e;
pub const hpmcounter15h: u64 = 0xc8f;
pub const hpmcounter16h: u64 = 0xc90;
pub const hpmcounter17h: u64 = 0xc91;
pub const hpmcounter18h: u64 = 0xc92;
pub const hpmcounter19h: u64 = 0xc93;
pub const hpmcounter20h: u64 = 0xc94;
pub const hpmcounter21h: u64 = 0xc95;
pub const hpmcounter22h: u64 = 0xc96;
pub const hpmcounter23h: u64 = 0xc97;
pub const hpmcounter24h: u64 = 0xc98;
pub const hpmcounter25h: u64 = 0xc99;
pub const hpmcounter26h: u64 = 0xc9a;
pub const hpmcounter27h: u64 = 0xc9b;
pub const hpmcounter28h: u64 = 0xc9c;
pub const hpmcounter29h: u64 = 0xc9d;
pub const hpmcounter30h: u64 = 0xc9e;
pub const hpmcounter31h: u64 = 0xc9f;
pub const mcycle: u64 = 0xb00;
pub const minstret: u64 = 0xb02;
pub const mcycleh: u64 = 0xb80;
pub const minstreth: u64 = 0xb82;
pub const mvendorid: u64 = 0xf11;
pub const marchid: u64 = 0xf12;
pub const mimpid: u64 = 0xf13;
pub const mhartid: u64 = 0xf14;
pub const mstatus: u64 = 0x300;
pub const misa: u64 = 0x301;
pub const medeleg: u64 = 0x302;
pub const mideleg: u64 = 0x303;
pub const mie: u64 = 0x304;
pub const mtvec: u64 = 0x305;
pub const mcounteren: u64 = 0x306;
pub const mtvt: u64 = 0x307;
pub const mucounteren: u64 = 0x320;
pub const mscounteren: u64 = 0x321;
pub const mscratch: u64 = 0x340;
pub const mepc: u64 = 0x341;
pub const mcause: u64 = 0x342;
pub const mbadaddr: u64 = 0x343;
pub const mtval: u64 = 0x343;
pub const mip: u64 = 0x344;
pub const mnxti: u64 = 0x345;
pub const mintstatus: u64 = 0x346;
pub const mscratchcsw: u64 = 0x348;
pub const sstatus: u64 = 0x100;
pub const sedeleg: u64 = 0x102;
pub const sideleg: u64 = 0x103;
pub const sie: u64 = 0x104;
pub const stvec: u64 = 0x105;
pub const scounteren: u64 = 0x106;
pub const stvt: u64 = 0x107;
pub const sscratch: u64 = 0x140;
pub const sepc: u64 = 0x141;
pub const scause: u64 = 0x142;
pub const sbadaddr: u64 = 0x143;
pub const stval: u64 = 0x143;
pub const sip: u64 = 0x144;
pub const snxti: u64 = 0x145;
pub const sintstatus: u64 = 0x146;
pub const sscratchcsw: u64 = 0x148;
pub const sptbr: u64 = 0x180;
pub const satp: u64 = 0x180;
pub const pmpcfg0: u64 = 0x3a0;
pub const pmpcfg1: u64 = 0x3a1;
pub const pmpcfg2: u64 = 0x3a2;
pub const pmpcfg3: u64 = 0x3a3;
pub const pmpaddr0: u64 = 0x3b0;
pub const pmpaddr1: u64 = 0x3b1;
pub const pmpaddr2: u64 = 0x3b2;
pub const pmpaddr3: u64 = 0x3b3;
pub const pmpaddr4: u64 = 0x3b4;
pub const pmpaddr5: u64 = 0x3b5;
pub const pmpaddr6: u64 = 0x3b6;
pub const pmpaddr7: u64 = 0x3b7;
pub const pmpaddr8: u64 = 0x3b8;
pub const pmpaddr9: u64 = 0x3b9;
pub const pmpaddr10: u64 = 0x3ba;
pub const pmpaddr11: u64 = 0x3bb;
pub const pmpaddr12: u64 = 0x3bc;
pub const pmpaddr13: u64 = 0x3bd;
pub const pmpaddr14: u64 = 0x3be;
pub const pmpaddr15: u64 = 0x3bf;
pub const tselect: u64 = 0x7a0;
pub const tdata1: u64 = 0x7a1;
pub const tdata2: u64 = 0x7a2;
pub const tdata3: u64 = 0x7a3;
pub const dcsr: u64 = 0x7b0;
pub const dpc: u64 = 0x7b1;
pub const dscratch: u64 = 0x7b2;
pub const mhpmcounter3: u64 = 0xb03;
pub const mhpmcounter4: u64 = 0xb04;
pub const mhpmcounter5: u64 = 0xb05;
pub const mhpmcounter6: u64 = 0xb06;
pub const mhpmcounter7: u64 = 0xb07;
pub const mhpmcounter8: u64 = 0xb08;
pub const mhpmcounter9: u64 = 0xb09;
pub const mhpmcounter10: u64 = 0xb0a;
pub const mhpmcounter11: u64 = 0xb0b;
pub const mhpmcounter12: u64 = 0xb0c;
pub const mhpmcounter13: u64 = 0xb0d;
pub const mhpmcounter14: u64 = 0xb0e;
pub const mhpmcounter15: u64 = 0xb0f;
pub const mhpmcounter16: u64 = 0xb10;
pub const mhpmcounter17: u64 = 0xb11;
pub const mhpmcounter18: u64 = 0xb12;
pub const mhpmcounter19: u64 = 0xb13;
pub const mhpmcounter20: u64 = 0xb14;
pub const mhpmcounter21: u64 = 0xb15;
pub const mhpmcounter22: u64 = 0xb16;
pub const mhpmcounter23: u64 = 0xb17;
pub const mhpmcounter24: u64 = 0xb18;
pub const mhpmcounter25: u64 = 0xb19;
pub const mhpmcounter26: u64 = 0xb1a;
pub const mhpmcounter27: u64 = 0xb1b;
pub const mhpmcounter28: u64 = 0xb1c;
pub const mhpmcounter29: u64 = 0xb1d;
pub const mhpmcounter30: u64 = 0xb1e;
pub const mhpmcounter31: u64 = 0xb1f;
pub const mhpmevent3: u64 = 0x323;
pub const mhpmevent4: u64 = 0x324;
pub const mhpmevent5: u64 = 0x325;
pub const mhpmevent6: u64 = 0x326;
pub const mhpmevent7: u64 = 0x327;
pub const mhpmevent8: u64 = 0x328;
pub const mhpmevent9: u64 = 0x329;
pub const mhpmevent10: u64 = 0x32a;
pub const mhpmevent11: u64 = 0x32b;
pub const mhpmevent12: u64 = 0x32c;
pub const mhpmevent13: u64 = 0x32d;
pub const mhpmevent14: u64 = 0x32e;
pub const mhpmevent15: u64 = 0x32f;
pub const mhpmevent16: u64 = 0x330;
pub const mhpmevent17: u64 = 0x331;
pub const mhpmevent18: u64 = 0x332;
pub const mhpmevent19: u64 = 0x333;
pub const mhpmevent20: u64 = 0x334;
pub const mhpmevent21: u64 = 0x335;
pub const mhpmevent22: u64 = 0x336;
pub const mhpmevent23: u64 = 0x337;
pub const mhpmevent24: u64 = 0x338;
pub const mhpmevent25: u64 = 0x339;
pub const mhpmevent26: u64 = 0x33a;
pub const mhpmevent27: u64 = 0x33b;
pub const mhpmevent28: u64 = 0x33c;
pub const mhpmevent29: u64 = 0x33d;
pub const mhpmevent30: u64 = 0x33e;
pub const mhpmevent31: u64 = 0x33f;
pub const mhpmcounter3h: u64 = 0xb83;
pub const mhpmcounter4h: u64 = 0xb84;
pub const mhpmcounter5h: u64 = 0xb85;
pub const mhpmcounter6h: u64 = 0xb86;
pub const mhpmcounter7h: u64 = 0xb87;
pub const mhpmcounter8h: u64 = 0xb88;
pub const mhpmcounter9h: u64 = 0xb89;
pub const mhpmcounter10h: u64 = 0xb8a;
pub const mhpmcounter11h: u64 = 0xb8b;
pub const mhpmcounter12h: u64 = 0xb8c;
pub const mhpmcounter13h: u64 = 0xb8d;
pub const mhpmcounter14h: u64 = 0xb8e;
pub const mhpmcounter15h: u64 = 0xb8f;
pub const mhpmcounter16h: u64 = 0xb90;
pub const mhpmcounter17h: u64 = 0xb91;
pub const mhpmcounter18h: u64 = 0xb92;
pub const mhpmcounter19h: u64 = 0xb93;
pub const mhpmcounter20h: u64 = 0xb94;
pub const mhpmcounter21h: u64 = 0xb95;
pub const mhpmcounter22h: u64 = 0xb96;
pub const mhpmcounter23h: u64 = 0xb97;
pub const mhpmcounter24h: u64 = 0xb98;
pub const mhpmcounter25h: u64 = 0xb99;
pub const mhpmcounter26h: u64 = 0xb9a;
pub const mhpmcounter27h: u64 = 0xb9b;
pub const mhpmcounter28h: u64 = 0xb9c;
pub const mhpmcounter29h: u64 = 0xb9d;
pub const mhpmcounter30h: u64 = 0xb9e;
pub const mhpmcounter31h: u64 = 0xb9f;

// HU-extension CSRs
pub const vtimecmp:  u64 = 0x401;
pub const vtimecmph: u64 = 0x481;

/// atomic read from CSR

pub macro_rules! csrr {
    ( $r:ident ) => {{
        let value: u64;
        #[allow(unused_unsafe)]
        unsafe { unsafe{ llvm_asm!("csrr $0, $1" : "=r"(value) : "i"($r)) } };
        value
    }};
}

/// atomic write to CSR

pub macro_rules! csrw {
    ( $r:ident, $x:expr ) => {{
        let x: u64 = $x;
        unsafe{ llvm_asm!("csrw $0, $1" :: "i"($r), "r"(x) :: "volatile") };
    }};
}

/// atomic write to CSR from immediate

pub macro_rules! csrwi {
    ( $r:ident, $x:expr ) => {{
        const X: u64 = $x;
        unsafe{ llvm_asm!("csrwi $0, $1" :: "i"($r), "i"(X) :: "volatile") };
    }};
}

/// atomic read and set bits in CSR

pub macro_rules! csrs {
    ( $r:ident, $x:expr ) => {{
        let x: u64 = $x;
        unsafe{ llvm_asm!("csrs $0, $1" :: "i"($r), "r"(x) :: "volatile") };
    }};
}

/// atomic read and set bits in CSR using immediate

pub macro_rules! csrsi {
    ( $r:ident, $x:expr ) => {{
        const X: u64 = $x;
        unsafe{ llvm_asm!("csrsi $0, $1" :: "i"($r), "i"(X) :: "volatile") };
    }};
}

/// atomic read and clear bits in CSR

pub macro_rules! csrc {
    ( $r:ident, $x:expr ) => {{
        let x: u64 = $x;
        unsafe{ llvm_asm!("csrc $0, $1" :: "i"($r), "r"(x) :: "volatile") };
    }};
}

/// atomic read and clear bits in CSR using immediate

pub macro_rules! csrci {
    ( $r:ident, $x:expr ) => {{
        const X: u64 = $x;
        unsafe{ llvm_asm!("csrci $0, $1" :: "i"($r), "i"(X) :: "volatile") };
    }};
}