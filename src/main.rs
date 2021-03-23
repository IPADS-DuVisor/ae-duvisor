use std::process;
use std::fs;
use std::error::Error;
use laputa::Config;

#[macro_use]
extern crate clap;
use clap::App;

fn main() -> Result<(), Box<dyn Error>>{
    let mut vm_config = Config {
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
        let vm_config_contents = fs::read_to_string(vm_config_path)?;

        if let Err(e) = laputa::parse_vm_config_file(&vm_config_contents,
                                                      &vm_config) {
            eprintln!("Failed to parse vm config file: {}", e);
            process::exit(1);
        }
    } else { // We get VM arguments from command line.
        // Get machine type 
        if matches.is_present("machine") {
            vm_config.machine_type = matches.value_of("machine").unwrap().to_string();
        }

        // Get vcpu count
        vm_config.vcpu_count = value_t!(matches.value_of("smp"), u32).unwrap_or(0);
        if vm_config.vcpu_count == 0 {
            eprintln!("Error: please provide vcpu count by using --smp or config files.");
            process::exit(1);
        }

        // Get memory size
        vm_config.mem_size = value_t!(matches.value_of("memory"), u32).unwrap_or(0);
        if vm_config.mem_size == 0 {
            eprintln!("Error: please provide memory size by using --memory or config files.");
            process::exit(1);
        }

        // Get kernel_image_path 
        if matches.is_present("kernel") {
            vm_config.kernel_img_path = matches.value_of("kernel").unwrap().to_string();
        } else {
            eprintln!("Error: please provide kernel image by using --kernel or config files.");
            process::exit(1);
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


    Ok(())
}
