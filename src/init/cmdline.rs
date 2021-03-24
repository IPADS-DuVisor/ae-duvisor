use std::fs;
use std::path::Path;
use colored::*;

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

        let yaml = load_yaml!("../clap_config.yml");
        let matches = App::from_yaml(yaml).get_matches();

        // We get VM arguments from vm_config file. 
        if matches.is_present("vm_config") {
            let vm_config_path = matches.value_of("vm_config").unwrap().to_string();
            println!("vm_config_path = {}", vm_config_path);
            let vm_config_contents = match fs::read_to_string(vm_config_path) {
                Ok(contents) => contents,
                Err(_e) => return Err("Failed to read vm_config file"),
            };

            if !VMConfig::parse_vm_config_file(&vm_config_contents,
                                                         &mut vm_config) {
                return Err("Failed to parse vm config file!");
            }
        } else { // We get VM arguments from command line.
            // Get machine type 
            if matches.is_present("machine") {
                vm_config.machine_type = matches.value_of("machine").unwrap().to_string();
            }

            // Get vcpu count
            vm_config.vcpu_count = value_t!(matches.value_of("smp"), u32).unwrap_or(0);
            if vm_config.vcpu_count == 0 {
                return Err("please set vcpu count by using --smp or config files.");
            }

            // Get memory size
            vm_config.mem_size = value_t!(matches.value_of("memory"), u32).unwrap_or(0);
            if vm_config.mem_size == 0 {
                return Err("please set memory size by using --memory or config files.");
            }

            // Get kernel_image_path 
            if matches.is_present("kernel") {
                vm_config.kernel_img_path = matches.value_of("kernel").unwrap().to_string();
            } else {
                return Err("please set kernel image by using --kernel or config files.");
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
                                vm_config: &mut VMConfig) -> bool {
        for line in contents.lines() {
            let words = line.split("=").collect::<Vec<&str>>();
            match words[0].trim() {
                "smp" => {
                    if words.len() >= 2 {
                        vm_config.vcpu_count = words[1].trim().to_string().parse::<u32>().unwrap_or(0);
                    }
                }
                "memory" => {
                    if words.len() >= 2 {
                        vm_config.mem_size = words[1].trim().to_string().parse::<u32>().unwrap_or(0);
                    }
                }
                "kernel" => {
                    if words.len() >= 2 {
                        vm_config.kernel_img_path = words[1].trim().to_string();
                    }
                }
                "initrd" => {
                    if words.len() >= 2 {
                        vm_config.initrd_path = words[1].trim().to_string();
                    }
                }
                "dtb" => {
                    if words.len() >= 2 {
                        vm_config.dtb_path = words[1].trim().to_string();
                    }
                }
                "machine" => {
                    if words.len() >= 2 {
                        vm_config.machine_type = words[1].trim().to_string();
                    }
                }
                _ => {
                    vm_config.vcpu_count = 0;
                    vm_config.mem_size = 0;
                    vm_config.machine_type = String::from("");
                    vm_config.kernel_img_path = String::from("");
                    vm_config.initrd_path = String::from("");
                    vm_config.dtb_path = String::from("");

                    eprintln!("{} failed to parse argument {}",
                              "error:".bright_red(), words[0]);
                    return false;
                }
            }
        }

        true
    }

    /*
     * Check whether arguments in vm_config are legal or not.
     */
    pub fn verify_args(vm_config: &VMConfig) -> bool {
        if vm_config.vcpu_count == 0 || vm_config.vcpu_count > 8 {
            eprintln!("{} failed to set vcpu_count", "error:".bright_red());
            return false;
        }

        if vm_config.mem_size == 0 || vm_config.mem_size > 4096 {
            eprintln!("{} failed to set memory size", "error:".bright_red());
            return false;
        }

        if vm_config.machine_type != "laputa_virt" && vm_config.machine_type != "test_type" {
            eprintln!("{} failed to set machine_type for {}",
                      "error:".bright_red(), vm_config.machine_type);
            return false;
        }

        if !Path::new(&vm_config.kernel_img_path).is_file() {
            eprintln!("{} failed to open kernel file {}",
                      "error:".bright_red(), vm_config.kernel_img_path);
            return false;
        }

        if vm_config.initrd_path.len() != 0 {
            if !Path::new(&vm_config.initrd_path).is_file() {
                eprintln!("{} failed to open initrd file {}",
                          "error:".bright_red(), vm_config.initrd_path);
                return false;
            }
        }

        if vm_config.dtb_path.len() != 0 {
            if !Path::new(&vm_config.dtb_path).is_file() {
                eprintln!("{} failed to open dtb file {}",
                          "error:".bright_red(), vm_config.dtb_path);
                return false;
            }
        }

        true
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn setup_vm_config(vcpu: u32, mem: u32,
                       machine: &str, kernel: &str,
                       initrd: &str, dtb: &str) -> VMConfig {
        VMConfig {
            vcpu_count: vcpu,
            mem_size: mem,
            machine_type: String::from(machine),
            kernel_img_path: String::from(kernel),
            initrd_path: String::from(initrd),
            dtb_path: String::from(dtb),
        }
    }

    #[test]
    fn test_verify_args_normal() {
        let vm_config = setup_vm_config(2, 20, "laputa_virt",
                                    "unitestfiles/unitest_kernel",
                                    "", "");

        assert_eq!(VMConfig::verify_args(&vm_config), true);
    }

    #[test]
    fn test_verify_args_vcpu_count_large_value() {
        let vm_config = setup_vm_config(1024, 20, "laputa_virt",
                                    "unitestfiles/unitest_kernel",
                                    "", "");

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_vcpu_count_zero() {
        let vm_config = setup_vm_config(0, 20, "laputa_virt",
                                    "unitestfiles/unitest_kernel",
                                    "", "");

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_mem_large_value() {
        let vm_config = setup_vm_config(4, 5000, "laputa_virt",
                                    "unitestfiles/unitest_kernel",
                                    "", "");

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_mem_zero() {
        let vm_config = setup_vm_config(4, 0, "laputa_virt",
                                    "unitestfiles/unitest_kernel",
                                    "", "");

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_type_invalid() {
        let vm_config = setup_vm_config(4, 1024, "laputa_virt2",
                                    "unitestfiles/unitest_kernel",
                                    "", "");

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_kernel_img_not_exist() {
        let vm_config = setup_vm_config(4, 1024, "laputa_virt",
                                    "err_unitest_kernel",
                                    "", "");

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_initrd_invalid() {
        let vm_config = setup_vm_config(4, 1024, "laputa_virt",
                                    "unitestfiles/unitest_kernel",
                                    "err_initrd", "");

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_verify_args_dtb_invalid() {
        let vm_config = setup_vm_config(4, 1024, "laputa_virt",
                                    "unitestfiles/unitest_kernel",
                                    "", "err_dtb");

        assert_eq!(VMConfig::verify_args(&vm_config), false);
    }

    #[test]
    fn test_parse_vm_config_file_normal() {
        let mut vm_config = setup_vm_config(0, 0, "", "", "", "");

        let contents_str = "smp = 3\r\nmemory = 320\r\nkernel = kernel.file\r\n\
            initrd = initrd.file\r\ndtb = dtb.file\r\nmachine = test_type";
        let contents = String::from(contents_str);
        assert_eq!(true,
                   VMConfig::parse_vm_config_file(&contents, &mut vm_config));

        assert_eq!(vm_config.vcpu_count, 3);
        assert_eq!(vm_config.mem_size, 320);
        assert_eq!(vm_config.kernel_img_path, "kernel.file");
        assert_eq!(vm_config.machine_type, "test_type");
        assert_eq!(vm_config.initrd_path, "initrd.file");
        assert_eq!(vm_config.dtb_path, "dtb.file");
    }

    #[test]
    fn test_parse_vm_config_file_smp_invalid() {
        let mut vm_config = setup_vm_config(0, 0, "", "", "", "");

        let contents_str = "smp = asd\r\nmemory = 320\r\nkernel = kernel.file\r\n\
            initrd = initrd.file\r\ndtb = dtb.file\r\nmachine = test_type";
        let contents = String::from(contents_str);
        assert_eq!(true,
                   VMConfig::parse_vm_config_file(&contents, &mut vm_config));

        assert_eq!(vm_config.vcpu_count, 0);
        assert_eq!(vm_config.mem_size, 320);
        assert_eq!(vm_config.kernel_img_path, "kernel.file");
        assert_eq!(vm_config.machine_type, "test_type");
        assert_eq!(vm_config.initrd_path, "initrd.file");
        assert_eq!(vm_config.dtb_path, "dtb.file");
    }

    #[test]
    fn test_parse_vm_config_file_smp_empty() {
        let mut vm_config = setup_vm_config(0, 0, "", "", "", "");

        let contents_str = "smp =\r\nmemory = 320\r\nkernel = kernel.file\r\n\
            initrd = initrd.file\r\ndtb = dtb.file\r\nmachine = test_type";
        let contents = String::from(contents_str);
        assert_eq!(true,
                   VMConfig::parse_vm_config_file(&contents, &mut vm_config));

        assert_eq!(vm_config.vcpu_count, 0);
        assert_eq!(vm_config.mem_size, 320);
        assert_eq!(vm_config.kernel_img_path, "kernel.file");
        assert_eq!(vm_config.machine_type, "test_type");
        assert_eq!(vm_config.initrd_path, "initrd.file");
        assert_eq!(vm_config.dtb_path, "dtb.file");
    }

    #[test]
    fn test_parse_vm_config_file_smp_no_equalsymbol() {
        let mut vm_config = setup_vm_config(0, 0, "", "", "", "");

        let contents_str = "smp\r\nmemory = 320\r\nkernel = kernel.file\r\n\
            initrd = initrd.file\r\ndtb = dtb.file\r\nmachine = test_type";
        let contents = String::from(contents_str);
        assert_eq!(true,
                   VMConfig::parse_vm_config_file(&contents, &mut vm_config));

        assert_eq!(vm_config.vcpu_count, 0);
        assert_eq!(vm_config.mem_size, 320);
        assert_eq!(vm_config.kernel_img_path, "kernel.file");
        assert_eq!(vm_config.machine_type, "test_type");
        assert_eq!(vm_config.initrd_path, "initrd.file");
        assert_eq!(vm_config.dtb_path, "dtb.file");
    }

    #[test]
    fn test_parse_vm_config_file_memory_invalid() {
        let mut vm_config = setup_vm_config(0, 0, "", "", "", "");

        let contents_str = "smp = 3\r\nmemory = asdas\r\nkernel = kernel.file\r\n\
            initrd = initrd.file\r\ndtb = dtb.file\r\nmachine = test_type";
        let contents = String::from(contents_str);
        assert_eq!(true,
                   VMConfig::parse_vm_config_file(&contents, &mut vm_config));

        assert_eq!(vm_config.vcpu_count, 3);
        assert_eq!(vm_config.mem_size, 0);
        assert_eq!(vm_config.kernel_img_path, "kernel.file");
        assert_eq!(vm_config.machine_type, "test_type");
        assert_eq!(vm_config.initrd_path, "initrd.file");
        assert_eq!(vm_config.dtb_path, "dtb.file");
    }

    #[test]
    fn test_parse_vm_config_file_memory_emptry() {
        let mut vm_config = setup_vm_config(0, 0, "", "", "", "");

        let contents_str = "smp = 3\r\nmemory =\r\nkernel = kernel.file\r\n\
            initrd = initrd.file\r\ndtb = dtb.file\r\nmachine = test_type";
        let contents = String::from(contents_str);
        assert_eq!(true,
                   VMConfig::parse_vm_config_file(&contents, &mut vm_config));

        assert_eq!(vm_config.vcpu_count, 3);
        assert_eq!(vm_config.mem_size, 0);
        assert_eq!(vm_config.kernel_img_path, "kernel.file");
        assert_eq!(vm_config.machine_type, "test_type");
        assert_eq!(vm_config.initrd_path, "initrd.file");
        assert_eq!(vm_config.dtb_path, "dtb.file");
    }

    #[test]
    fn test_parse_vm_config_file_memory_no_equalsymbol() {
        let mut vm_config = setup_vm_config(0, 0, "", "", "", "");

        let contents_str = "smp = 3\r\nmemory\r\nkernel = kernel.file\r\n\
            initrd = initrd.file\r\ndtb = dtb.file\r\nmachine = test_type";
        let contents = String::from(contents_str);
        assert_eq!(true,
                   VMConfig::parse_vm_config_file(&contents, &mut vm_config));

        assert_eq!(vm_config.vcpu_count, 3);
        assert_eq!(vm_config.mem_size, 0);
        assert_eq!(vm_config.kernel_img_path, "kernel.file");
        assert_eq!(vm_config.machine_type, "test_type");
        assert_eq!(vm_config.initrd_path, "initrd.file");
        assert_eq!(vm_config.dtb_path, "dtb.file");
    }

    #[test]
    fn test_parse_vm_config_file_multiple_values_in_one_line() {
        let mut vm_config = setup_vm_config(0, 0, "", "", "", "");

        let contents_str = "smp = 3\r\nmemory = 320\r\nkernel = kernel.file = two = three\r\n\
            initrd = initrd.file\r\ndtb = dtb.file\r\nmachine = test_type";
        let contents = String::from(contents_str);
        assert_eq!(true,
                   VMConfig::parse_vm_config_file(&contents, &mut vm_config));

        assert_eq!(vm_config.vcpu_count, 3);
        assert_eq!(vm_config.mem_size, 320);
        assert_eq!(vm_config.kernel_img_path, "kernel.file");
        assert_eq!(vm_config.machine_type, "test_type");
        assert_eq!(vm_config.initrd_path, "initrd.file");
        assert_eq!(vm_config.dtb_path, "dtb.file");
    }

    #[test]
    fn test_parse_vm_config_file_long_string_value() {
        let mut vm_config = setup_vm_config(0, 0, "", "", "", "");

        let contents_str = "smp = 3\r\nmemory = 320\r\nkernel = kernel.file\r\n\
            initrd = initrd.fileeeeeeeeeeeeeeeeeeeeeeeeee\ndtb = dtb.file\r\nmachine = test_type";
        let contents = String::from(contents_str);
        assert_eq!(true,
                   VMConfig::parse_vm_config_file(&contents, &mut vm_config));

        assert_eq!(vm_config.vcpu_count, 3);
        assert_eq!(vm_config.mem_size, 320);
        assert_eq!(vm_config.kernel_img_path, "kernel.file");
        assert_eq!(vm_config.machine_type, "test_type");
        assert_eq!(vm_config.initrd_path, "initrd.fileeeeeeeeeeeeeeeeeeeeeeeeee");
        assert_eq!(vm_config.dtb_path, "dtb.file");
    }

    #[test]
    fn test_parse_vm_config_file_invalid_arg() {
        let mut vm_config = setup_vm_config(0, 0, "", "", "", "");

        let contents_str = "smp = 3\r\nmemory = 320\r\ninvalid = invalid\r\nkernel = kernel.file\r\n\
            initrd = initrd.file\ndtb = dtb.file\r\nmachine = test_type\r\n";
        let contents = String::from(contents_str);
        assert_eq!(false,
                   VMConfig::parse_vm_config_file(&contents, &mut vm_config));

        assert_eq!(vm_config.vcpu_count, 0);
        assert_eq!(vm_config.mem_size, 0);
        assert_eq!(vm_config.kernel_img_path, "");
        assert_eq!(vm_config.machine_type, "");
        assert_eq!(vm_config.initrd_path, "");
        assert_eq!(vm_config.dtb_path, "");
    }
}
