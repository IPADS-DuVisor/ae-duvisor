use std::process;
use colored::*;
use laputa::init::cmdline;

#[allow(unused)]
extern "C"
{
    fn getchar_emulation() -> i32;
}

fn main() {
    unsafe {
        getchar_emulation();
    }
    let vm_config = cmdline::VMConfig::new().unwrap_or_else(|err| {
        eprintln!("{}: {}", "error".bright_red(), err);
        process::exit(1);
    });

    if !cmdline::VMConfig::verify_args(&vm_config) {
        process::exit(1);
    }

    laputa::run(vm_config);
}
