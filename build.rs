#[path = "preparefile.rs"] mod preparefile;

use preparefile::*;

extern crate cc;


fn main() {
    // Prepare /guestentry/asm_offset.h
    prepare_asm_offset_header();

    cc::Build::new()
        .file("guestentry/enter_guest.S")
        .compile("enter_guest");
}