use crate::init::cmdline::MAX_VCPU;
use std::sync::atomic::{AtomicU64, Ordering}; 
#[allow(unused)]
use crate::vcpu::utils::*;
use crate::vcpu::virtualcpu::SEND_UIPI_CNT;

#[allow(unused)]
pub struct VirtualIpi {
    pub id_map: Vec<AtomicU64>,
    pub vcpu_num: u32,
}

impl VirtualIpi {
    pub fn new(vcpu_num: u32) -> Self {
        let mut id_map: Vec<AtomicU64> = Vec::with_capacity(vcpu_num as usize);
        
        for _ in 0..vcpu_num {
            id_map.push(AtomicU64::new(0));
        }

        Self {
            id_map,
            vcpu_num,
        }
    }

    // VIPI_ID = MAX_VCPU * VMID + VCPUID + 1
    pub fn vcpu_regist(&self, vcpu_id: u32, vipi_id: u64) {
        self.id_map[vcpu_id as usize].store(vipi_id, Ordering::SeqCst);

        unsafe {
            csrw!(VCPUID, vipi_id);
        }

        println!("vcpu_regist {}", unsafe {csrr!(VCPUID)});
    }

    /* TODO: Get cpu mask for the target vcpus */
    pub fn send_vipi(&self, hart_mask: u64) {
        let mut vipi_id: u64;
        for i in 0..MAX_VCPU {
            if ((1 << i) & hart_mask) != 0 {
                vipi_id = self.id_map[i as usize].load(Ordering::SeqCst);
                self.send_uipi(vipi_id);
            }
        }
    }

    pub fn send_uipi(&self, vipi_id: u64) {
        match vipi_id {
            1..=63 => { /* Set VIPI0 */
                println!("VIPI 0 SET");
                unsafe {
                    csrs!(VIPI0, 1 << vipi_id);
                    SEND_UIPI_CNT += 1;
                }
            },
            64..=127 => { /* Set VIPI1 */
                println!("VIPI 1 SET");
                unsafe {
                    csrs!(VIPI1, 1 << (vipi_id - 64));
                    SEND_UIPI_CNT += 1;
                }
            },
            128..=191 => { /* Set VIPI2 */
                println!("VIPI 2 SET");
                unsafe {
                    csrs!(VIPI2, 1 << (vipi_id - 128));
                    SEND_UIPI_CNT += 1;
                }
            },
            192..=255 => { /* Set VIPI3 */
                println!("VIPI 3 SET");
                unsafe {
                    csrs!(VIPI3, 1 << (vipi_id - 192));
                    SEND_UIPI_CNT += 1;
                }
            },
            _ => {
                println!("Invalid vipi id ! {}", vipi_id);
            },
        }
    }

    pub fn clear_vipi(vipi_id: u64) {
        match vipi_id {
            1..=63 => { /* Clear VIPI0 */
                unsafe {
                    csrc!(VIPI0, 1 << vipi_id);
                }
            },
            64..=127 => { /* Clear VIPI1 */
                unsafe {
                    csrc!(VIPI1, 1 << (vipi_id - 64));
                }
            },
            128..=191 => { /* Clear VIPI2 */
                unsafe {
                    csrc!(VIPI2, 1 << (vipi_id - 128));
                }
            },
            192..=255 => { /* Clear VIPI3 */
                unsafe {
                    csrc!(VIPI3, 1 << (vipi_id - 192));
                }
            },
            _ => {
                println!("Invalid vipi id ! {}", vipi_id);
            },
        }
    }
}

#[cfg(test)]
pub mod tests {
    use rusty_fork::rusty_fork_test;
    use crate::vm::virtualmachine;
    use crate::test::utils::configtest::test_vm_config_create;
    use std::{thread, time};
    use crate::mm::gstagemmu::*;
    use crate::mm::utils::*;
    use crate::vcpu::utils::*;
    use crate::vcpu::virtualcpu::GET_UIPI_CNT;
    use crate::init::cmdline::MAX_VCPU;

    pub static mut HU_IPI_CNT: i32 = 0;

