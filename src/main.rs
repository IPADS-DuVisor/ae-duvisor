use std::process;
use laputa::VMConfig;
use colored::*;

fn main() {
    let vm_config = VMConfig::new().unwrap_or_else(|err| {
        eprintln!("{}: {}", "error".bright_red(), err);
        process::exit(1);
    });

    if !VMConfig::verify_args(&vm_config) {
        process::exit(1);
    } 

    laputa::run(&vm_config); 
}
