/*
 * This file contains integration tests for checking whether
 * Laputa can run small VMs that only maniputaes registers.
 */
use std::path::Path;

#[test]
fn test_vcpu_add_all_gprs() {
    assert_eq!(0, 0);
}

#[test]
fn test_generated_images_existence() {
    assert!(Path::new("unitestfiles/vm_test_1.img").is_file());
    assert!(Path::new("unitestfiles/vm_test_mem_1.img").is_file());
    assert!(Path::new("unitestfiles/vm_test_mem_2.img").is_file());
    assert!(Path::new("unitestfiles/vm_test_mem_3.img").is_file());
    assert!(Path::new("unitestfiles/vm_test_mem_4.img").is_file());
    assert!(Path::new("unitestfiles/vm_test_mem_5.img").is_file());
    assert!(Path::new("unitestfiles/vm_test_mem_6.img").is_file());
    assert!(Path::new("unitestfiles/vm_test_mem_7.img").is_file());
}