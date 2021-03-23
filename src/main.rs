use std::process;
use std::error::Error;
use laputa::VMConfig;

fn main() -> Result<(), Box<dyn Error>>{
    let vm_config = VMConfig::new().unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });

    Ok(())
}