    /* test_vipi_virtual_ipi_remote_running */
    pub static mut TEST_SUCCESS_CNT: i32 = 0;

    /* test_vipi_send_to_null_vcpu */
    pub static mut INVALID_TARGET_VCPU: i32 = 0;

    rusty_fork_test! {
        #[test]
        fn test_vipi_user_ipi_remote() {
            unsafe {
                println!("Init GET_UIPI_CNT {}", GET_UIPI_CNT);
            }
            let mut vm_config = test_vm_config_create();
            /* Multi vcpu test */
            //vm_config.vcpu_count = 2;
            let elf_path: &str = "./tests/integration/vipi_user_ipi_remote.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            /* Set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            //vm.vcpus[1].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
            //        = entry_point;

            /* Set a0 = vcpu_id */
            vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[0].vcpu_id as u64;
            //vm.vcpus[1].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
            //    = vm.vcpus[1].vcpu_id as u64;

            /* Set target address for sync */
            /* Target address will be set with 0x1 if the vcpu is ready */
            let target_address = 0x3000;

            /* Add gpa_block for target_address in advance */
            let res = vm.vm_state.gsmmu.lock().unwrap()
                .gpa_block_add(target_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!");
            }

            /* Get the hva of 0x3000(gpa) */
            let (hva, hpa) = res.unwrap();
            println!("hva {:x}, hpa {:x}", hva, hpa);

            /* Map the page on g-stage */
            let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                    | PTE_EXECUTE;
            vm.vm_state.gsmmu.lock().unwrap().map_page(target_address, hpa, 
                    flag);

            /* Clear target address before the threads run */
            unsafe {
                *(hva as *mut u64) = 0;
            }

            let vmid = vm.vm_state.vm_id;
            println!("******Test 1 vmid {}", vmid);

            /* Start a thread to wait for vcpu 1 ready and send user ipi */
            let handle: thread::JoinHandle<()>;

            handle = thread::spawn(move || {
                println!("Wait for vcpu 1");

                unsafe {
                    while *(hva as *mut u64) == 0 {
                        let ten_millis = time::Duration::from_millis(10);

                        thread::sleep(ten_millis);
                    }
                }

                unsafe {
                    println!("Vcpu ready! {:x}", *(hva as *mut u64));
                    let target_vipi_id = vmid * (MAX_VCPU as u64) + 1;
                    println!("target_vipi_id: {}", target_vipi_id);

                    /* Send user ipi via VIPI0_CSR */
                    csrs!(VIPI0, 1 << target_vipi_id);

                    /*
                     * Set *0x3000 = 2 to drive the vcpu continue to end. 
                     * Otherwise the vcpu will loop forever and there will
                     * be no output even eith --nocapture
                     */
                    *(hva as *mut u64) = 2;
                }
            });

            /* Start the test vm */
            vm.vm_run();

            let u_ipi_cnt: i64;

            unsafe {
                u_ipi_cnt = GET_UIPI_CNT;
            }
            
            println!("Get {} user ipi", u_ipi_cnt);

            vm.vm_destroy();

            /* This test case should only get 1 user ipi and end immediately */
            assert_eq!(1, u_ipi_cnt);
        }

        #[test]
        fn test_vipi_user_ipi_remote1() { 
            unsafe {
                println!("Init GET_UIPI_CNT {}", GET_UIPI_CNT);
            }
            let mut vm_config = test_vm_config_create();
            /* Multi vcpu test */
            vm_config.vcpu_count = 2;
            let elf_path: &str = "./tests/integration/vipi_user_ipi_remote1.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            /* Set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;

            /* Set a0 = vcpu_id */
            vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[0].vcpu_id as u64;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[1].vcpu_id as u64;

            /* Set target address for sync */
            /* Target address will be set with 0x1 if the vcpu is ready */
            let target_address = 0x3000;

            /* Add gpa_block for target_address in advance */
            let res = vm.vm_state.gsmmu.lock().unwrap()
                .gpa_block_add(target_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!");
            }

            /* Get the hva of 0x3000(gpa) */
            let (hva, hpa) = res.unwrap();
            println!("hva {:x}, hpa {:x}", hva, hpa);

            /* Map the page on g-stage */
            let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                    | PTE_EXECUTE;
            vm.vm_state.gsmmu.lock().unwrap().map_page(target_address, hpa, 
                    flag);

            /* Clear target address before the threads run */
            unsafe {
                *(hva as *mut u64) = 0;
            }

            let vmid = vm.vm_state.vm_id;
            println!("******Test 2 vmid {}", vmid);

            /* Start a thread to wait for vcpu 1 ready and send user ipi */
            let handle: thread::JoinHandle<()>;

            handle = thread::spawn(move || {
                println!("Wait for vcpu 1");

                unsafe {
                    while *(hva as *mut u64) == 0 {
                        let ten_millis = time::Duration::from_millis(10);

                        thread::sleep(ten_millis);
                    }
                }

                unsafe {
                    println!("Vcpu ready! {:x}", *(hva as *mut u64));
                    let target_vipi_id = vmid * (MAX_VCPU as u64) + 2;
                    println!("target_vipi_id: {}", target_vipi_id);

                    /* 
                     * Send user ipi via VIPI0_CSR before change the
                     * sync data.
                     */
                    csrs!(VIPI0, 1 << target_vipi_id);

                    /* 
                     * Set *0x3000 = 2 to drive the vcpu continue to end. 
                     * Otherwise the vcpu will loop forever and there will
                     * be no output even eith --nocapture
                     */
                    *(hva as *mut u64) = 2;
                }
            });
            
            /* Start the test vm */
            vm.vm_run();

            let u_ipi_cnt: i64;

            unsafe {
                u_ipi_cnt = GET_UIPI_CNT;
            }
            
            println!("Get {} user ipi", u_ipi_cnt);

            vm.vm_destroy();

            /* This test case should only get 1 user ipi and end immediately */
            assert_eq!(1, u_ipi_cnt);
        }

        /* #[test]
        fn test_vipi_virtual_ipi_local() {
            unsafe {
                println!("Init GET_UIPI_CNT {}", GET_UIPI_CNT);
            }
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/vipi_virtual_ipi_local.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let vmid = vm.vm_state.vm_id;
            println!("******Test 3 vmid {}", vmid);

            /* Set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;

            /* Start the test vm */
            vm.vm_run();

            vm.vm_destroy();

            /* This test case is passed if the vm_run can bypass the loop */
        } */

