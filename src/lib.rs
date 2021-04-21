#![feature(llvm_asm)]
#![feature(global_asm)]
#[macro_use]
extern crate clap;

pub mod vm;
mod vcpu;
mod mm;
mod irq;
use vm::VirtualMachine;

pub mod init;

use init::cmdline;

pub fn run(config: &cmdline::VMConfig) {
    // TODO: assume everything else for laputa init has been finished
    let vcpu_num = config.vcpu_count;

    let mut vm = VirtualMachine::new(vcpu_num);
    vm.vm_run();
    vm.vm_destory();
    println!("Finish vm running...");
}
