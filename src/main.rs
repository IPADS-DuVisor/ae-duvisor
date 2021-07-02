use std::process;
use colored::*;
use laputa::init::cmdline;
use laputa::debug::utils::configtest::test_vm_config_create;
use laputa::mm::gparegion::GpaRegion;

fn main() {
    /* let vm_config = cmdline::VMConfig::new().unwrap_or_else(|err| {
        eprintln!("{}: {}", "error".bright_red(), err);
        process::exit(1);
    });

    if !cmdline::VMConfig::verify_args(&vm_config) {
        process::exit(1);
    } */

    let mut vm_config = test_vm_config_create();
    let elf_path: &str = "./test-files-laputa/Image";
    vm_config.kernel_img_path = String::from(elf_path);
    let dtb_path: &str = "./test-files-laputa/vmlinux.dtb";
    vm_config.dtb_path = String::from(dtb_path);
    let initrd_path: &str = "./test-files-laputa/rootfs-vm.img";
    vm_config.initrd_path = String::from(initrd_path);
    vm_config.mmio_regions.pop();
    vm_config.mmio_regions.pop();
    vm_config.mmio_regions.pop();

    const TEST_MMIO_REGION_1: GpaRegion = GpaRegion {
        gpa: 0x0,
        length: 0x10000,
    };

    vm_config.mmio_regions.push(TEST_MMIO_REGION_1);
    vm_config.vcpu_count = 1;

    laputa::run(vm_config);
}
