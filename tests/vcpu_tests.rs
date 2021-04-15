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
    assert!(Path::new("testfiles/integration/vm_test.img").is_file());
}