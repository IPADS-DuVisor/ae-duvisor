#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(pub_macro_rules)]
#[allow(unused_imports)]

#[macro_use]
extern crate clap;

pub mod vm;
pub mod vcpu;
pub mod mm;
pub mod irq;
pub mod plat;
pub mod debug;
use vm::virtualmachine::VirtualMachine;

pub mod init;

use init::cmdline;

pub fn run(config: cmdline::VMConfig) {
    let mut vm = VirtualMachine::new(config);
    vm.vm_init();
    vm.vm_run();
    vm.vm_destroy();
    println!("Finish vm running...");
}
