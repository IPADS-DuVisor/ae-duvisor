#[cfg(test)]
mod tests {
    #[test]
    fn unit_test_example() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_simple_add() {
        use crate::simple_add;
        assert_eq!(5, simple_add(2, 3));
    }
}

pub fn simple_add(value1: i32, value2 :i32) -> i32 {
    value1 + value2
}

use std::error::Error;
use std::path::Path;

pub struct Config {
    pub vcpu_count: u32,
    pub mem_size: u32,
    pub machine_type: String,
    pub kernel_img_path: String,
    pub initrd_path: String,
    pub dtb_path: String,
}

/*
 * Parsing vm configs from the config file
 * All existing arguments in vm_config struct will be overwritten.
 */
pub fn parse_vm_config_file(contents: &String,
                            vm_config: &Config) -> Result<(), &'static str>{

    Ok(())
}

/*
 * Check whether arguments in vm_config is legal or not.
 */
fn verify_args(vm_config: &Config) -> bool {

    true
}

pub fn run(config: Config) -> Result<(), &'static str> {
    if verify_args(&config) {
        return Err("Encounter illegal arguments");
    }

    Ok(())
}
