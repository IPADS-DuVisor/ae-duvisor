use std::process;
use laputa::VMConfig;

fn main() {
    let vm_config = VMConfig::new().unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });


    if VMConfig::verify_args(&vm_config) {
       laputa::run(&vm_config); 
    }
}
