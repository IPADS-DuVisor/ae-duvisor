#[path = "preparefile.rs"] mod preparefile;

use preparefile::*;


fn main() {
    println!("cargo:warning=------------build.rs start!-------------");

    // Prepare src/vcpu/asm_offset.S
    prepare_asm_offset_file();

    // Prepare src/vcpu/asm_csr.S
    prepare_asm_csr_file();

    // Prepare src/vcpu/asm_switch.S
    prepare_asm_switch_file();

    println!("cargo:warning=------------build.rs end!---------------");
}