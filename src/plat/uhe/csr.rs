// HU CSR

#[allow(unused)]
pub mod csr_constants {
    // HUSTATUS
    pub const HUSTATUS_SPV_SHIFT: u64 = 7;
    pub const HUSTATUS_SPVP_SHIFT: u64 = 8;

    pub const HUGATP_MODE_SHIFT: u64 = 60;
    pub const HUGATP_MODE_SV39: u64 = 8 << HUGATP_MODE_SHIFT;
    pub const HUGATP_MODE_SV48: u64 = 9 << HUGATP_MODE_SHIFT;
}