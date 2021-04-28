#[allow(unused)]
pub mod delegation_constants {
    // Exception delegation
    pub const EXC_VIRTUAL_SUPERVISOR_SYSCALL: u64 = 10;
    pub const EXC_INST_GUEST_PAGE_FAULT: u64 = 20;
    pub const EXC_LOAD_GUEST_PAGE_FAULT: u64 = 21;
    pub const EXC_VIRTUAL_INST_FAULT: u64 = 22;
    pub const EXC_STORE_GUEST_PAGE_FAULT: u64 = 23;
    pub const EXC_IRQ_MASK: u64 = 1 << 63;

    // Interrupt delegation
    pub const IRQ_S_SOFT: u64 = 0;
}
