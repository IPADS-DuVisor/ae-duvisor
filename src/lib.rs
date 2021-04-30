#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(pub_macro_rules)]
#[allow(unused_imports)]

#[macro_use]
extern crate clap;



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
    let mem_size = config.mem_size;

    let mut vm = VirtualMachine::new(vcpu_num, mem_size);
    vm.vm_init();
    vm.vm_run();
    vm.vm_destroy();
    println!("Finish vm running...");
}
