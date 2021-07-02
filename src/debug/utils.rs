pub mod configtest {
    use crate::mm::utils::*;
    use crate::init::cmdline::VMConfig;
    use crate::mm::gparegion::GpaRegion;

    const ELF_IMG_PATH: &str = "./tests/integration/vcpu_add_all_gprs.img";
    const DTB_PATH: &str = "./test-files-laputa/hifive-unleashed-a00.dtb";
    const TEST_MMIO_REGION_1: GpaRegion = GpaRegion {
        gpa: 0x18000,
        length: 0x2000,
    };
    const TEST_MMIO_REGION_2: GpaRegion = GpaRegion {
        gpa: 0x34000,
        length: 0x3000,
    };
    const TEST_MMIO_REGION_3: GpaRegion = GpaRegion {
        gpa: 0x77000,
        length: 0x4000,
    };

    pub fn test_vm_config_create() -> VMConfig {
        let mut test_vm_config: VMConfig = VMConfig {
            vcpu_count: 1,
            mem_size: 1 << TB_SHIFT,
            machine_type: String::from(""),
            kernel_img_path: String::from(ELF_IMG_PATH),
            initrd_path: String::from(""),
            dtb_path: String::from(DTB_PATH),
            mmio_regions: Vec::new(),
        };

        test_vm_config.mmio_regions.push(TEST_MMIO_REGION_1);
        test_vm_config.mmio_regions.push(TEST_MMIO_REGION_2);
        test_vm_config.mmio_regions.push(TEST_MMIO_REGION_3);

        test_vm_config
    }
}