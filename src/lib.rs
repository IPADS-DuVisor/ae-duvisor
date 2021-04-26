#![feature(llvm_asm)]
#![feature(global_asm)]

#[macro_use]
extern crate clap;

#[macro_use]
extern crate rusty_fork;

pub mod vm;
mod vcpu;
mod mm;
mod irq;
mod plat;
use vm::VirtualMachine;

pub mod init;

use init::cmdline;

pub fn run(config: &cmdline::VMConfig) {
    // TODO: assume everything else for laputa init has been finished
    let vcpu_num = config.vcpu_count;

    let mut vm = VirtualMachine::new(vcpu_num);
    vm.vm_init();
    vm.vm_run();
    vm.vm_destroy();
    println!("Finish vm running...");
}
