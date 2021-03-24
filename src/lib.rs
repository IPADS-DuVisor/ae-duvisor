#[macro_use]
extern crate clap;

pub mod init;

use init::cmdline;

pub fn run(config: &cmdline::VMConfig) {
    println!("running... {}", config.machine_type);
}
