#[macro_use]
extern crate clap;

mod vm;
mod vcpu;
use vm::VirtualMachine;

pub mod init;

use init::cmdline;

pub fn run(config: &cmdline::VMConfig) {
    // TODO: assume everything else for laputa init has been finished
    let vcpu_num = config.vcpu_count;

    let mut vm = VirtualMachine::new(vcpu_num);
    vm.vm_run();
    println!("Finish vm running...");
}