        #[test]
        fn test_vipi_virtual_ipi_remote_running() { 
            unsafe {
                println!("Init GET_UIPI_CNT {}", GET_UIPI_CNT);
            }
            let mut vm_config = test_vm_config_create();
            /* Multi vcpu test */
            vm_config.vcpu_count = 2;
            let elf_path: &str = "./tests/integration/vipi_virtual_ipi_remote_running.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let vmid = vm.vm_state.vm_id;
            println!("******Test 4 vmid {}", vmid);

            /* Set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;

            /* Set a0 = vcpu_id */
            vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[0].vcpu_id as u64;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[1].vcpu_id as u64;

            /* Set target address for sync */
            /* Target address will be set with 0x1 if the vcpu is ready */
            let target_address = 0x3000;

            /* Add gpa_block for target_address in advance */
            let res = vm.vm_state.gsmmu.lock().unwrap()
                .gpa_block_add(target_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!");
            }

            /* Get the hva of 0x3000(gpa) */
            let (hva, hpa) = res.unwrap();
            println!("hva {:x}, hpa {:x}", hva, hpa);

            /* Map the page on g-stage */
            let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                    | PTE_EXECUTE;
            vm.vm_state.gsmmu.lock().unwrap().map_page(target_address, hpa, 
                    flag);

            /* Clear target address before the threads run */
            unsafe {
                *(hva as *mut u64) = 0;
            }
            
            /* Start the test vm */
            vm.vm_run();

            let success_cnt: i32;

            unsafe {
                success_cnt = TEST_SUCCESS_CNT;
            }
            
            println!("Get {} success cnt", success_cnt);

            vm.vm_destroy();

            /* Vcpu 1 should exit from irq_handler */
            assert_eq!(1, success_cnt);
        }

