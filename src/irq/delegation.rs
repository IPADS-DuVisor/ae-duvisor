#[allow(unused)]
pub mod delegation_constants {
    // Exception delegation
    pub const INST_GUEST_PAGE_FAULT: u64 = 20;
    pub const LOAD_GUEST_ACCESS_FAULT: u64 = 21;
    pub const VIRT_INSTRUCTION_FAULT: u64 = 22;
    pub const STORE_GUEST_AMO_ACCESS_FAULT: u64 = 23;

    // Interrupt delegation
    pub const S_SOFT: u64 = 0;
}