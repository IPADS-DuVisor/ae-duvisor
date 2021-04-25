#[path = "preparefile.rs"] mod preparefile;

use preparefile::*;

extern crate cc;


fn main() {
    println!("cargo:warning=------------build.rs start-------------");

    // Prepare /guestentry/asm_offset.h
    prepare_asm_offset_header();

    cc::Build::new()
        .file("guestentry/guest_entry.c")
        .file("guestentry/enter_guest.S")
        .compile("enter_guest");

    println!("cargo:warning=------------build.rs end---------------");
}