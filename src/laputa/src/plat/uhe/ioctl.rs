/* User Hypervisor Extension (UHE) */

#[allow(unused)]
pub mod ioctl_constants {
    /* Ioctl id */
    pub const IOCTL_LAPUTA_GET_API_VERSION: u64 = 0x80086B01;
    pub const IOCTL_LAPUTA_REQUEST_DELEG: u64 = 0x40106B03;
    pub const IOCTL_LAPUTA_REGISTER_VCPU: u64 = 0x6B04;
    pub const IOCTL_LAPUTA_UNREGISTER_VCPU: u64 = 0x6B05;
    pub const IOCTL_LAPUTA_QUERY_PFN: u64 = 0xc0086b06;
    pub const IOCTL_LAPUTA_RELEASE_PFN: u64 = 0x40086b07;
    pub const IOCTL_REMOTE_FENCE: u64 = 0x80106b08;
    pub const IOCTL_LAPUTA_GET_VMID: u64 = 0x80086b09;
    pub const IOCTL_LAPUTA_GET_VINTERRUPT_ADDR: u64 = 0x80086b0a;
    pub const IOCTL_LAPUTA_GET_CPUID: u64 = 0x80086b0b;
    pub const IOCTL_LAPUTA_SET_VINTERRUPT: u64 = 0x80086b0c;
    pub const IOCTL_LAPUTA_VPLIC_CLAIM: u64 = 0x80086b0d;
}
