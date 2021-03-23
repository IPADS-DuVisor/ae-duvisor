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

use std::fs;

#[macro_use]
extern crate clap;
use clap::App;

pub struct VMConfig {
    pub vcpu_count: u32,
    pub mem_size: u32,
    pub machine_type: String,
    pub kernel_img_path: String,
    pub initrd_path: String,
    pub dtb_path: String,
}

impl VMConfig {
    pub fn new() -> Result<VMConfig, &'static str> {
        let mut vm_config = VMConfig {
            vcpu_count: 0,
            mem_size: 0,
            machine_type: String::from(""),
            kernel_img_path: String::from(""),
            initrd_path: String::from(""),
            dtb_path: String::from(""),
        };

        let yaml = load_yaml!("clap_config.yml");
        let matches = App::from_yaml(yaml).get_matches();

        // We get VM arguments from vm_config file. 
        if matches.is_present("vm_config") {
            let vm_config_path = matches.value_of("vm_config").unwrap().to_string();
            println!("vm_config_path = {}", vm_config_path);
            let vm_config_contents = match fs::read_to_string(vm_config_path) {
                Ok(contents) => contents,
                Err(_e) => return Err("Failed to read vm_config file"),
            };

            if let Err(e) = parse_vm_config_file(&vm_config_contents,
                                                         &vm_config) {
                return Err(e);
            }
        } else { // We get VM arguments from command line.
            // Get machine type 
            if matches.is_present("machine") {
                vm_config.machine_type = matches.value_of("machine").unwrap().to_string();
            }

            // Get vcpu count
            vm_config.vcpu_count = value_t!(matches.value_of("smp"), u32).unwrap_or(0);
            if vm_config.vcpu_count == 0 {
                return Err("Error: please provide vcpu count by using --smp or config files.");
            }

            // Get memory size
            vm_config.mem_size = value_t!(matches.value_of("memory"), u32).unwrap_or(0);
            if vm_config.mem_size == 0 {
                return Err("Error: please provide memory size by using --memory or config files.");
            }

            // Get kernel_image_path 
            if matches.is_present("kernel") {
                vm_config.kernel_img_path = matches.value_of("kernel").unwrap().to_string();
            } else {
                return Err("Error: please provide kernel image by using --kernel or config files.");
            }

            // Get dtb_path 
            if matches.is_present("dtb") {
                vm_config.dtb_path = matches.value_of("dtb").unwrap().to_string()
            }

            // Get initrd_path 
            if matches.is_present("initrd") {
                vm_config.initrd_path = matches.value_of("initrd").unwrap().to_string();
            }
        }

        println!("machine_type = {}, smp = {}, mem_size = {}, kernel_img_path = {}, dtb_path = {}, initrd_path = {}",
                 vm_config.machine_type,
                 vm_config.vcpu_count, vm_config.mem_size,
                 vm_config.kernel_img_path, vm_config.dtb_path, vm_config.initrd_path);

        Ok(vm_config)
    }
}

/*
 * Parsing vm configs from the config file
 * All existing arguments in vm_config struct will be overwritten.
 */
pub fn parse_vm_config_file(contents: &String,
                            vm_config: &VMConfig) -> Result<(), &'static str>{
    Ok(())
}

/*
 * Check whether arguments in vm_config is legal or not.
 */
fn verify_args(vm_config: &VMConfig) -> bool {

    true
}

pub fn run(config: &VMConfig) -> Result<(), &'static str> {
    if verify_args(&config) {
        return Err("Encounter illegal arguments");
    }

    Ok(())
}
