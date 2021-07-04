#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(pub_macro_rules)]
#![feature(once_cell)]
#[allow(unused_imports)]

#[macro_use]
extern crate clap;

pub mod vm;
pub mod vcpu;
pub mod mm;
pub mod irq;
pub mod plat;
pub mod debug;
pub mod devices;
use vm::virtualmachine::VirtualMachine;

pub mod init;

use init::cmdline;

pub fn run(config: cmdline::VMConfig) {
    let mut vm = VirtualMachine::new(config);
    let ret = vm.vm_init();

    if ret.len() == 0 {
        /* No kernel data has been loaded */
        panic!("VM init failed");
    }

    vm.vm_run();

    vm.vm_destroy();

    println!("Finish vm running...");
}
