// User Hypervisor Extension (UHE)

#[allow(unused)]
pub mod ioctl_constants {
    // ioctl id
    pub const IOCTL_LAPUTA_GET_API_VERSION: u64 = 0x80086B01;
    pub const IOCTL_LAPUTA_REQUEST_DELEG: u64 = 0x40106B03;
    pub const IOCTL_LAPUTA_REGISTER_VCPU: u64 = 0x6B04;
    pub const IOCTL_LAPUTA_UNREGISTER_VCPU: u64 = 0x6B05;
}

#[allow(unused)]
pub mod hustatus_constants {
    pub const HUSTATUS_SPV: u64 = 7;
    pub const HUSTATUS_SPVP: u64 = 8;
}