/*
 * This file contains integration tests for checking whether
 * Laputa can run small VMs that only maniputaes registers.
 */
use std::path::Path;
use laputa::init::cmdline;
use laputa::vm::virtualmachine;
use laputa::debug::utils::configtest::test_vm_config_create;
use rusty_fork::rusty_fork_test;
use laputa::mm::gstagemmu::gsmmu_constants::*;
use libc::c_void;
use laputa::mm::utils::*;

#[test]
fn test_vcpu_add_all_gprs() {
    let vm_config = cmdline::VMConfig{
        vcpu_count: 1,
        mem_size: 1024,
        machine_type: String::from("test_type"),
        kernel_img_path: String::from("tests/integration/vcpu_add_all_gprs.img"),
        initrd_path: String::from(""),
        dtb_path: String::from(""),
        mmio_regions: Vec::new(),
    };
    assert!(cmdline::VMConfig::verify_args(&vm_config));

    // TODO: VirtualMachine::new, there should be assert codes. When vm initialization failure,
    // such as memory allocation failure or exceeding vm number failure, occurs, 
    // VirtualMachine::new should assert ABORT, and the tests will failed.

    // TODO: use constants to specify gpr
    // TODO: laputa::set_one_gregs(vcpu_num: u64, gpr_num: u64, val: u64)

    // TODO: use constants to specify exit reason
    // TODO: laputa::get_exit_reason(vcpu_num: u64), get exit reason of vcpu with number vcpu_num

    // TODO: use constants to specify gpr
    // TODO: laputa::get_one_greg(vcpu_num: u64, gpr_num: u64)

    assert_eq!(0, 0);
}

#[test]
fn test_generated_images_existence() {
    assert!(Path::new("tests/integration/vcpu_add_all_gprs.img").is_file());
}

