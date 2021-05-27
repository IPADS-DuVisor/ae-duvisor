/*
 * This file contains integration tests for checking whether
 * Laputa can run small VMs that only maniputaes registers.
 */
use std::path::Path;
use laputa::init::cmdline;

#[test]
fn test_vcpu_add_all_gprs() {
    let vm_config = cmdline::VMConfig{
        vcpu_count: 1,
        mem_size: 1024,
        machine_type: String::from("test_type"),
        kernel_img_path: String::from("tests/integration/vcpu_add_all_gprs.img"),
        initrd_path: String::from(""),
        dtb_path: String::from(""),
        mmio_regions: Vec::new(),
    };
    assert!(cmdline::VMConfig::verify_args(&vm_config));

    // TODO: VirtualMachine::new, there should be assert codes. When vm initialization failure,
    // such as memory allocation failure or exceeding vm number failure, occurs, 
    // VirtualMachine::new should assert ABORT, and the tests will failed.

    // TODO: use constants to specify gpr
    // TODO: laputa::set_one_gregs(vcpu_num: u64, gpr_num: u64, val: u64)

    // TODO: use constants to specify exit reason
    // TODO: laputa::get_exit_reason(vcpu_num: u64), get exit reason of vcpu with number vcpu_num

    // TODO: use constants to specify gpr
    // TODO: laputa::get_one_greg(vcpu_num: u64, gpr_num: u64)

    assert_eq!(0, 0);
}

#[test]
fn test_generated_images_existence() {
    assert!(Path::new("tests/integration/vcpu_add_all_gprs.img").is_file());
}
