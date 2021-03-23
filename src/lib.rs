use std::fs;
use std::path::Path;

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

            if let Err(e) = VMConfig::parse_vm_config_file(&vm_config_contents,
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

    /*
     * Parsing vm configs from the config file
     * All existing arguments in vm_config struct will be overwritten.
     */
    fn parse_vm_config_file(contents: &String,
                                vm_config: &VMConfig) -> Result<(), &'static str>{
        Ok(())
    }

    /*
     * Check whether arguments in vm_config are legal or not.
     */
    pub fn verify_args(vm_config: &VMConfig) -> bool {
        if vm_config.vcpu_count == 0 || vm_config.vcpu_count > 8 {
            return false;
        }

        if vm_config.mem_size == 0 || vm_config.mem_size > 4096 {
            return false;
        }

        if vm_config.machine_type != "laputa_virt" {
            return false;
        }

        if !Path::new(&vm_config.kernel_img_path).is_file() {
            return false;
        }

        if vm_config.initrd_path.len() != 0 {
            if !Path::new(&vm_config.initrd_path).is_file() {
                return false;
            }
        }

        if vm_config.dtb_path.len() != 0 {
            if !Path::new(&vm_config.dtb_path).is_file() {
                return false;
            }
        }

        true
    }
}

pub fn run(config: &VMConfig) {
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_args_normal() {
        let vm_config = VMConfig {
            vcpu_count: 2,
            mem_size: 20,
            machine_type: String::from("laputa_virt"),
            kernel_img_path: String::from("unitestfiles/unitest_kernel"),
            initrd_path: String::from(""),
            dtb_path: String::from(""),
        };

        assert_eq!(VMConfig::verify_args(&vm_config), true);
    }

    #[test]
    fn test_verify_args_vcpu_count_large_value() {
        let vm_config = VMConfig {
            vcpu_count: 1024,
            mem_size: 20,
            machine_type: String::from("laputa_virt"),
            kernel_img_path: String::from("unitestfiles/unitest_kernel"),
            initrd_path: String::from(""),
            dtb_path: String::from(""),
        };

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_vcpu_count_zero() {
        let vm_config = VMConfig {
            vcpu_count: 0,
            mem_size: 20,
            machine_type: String::from("laputa_virt"),
            kernel_img_path: String::from("unitestfiles/unitest_kernel"),
            initrd_path: String::from(""),
            dtb_path: String::from(""),
        };

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_mem_large_value() {
        let vm_config = VMConfig {
            vcpu_count: 4,
            mem_size: 5000,
            machine_type: String::from("laputa_virt"),
            kernel_img_path: String::from("unitestfiles/unitest_kernel"),
            initrd_path: String::from(""),
            dtb_path: String::from(""),
        };

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_mem_zero() {
        let vm_config = VMConfig {
            vcpu_count: 4,
            mem_size: 0,
            machine_type: String::from("laputa_virt"),
            kernel_img_path: String::from("unitestfiles/unitest_kernel"),
            initrd_path: String::from(""),
            dtb_path: String::from(""),
        };

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_type_invalid() {
        let vm_config = VMConfig {
            vcpu_count: 4,
            mem_size: 1024,
            machine_type: String::from("laputa_virt2"),
            kernel_img_path: String::from("unitestfiles/unitest_kernel"),
            initrd_path: String::from(""),
            dtb_path: String::from(""),
        };

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_kernel_img_not_exist() {
        let vm_config = VMConfig {
            vcpu_count: 4,
            mem_size: 1024,
            machine_type: String::from("laputa_virt"),
            kernel_img_path: String::from("err_unitest_kernel"),
            initrd_path: String::from(""),
            dtb_path: String::from(""),
        };

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_initrd_invalid() {
        let vm_config = VMConfig {
            vcpu_count: 4,
            mem_size: 1024,
            machine_type: String::from("laputa_virt"),
            kernel_img_path: String::from("unitestfiles/unitest_kernel"),
            initrd_path: String::from("err_initrd"),
            dtb_path: String::from(""),
        };

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_dtb_invalid() {
        let vm_config = VMConfig {
            vcpu_count: 4,
            mem_size: 1024,
            machine_type: String::from("laputa_virt"),
            kernel_img_path: String::from("unitestfiles/unitest_kernel"),
            initrd_path: String::from(""),
            dtb_path: String::from("err_dtb"),
        };

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }
}
