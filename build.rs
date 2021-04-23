#[path = "preparefile.rs"] mod preparefile;

use preparefile::*;

extern crate cc;


fn main() {
    println!("cargo:warning=------------build.rs start-------------");

    prepare_asm_offset_header();

    //cc::Build::new()
    //    .file("guestentry/foo.c")
    //    .flag_if_supported("-Wall")
    //    .flag_if_supported("-Werror")
    //    .pic(true)
    //    .shared_flag(true)
    //    .compile("guestentry/libfoo1.so");

    // gcc -c -Wall -Werror -fpic guestentry/foo.c
    /* cc::Build::new()
        .file("guestentry/foo.c")
        .flag_if_supported("-Wall")
        .flag_if_supported("-Werror")
        .pic(true)
        .shared_flag(true)
        .compile("foo"); */

        cc::Build::new()
        .file("guestentry/guest_entry.c")
        .file("guestentry/enter_guest.S")
        .compile("enter_guest");

    //println!("cargo:rustc-link-search=native=./lib");

    // Prepare src/vcpu/asm_offset.S
    prepare_asm_offset_file();

    // Prepare src/vcpu/asm_csr.S
    prepare_asm_csr_file();

    // Prepare src/vcpu/asm_switch.S
    prepare_asm_switch_file();

    println!("cargo:warning=------------build.rs end---------------");
}