        #[test]
        fn test_vipi_virtual_ipi_remote_not_running() { 
            unsafe {
                println!("Init GET_UIPI_CNT {}", GET_UIPI_CNT);
            }
            let mut vm_config = test_vm_config_create();
            /* Multi vcpu test */
            vm_config.vcpu_count = 2;
            let elf_path: &str = "./tests/integration/vipi_virtual_ipi_remote_not_running.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let vmid = vm.vm_state.vm_id;
            println!("******Test 5 vmid {}", vmid);

            /* Set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;

            /* Set a0 = vcpu_id */
            vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[0].vcpu_id as u64;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[1].vcpu_id as u64;

            /* Set target address for sync */
            /* Target address will be set with 0x1 if the vcpu is ready */
            let target_address = 0x3000;

            /* Add gpa_block for target_address in advance */
            let res = vm.vm_state.gsmmu.lock().unwrap()
                .gpa_block_add(target_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!");
            }

            /* Get the hva of 0x3000(gpa) */
            let (hva, hpa) = res.unwrap();
            println!("hva {:x}, hpa {:x}", hva, hpa);

            /* Map the page on g-stage */
            let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                    | PTE_EXECUTE;
            vm.vm_state.gsmmu.lock().unwrap().map_page(target_address, hpa,
                    flag);

            /* Clear target address before the threads run */
            unsafe {
                *(hva as *mut u64) = 0;
            }

            /* Set a1 = hva */
            vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[11]
                = hva;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[11]
                = hva;
            
            /* Start the test vm */
            vm.vm_run();

            let success_cnt: i32;

            unsafe {
                success_cnt = TEST_SUCCESS_CNT;
            }
            
            println!("Get {} success cnt", success_cnt);

            vm.vm_destroy();

            /* Vcpu 1 should exit from irq_handler */
            assert_eq!(1, success_cnt);
        }

        #[test]
        fn test_vipi_virtual_ipi_remote_each() { 
            unsafe {
                println!("Init GET_UIPI_CNT {}", GET_UIPI_CNT);
            }
            let mut vm_config = test_vm_config_create();
            /* Multi vcpu test */
            vm_config.vcpu_count = 2;
            let elf_path: &str = "./tests/integration/vipi_virtual_ipi_remote_each.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let vmid = vm.vm_state.vm_id;
            println!("******Test 6 vmid {}", vmid);

            /* Set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;

            /* Set a0 = vcpu_id */
            vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[0].vcpu_id as u64;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[1].vcpu_id as u64;

            /* Set target address for sync */
            /* Target address will be set with 0x1 if the vcpu is ready */
            let target_address = 0x3000;

            /* Add gpa_block for target_address in advance */
            let res = vm.vm_state.gsmmu.lock().unwrap()
                .gpa_block_add(target_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!");
            }

            /* Get the hva of 0x3000(gpa) */
            let (hva, hpa) = res.unwrap();
            println!("hva {:x}, hpa {:x}", hva, hpa);

            /* Map the page on g-stage */
            let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                    | PTE_EXECUTE;
            vm.vm_state.gsmmu.lock().unwrap().map_page(target_address, hpa,
                    flag);

            /* Clear target address before the threads run */
            unsafe {
                *(hva as *mut u64) = 0;
            }

            /* Set a1 = hva */
            vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[11]
                = hva;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[11]
                = hva;
            
            /* Start the test vm */
            vm.vm_run();

            let success_cnt: i32;

            unsafe {
                success_cnt = TEST_SUCCESS_CNT;
            }
            
            println!("Get {} success cnt", success_cnt);

            vm.vm_destroy();

            /* Vcpu 1 should exit from irq_handler */
            assert_eq!(1, success_cnt);
        }