rusty_fork_test! {
    #[test]
    fn test_vm_add_all_gprs() { 
        println!("---------start vm------------");
        let sum_ans = 10;
        let mut sum = 0;
        let mut vm_config = test_vm_config_create();
        let elf_path: &str = "./tests/integration/vcpu_add_all_gprs.img";
        vm_config.kernel_img_path = String::from(elf_path);
        let mut vm = virtualmachine::VirtualMachine::new(vm_config);
        
        vm.vm_init();

        let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

        vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
            = entry_point;

        vm.vm_run();
            
        sum += vm.vcpus[0].lock().unwrap().vcpu_ctx.guest_ctx.gp_regs.x_reg[10];

        vm.vm_destroy();

        assert_eq!(sum, sum_ans);
    }

    #[test]
    fn test_vmem_ro() { 
        let exit_reason_ans = 2; // g-stage page fault for no permission
        let exit_reason;
        let mut vm_config = test_vm_config_create();
        let elf_path: &str = "./tests/integration/vmem_W_Ro.img";
        vm_config.kernel_img_path = String::from(elf_path);
        let mut vm = virtualmachine::VirtualMachine::new(vm_config);

        vm.vm_init();

        let ro_address = 0x3000;

        let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

        let res = vm.vm_state.lock().unwrap()
            .gsmmu.gpa_block_add(ro_address, PAGE_SIZE);
        if !res.is_ok() {
            panic!("gpa region add failed!")
        }

        let (_hva, hpa) = res.unwrap();
        let mut flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
            | PTE_EXECUTE;

        vm.vm_state.lock().unwrap().gsmmu.map_page(ro_address, hpa, flag);

        // read-only
        flag = PTE_USER | PTE_VALID | PTE_READ;
        vm.vm_state.lock().unwrap().gsmmu.map_protect(ro_address, flag);

        vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
            = entry_point;
            
        vm.vm_run();
            
        exit_reason = vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.gp_regs
            .x_reg[0];
        vm.vm_destroy();

        assert_eq!(exit_reason, exit_reason_ans);
    }

    #[test]
    fn test_vmem_nx() { 
        let exit_reason_ans = 2; // g-stage page fault for no permission
        let exit_reason;
        let mut vm_config = test_vm_config_create();
        let elf_path: &str = "./tests/integration/vmem_X_nonX.img";
        vm_config.kernel_img_path = String::from(elf_path);
        let mut vm = virtualmachine::VirtualMachine::new(vm_config);

        vm.vm_init();

        let nx_address = 0x3000;

        let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

        let res = vm.vm_state.lock().unwrap()
            .gsmmu.gpa_block_add(nx_address, PAGE_SIZE);
        if !res.is_ok() {
            panic!("gpa region add failed!")
        }

        let (_hva, hpa) = res.unwrap();
        let mut flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE
            | PTE_EXECUTE;

        vm.vm_state.lock().unwrap().gsmmu.map_page(nx_address, hpa, flag);

        // non-execute
        flag = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE;
        vm.vm_state.lock().unwrap().gsmmu.map_protect(nx_address, flag);

        vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
            = entry_point;
        
        vm.vm_run();
        
        exit_reason = vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.gp_regs
        .x_reg[0];

        vm.vm_destroy();

        assert_eq!(exit_reason, exit_reason_ans);
    }

    /* check the correctness of loading data from specific gpa */
    #[test]
    fn test_vmem_ld_data() { 
        let load_value;
        let mut vm_config = test_vm_config_create();
        let elf_path: &str = "./tests/integration/vmem_ld_data.img";
        vm_config.kernel_img_path = String::from(elf_path);
        let mut vm = virtualmachine::VirtualMachine::new(vm_config);

        /* Answer will be saved at 0x3000(gpa) */
        let answer: u64 = 0x1213141516171819;

        vm.vm_init();

        let target_address = 0x3000;

        /* set entry point */
        let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

        let res = vm.vm_state.lock().unwrap()
            .gsmmu.gpa_block_add(target_address, PAGE_SIZE);
        if !res.is_ok() {
            panic!("gpa region add failed!")
        }

        let (hva, hpa) = res.unwrap();
        println!("hva {:x}, hpa {:x}", hva, hpa);

        unsafe {
            *(hva as *mut u64) = answer;
        }

        let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
            | PTE_EXECUTE;

        vm.vm_state.lock().unwrap().gsmmu.map_page(target_address, hpa, 
            flag);

        vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
            = entry_point;
            
        vm.vm_run();
            
        load_value = vm.vcpus[0].lock().unwrap().vcpu_ctx.guest_ctx.gp_regs
            .x_reg[5];

        vm.vm_destroy();

        println!("load value {:x}", load_value);

        assert_eq!(load_value, answer);
    }

    #[test]
    fn test_vmem_mapping() { 
        let exit_reason_ans = 0xdead;
        let exit_reason;
        let mut vm_config = test_vm_config_create();
        let elf_path: &str = "./tests/integration/vmem_ld_mapping.img";
        vm_config.kernel_img_path = String::from(elf_path);
        let mut vm = virtualmachine::VirtualMachine::new(vm_config);

        vm.vm_init();

        let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

        vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
            = entry_point;

        vm.vm_run();
        
        exit_reason = vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.gp_regs
            .x_reg[0];
        println!("exit reason {:x}", exit_reason);

        vm.vm_destroy();

        assert_eq!(exit_reason, exit_reason_ans);
    }

    #[test]
    fn test_vm_huge_mapping() { 
        println!("---------start test_vm_huge_mapping------------");
        let exit_reason_ans = 0xdead;
        let exit_reason;
        let mut vm_config = test_vm_config_create();

        // cancel the three mmio regions
        vm_config.mmio_regions = Vec::new();

        let elf_path: &str 
            = "./tests/integration/vmem_ld_sd_over_loop.img";
        vm_config.kernel_img_path = String::from(elf_path);

        let mut vm = virtualmachine::VirtualMachine::new(vm_config);

        vm.vm_init();

        let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

        vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
            = entry_point;

        vm.vm_run();
        
        exit_reason = vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.gp_regs
            .x_reg[0];
        println!("exit reason {:x}", exit_reason);

        vm.vm_destroy();

        assert_eq!(exit_reason_ans, exit_reason);
    }

    #[test]
    fn test_vm_ld_sd_sum() { 
        println!("---------start test_vm_huge_mapping------------");
        let mut sum_ans = 0;
        let sum;
        let mut vm_config = test_vm_config_create();

        // cancel the three mmio regions
        vm_config.mmio_regions = Vec::new();
        
        let elf_path: &str = "./tests/integration/vmem_ld_sd_sum.img";
        vm_config.kernel_img_path = String::from(elf_path);

        let mut vm = virtualmachine::VirtualMachine::new(vm_config);

        /* sum up 0..100 twice */
        for i in 0..100 {
            sum_ans += i;
        }
        sum_ans *= 2;

        vm.vm_init();

        let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

        vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
            = entry_point;

        vm.vm_run();
        
        sum = vm.vcpus[0].lock().unwrap().vcpu_ctx.guest_ctx.gp_regs
            .x_reg[29];
        println!("sum {}", sum);

        vm.vm_destroy();

        assert_eq!(sum_ans, sum);
    }

    #[test]
    fn test_ecall_putchar() { 
        let mut vm_config = test_vm_config_create();
        let elf_path: &str = "./tests/integration/opensbi_putchar.img";
        vm_config.kernel_img_path = String::from(elf_path);
        let mut vm = virtualmachine::VirtualMachine::new(vm_config);

        vm.vm_init();

        let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

        vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
            = entry_point;

        vm.vm_run();

        let t0: u64;
        let t1: u64;

        // Sum up the chars in "Hello Ecall\n"
        let t0_ans: u64 = 1023;

        // all the ecall should should return 0 
        let t1_ans: u64 = 0;

        t0 = vm.vcpus[0].lock().unwrap().vcpu_ctx.guest_ctx.gp_regs
            .x_reg[5];
        t1 = vm.vcpus[0].lock().unwrap().vcpu_ctx.guest_ctx.gp_regs
            .x_reg[6];

        vm.vm_destroy();

        assert_eq!(t0_ans, t0);
        assert_eq!(t1_ans, t1);
    }

    #[test]
    fn test_vtimer_sret() { 
        let mut vm_config = test_vm_config_create();
        let elf_path: &str = "./tests/integration/vtimer_sret.img";
        vm_config.kernel_img_path = String::from(elf_path);
        let mut vm = virtualmachine::VirtualMachine::new(vm_config);

        vm.vm_init();

        let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

        vm.vcpus[0].lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
            = entry_point;

        vm.vm_run();

        let a0: u64;

        // correct a0 after time irq\n"
        let a0_ans: u64 = 0xcafe;

        a0 = vm.vcpus[0].lock().unwrap().vcpu_ctx.guest_ctx.gp_regs
            .x_reg[10];

        vm.vm_destroy();

        assert_eq!(a0_ans, a0);
    }

    /* test the correctness of the data from initrd */
    #[test]
    fn test_initrd_load_data_vmlinux() {
        let mut vm_config = test_vm_config_create();
        let dtb_path: &str = "./test-files-laputa/vmlinux.dtb";
        vm_config.dtb_path = String::from(dtb_path);
        let initrd_path: &str = "./test-files-laputa/rootfs-vm.img";
        vm_config.initrd_path = String::from(initrd_path);
        let mut vm = virtualmachine::VirtualMachine::new(vm_config);

        let ans_res = std::fs::read(initrd_path);
        if ans_res.is_err() {
            panic!("Ans initrd load failed");
        }
        let ans_data = ans_res.unwrap();

        let dtb_res = vm.init_gpa_block_dtb();
        if dtb_res.is_none() {
            panic!("Load DTB failed");
        }

        let initrd_res = vm.init_gpa_block_initrd();
        if initrd_res.is_none() {
            panic!("Load initrd failed");
        }

        let (_initrd_gpa, initrd_hva) = initrd_res.unwrap();
        let result: i32;
        unsafe {
            result = libc::memcmp(initrd_hva as *const c_void,
                    ans_data.as_ptr() as *const c_void,
                    ans_data.len());
        }

        assert_eq!(result, 0);
    }

    /* test the initrd-loading function with a fake rootfs file */
    #[test]
    fn test_initrd_load_data_fake_file() {
        let mut vm_config = test_vm_config_create();
        let dtb_path: &str = "./test-files-laputa/vmlinux.dtb";
        vm_config.dtb_path = String::from(dtb_path);
        let initrd_path: &str = "./test-files-laputa/rootfs-vm.img";
        let wrong_path: &str = "./test-files-laputa/fake.img";
        vm_config.initrd_path = String::from(wrong_path);
        let mut vm = virtualmachine::VirtualMachine::new(vm_config);

        let ans_res = std::fs::read(initrd_path);
        if ans_res.is_err() {
            panic!("Ans initrd load failed");
        }
        let ans_data = ans_res.unwrap();

        let dtb_res = vm.init_gpa_block_dtb();
        if dtb_res.is_none() {
            panic!("Load DTB failed");
        }
        let initrd_res = vm.init_gpa_block_initrd();

        if initrd_res.is_none() {
            panic!("Load initrd failed");
        }

        let (_initrd_gpa, initrd_hva) = initrd_res.unwrap();
        let result: i32;
        unsafe {
            result = libc::memcmp(initrd_hva as *const c_void,
                    ans_data.as_ptr() as *const c_void,
                    ans_data.len());
        }

        assert_ne!(result, 0);
    }

    /* 
        * test the initrd-loading function with a legal but wrong rootfs 
        * file
        */
    #[test]
    fn test_initrd_load_data_wrong_file() {
        let mut vm_config = test_vm_config_create();
        let dtb_path: &str = "./test-files-laputa/vmlinux.dtb";
        vm_config.dtb_path = String::from(dtb_path);
        let initrd_path: &str = "./test-files-laputa/rootfs-vm.img";
        let wrong_path: &str = "./test-files-laputa/rootfs-vm-wrong.img";
        vm_config.initrd_path = String::from(wrong_path);
        let mut vm = virtualmachine::VirtualMachine::new(vm_config);

        let ans_res = std::fs::read(initrd_path);
        if ans_res.is_err() {
            panic!("Ans initrd load failed");
        }
        let ans_data = ans_res.unwrap();

        let dtb_res = vm.init_gpa_block_dtb();
        if dtb_res.is_none() {
            panic!("Load DTB failed");
        }
        let initrd_res = vm.init_gpa_block_initrd();

        if initrd_res.is_none() {
            panic!("Load initrd failed");
        }

        let (_initrd_gpa, initrd_hva) = initrd_res.unwrap();
        let result: i32;
        unsafe {
            result = libc::memcmp(initrd_hva as *const c_void,
                    ans_data.as_ptr() as *const c_void,
                    ans_data.len());
        }

        assert_ne!(result, 0);
    }
}