        #[test]
        fn test_vipi_send_to_null_vcpu() { 
            unsafe {
                println!("Init GET_UIPI_CNT {}", GET_UIPI_CNT);
            }
            let mut vm_config = test_vm_config_create();
            /* Multi vcpu test */
            vm_config.vcpu_count = 2;
            let elf_path: &str = "./tests/integration/vipi_send_to_null_vcpu.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let vmid = vm.vm_state.vm_id;
            println!("******Test 7 vmid {}", vmid);

            /* Set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;

            /* Set a0 = vcpu_id */
            vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[0].vcpu_id as u64;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[1].vcpu_id as u64;

            /* Set target address for sync */
            /* Target address will be set with 0x1 if the vcpu is ready */
            let target_address = 0x3000;

            /* Add gpa_block for target_address in advance */
            let res = vm.vm_state.gsmmu.lock().unwrap()
                .gpa_block_add(target_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!");
            }

            /* Get the hva of 0x3000(gpa) */
            let (hva, hpa) = res.unwrap();
            println!("hva {:x}, hpa {:x}", hva, hpa);

            /* Map the page on g-stage */
            let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                    | PTE_EXECUTE;
            vm.vm_state.gsmmu.lock().unwrap().map_page(target_address, hpa,
                    flag);

            /* Clear target address before the threads run */
            unsafe {
                *(hva as *mut u64) = 0;
            }

            /* Set a1 = hva */
            vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[11]
                = hva;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[11]
                = hva;
            
            /* Start the test vm */
            vm.vm_run();

            vm.vm_destroy();

            let invalid_cnt: i32;

            unsafe {
                invalid_cnt = INVALID_TARGET_VCPU;
            }
            
            println!("Get {} invalid cnt", invalid_cnt);

            /* Target vcpu [2,3,4,5,6,7] is invalid */
            assert_eq!(6, invalid_cnt);

            /* Vcpu 0 should exit from test_success */
            let success_cnt: i32;

            unsafe {
                success_cnt = TEST_SUCCESS_CNT;
            }
            
            println!("Get {} success cnt", success_cnt);

            assert_eq!(1, success_cnt);
        }

        #[test]
        fn test_vipi_virtual_ipi_accurate() { 
            unsafe {
                println!("Init GET_UIPI_CNT {}", GET_UIPI_CNT);
            }
            let mut vm_config = test_vm_config_create();
            /* Multi vcpu test */
            vm_config.vcpu_count = 3;
            let elf_path: &str = "./tests/integration/vipi_virtual_ipi_accurate.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let vmid = vm.vm_state.vm_id;
            println!("******Test 8 vmid {}", vmid);

            /* Set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            vm.vcpus[2].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;

            /* Set a0 = vcpu_id */
            vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[0].vcpu_id as u64;
            vm.vcpus[1].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[1].vcpu_id as u64;
            vm.vcpus[2].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10]
                = vm.vcpus[2].vcpu_id as u64;

            /* Set target address for sync */
            /* Target address will be set with 0x1 if the vcpu is ready */
            let target_address = 0x3000;

            /* Add gpa_block for target_address in advance */
            let res = vm.vm_state.gsmmu.lock().unwrap()
                .gpa_block_add(target_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!");
            }

            /* Get the hva of 0x3000(gpa) */
            let (hva, hpa) = res.unwrap();
            println!("hva {:x}, hpa {:x}", hva, hpa);

            /* Map the page on g-stage */
            let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                    | PTE_EXECUTE;
            vm.vm_state.gsmmu.lock().unwrap().map_page(target_address, hpa,
                    flag);

            /* Clear target address before the threads run */
            unsafe {
                *(hva as *mut u64) = 0;
            }
            
            /* Start the test vm */
            vm.vm_run();

            let success_cnt: i32;

            unsafe {
                success_cnt = TEST_SUCCESS_CNT;
            }
            
            println!("Get {} success cnt", success_cnt);

            vm.vm_destroy();

            /* 
             * Vcpu 0 and 2 should not exit from irq_handler and 
             * trigger SBI_TEST_SUCCESS both. Vcpu 1 should exit from
             * irq_handler once. So the answer is 3.
             */
            assert_eq!(3, success_cnt);
        }
    }
}
