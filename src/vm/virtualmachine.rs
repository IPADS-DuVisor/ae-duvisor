use crate::vcpu::virtualcpu;
use crate::mm::gstagemmu;
use crate::plat::uhe::ioctl::ioctl_constants;
use crate::irq::delegation::delegation_constants;
use std::thread;
use std::sync::{Arc, Mutex};
use std::ffi::CString;
use ioctl_constants::*;
use delegation_constants::*;
use crate::mm::utils::*;
use crate::init::cmdline::VMConfig;
use crate::vm::image;

#[allow(unused)]
extern "C"
{
    fn vcpu_ecall_exit();
    fn vcpu_ecall_exit_end();
    fn vcpu_add_all_gprs();
    fn vcpu_add_all_gprs_end();
    fn vmem_ld_mapping();
    fn vmem_ld_mapping_end();
    fn vmem_W_Ro();
    fn vmem_W_Ro_end();
    fn vmem_X_nonX();
    fn vmem_X_nonX_end();
    fn vmem_ld_sd_over_loop();
    fn vmem_ld_sd_over_loop_end();
    fn vmem_ld_sd_sum();
    fn vmem_ld_sd_sum_end();
    fn vmem_ld_data();
    fn vmem_ld_data_end();
}

// Export to vcpu
pub struct VmSharedState {
    pub vm_id: u32,
    pub ioctl_fd: i32,
    pub gsmmu: gstagemmu::GStageMmu,
}

impl VmSharedState {
    pub fn new(ioctl_fd: i32, mem_size: u64)
        -> Self {
        Self {
            vm_id: 0,
            ioctl_fd,
            gsmmu: gstagemmu::GStageMmu::new(ioctl_fd, mem_size),
        }
    }
}

pub struct VirtualMachine {
    pub vm_state: Arc<Mutex<VmSharedState>>,
    pub vcpus: Vec<Arc<Mutex<virtualcpu::VirtualCpu>>>,
    pub vcpu_num: u32,
    pub mem_size: u64,
    pub vm_image: image::VmImage,
}

impl VirtualMachine {
    pub fn open_ioctl() -> i32 {
        let file_path = CString::new("/dev/laputa_dev").unwrap();
        let ioctl_fd;

        unsafe {
            ioctl_fd = (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            if ioctl_fd == -1 {
                panic!("Open /dev/laputa_dev failed");
            }
        }

        ioctl_fd
    }

    pub fn new(vm_config: VMConfig) -> Self {
        let vcpu_num = vm_config.vcpu_count;
        let mem_size = vm_config.mem_size;
        let elf_path = &vm_config.kernel_img_path[..];
        let vcpus: Vec<Arc<Mutex<virtualcpu::VirtualCpu>>> = Vec::new();
        let vm_image = image::VmImage::new(elf_path);

        // get ioctl fd of "/dev/laputa_dev" 
        let ioctl_fd = VirtualMachine::open_ioctl();

        let vm_state = VmSharedState::new(ioctl_fd, mem_size);
        let vm_state_mutex = Arc::new(Mutex::new(vm_state));
        let mut vcpu_mutex: Arc<Mutex<virtualcpu::VirtualCpu>>;

        // Create vm struct instance
        let mut vm = Self {
            vcpus,
            vcpu_num,
            vm_state: vm_state_mutex.clone(),
            mem_size,
            vm_image,
        };

        // Create vcpu struct instance
        for i in 0..vcpu_num {
            let vcpu = virtualcpu::VirtualCpu::new(i,
                    vm_state_mutex.clone());
            vcpu_mutex = Arc::new(Mutex::new(vcpu));
            vm.vcpus.push(vcpu_mutex);
        }

        // Return vm instance with vcpus
        vm
    }

    fn load_img_data(dst: u64, src: u64, size: u64) {
        unsafe {
            std::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8,
                size as usize);

            dbgprintln!("copy_nonoverlapping: dst {:x}, src {:x}, size {:x}",
                dst, src, size);
        }
    }

    // init gpa block according to the elf file
    // return for test
    pub fn init_gpa_block_elf(&mut self) -> Vec<u64> {
        let mut hva_list: Vec<u64> = Vec::new();
        let mut offset: u64;
        let mut gpa: u64;
        let mut size: u64;
        let mut ph_data_ptr: u64;
        let img_data_ptr = self.vm_image.file_data.as_ptr() as u64;

        for i in &self.vm_image.elf_file.phdrs {
            // only PT_LOAD should be init
            if i.progtype != elf::types::PT_LOAD {
                continue;
            }

            offset = i.offset;
            gpa = i.vaddr;
            size = i.filesz;
            ph_data_ptr = img_data_ptr + offset;

            let res = self.vm_state.lock().unwrap().gsmmu
                .gpa_block_add(gpa, page_size_round_up(size));
            if !res.is_ok() {
                panic!("gpa block add failed");
            }

            let (hva, _hpa) = res.unwrap();
            hva_list.push(hva);

            VirtualMachine::load_img_data(hva, ph_data_ptr, size);
        }

        // return for test
        hva_list
    }

    // Init vm & vcpu before vm_run()
    // return for test
    pub fn vm_init(&mut self) -> Vec<u64> {
        let ioctl_fd = self.vm_state.lock().unwrap().ioctl_fd;

        // Delegate traps via ioctl
        VirtualMachine::hu_delegation(ioctl_fd);
        self.vm_state.lock().unwrap().gsmmu.allocator.set_ioctl_fd(ioctl_fd);

        // init gpa block from the elf file, return for test
        self.init_gpa_block_elf()
    }

    pub fn vm_img_load(&mut self, gpa_start: u64, length: u64) -> u64{
        let res = self.vm_state.lock().unwrap().
            gsmmu.gpa_block_add(gpa_start, length);
        if !res.is_ok() {
            panic!("vm_img_load failed");
        }

        let (hva, _hpa) = res.unwrap();
        dbgprintln!("New hpa: {:x}", _hpa);

        VirtualMachine::load_img_data(hva, gpa_start, length);

        gpa_start
    }

    pub fn vm_run(&mut self) {
        let mut vcpu_handle: Vec<thread::JoinHandle<()>> = Vec::new();
        let mut handle: thread::JoinHandle<()>;
        let mut vcpu_mutex;

        for i in &mut self.vcpus {
            vcpu_mutex = i.clone();

            // Start vcpu threads!
            handle = thread::spawn(move || {
                vcpu_mutex.lock().unwrap().thread_vcpu_run();
            });

            vcpu_handle.push(handle);
        }

        for i in vcpu_handle {
            i.join().unwrap();
        }
    }

    pub fn vm_destroy(&mut self) {
        unsafe {
            libc::close(self.vm_state.lock().unwrap().ioctl_fd);
        }
    }

    #[allow(unused)]
    pub fn hu_delegation(ioctl_fd: i32) {
        unsafe {
            let edeleg = ((1 << EXC_VIRTUAL_SUPERVISOR_SYSCALL) |
                (1 << EXC_INST_GUEST_PAGE_FAULT) | 
                (1 << EXC_VIRTUAL_INST_FAULT) |
                (1 << EXC_LOAD_GUEST_PAGE_FAULT) |
                (1 << EXC_STORE_GUEST_PAGE_FAULT)) as libc::c_ulong;
            let ideleg = (1 << IRQ_S_SOFT) as libc::c_ulong;
            let deleg = [edeleg, ideleg];
            let deleg_ptr = (&deleg) as *const u64;

            // call ioctl
            let res = libc::ioctl(ioctl_fd, IOCTL_LAPUTA_REQUEST_DELEG,
                deleg_ptr);
            dbgprintln!("ioctl result: {}", res);
        }
    }
}

// Check the correctness of vm new()
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::*;
    use rusty_fork::rusty_fork_test;
    use crate::mm::gstagemmu::gsmmu_constants;
    use gsmmu_constants::*;
    use crate::init::cmdline::configtest::test_vm_config_create;

    rusty_fork_test! {
        #[test]
        fn test_elf_parse() {
            let vm_config = test_vm_config_create();
            let vm = virtualmachine::VirtualMachine::new(vm_config);

            // answer
            let entry_ans = 0x1000;
            let phnum_ans = 1;
            let offset_ans = 0x1000;
            let paddr_ans = 0x1000;
            let vaddr_ans = 0x1000;

            let elf_file = vm.vm_image.elf_file;
            let entry_point = elf_file.ehdr.entry;
            let phnum = elf_file.phdrs.len();

            assert_eq!(entry_ans, entry_point);
            assert_eq!(phnum_ans, phnum);

            let mut p_offset = 0;
            let mut p_paddr = 0;
            let mut p_vaddr = 0;
            for i in &elf_file.phdrs {
                p_offset = i.offset;
                p_paddr = i.paddr;
                p_vaddr = i.vaddr;
            }

            println!("test_elf_parse: offset {}, paddr {}, vaddr {}", p_offset,
                p_paddr, p_vaddr);
            
            assert_eq!(offset_ans, p_offset);
            assert_eq!(paddr_ans, p_paddr);
            assert_eq!(vaddr_ans, p_vaddr);
        }

        // test init_gpa_block_elf() by compare the data from hva with img file
        #[test]
        fn test_init_gpa_block_elf() {
            let vm_config = test_vm_config_create();
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);
            let hva_list: Vec<u64>;

            hva_list = vm.vm_init();

            let mut gb_gpa;
            let mut gb_hpa;
            let mut gb_length;
            let mut target_hva = 0;
            for i in &vm.vm_state.lock().unwrap().gsmmu.mem_gpa_regions {
                gb_gpa = i.gpa;
                gb_length = i.length;
                println!("gpa_regions - gpa {:x}, length {:x}", gb_gpa,
                    gb_length);
            }

            for i in &vm.vm_state.lock().unwrap().gsmmu.gpa_blocks {
                gb_gpa = i.gpa;
                gb_hpa = i.hpa;
                gb_length = i.length;
                println!("gpa_blocks - gpa {:x}, hpa {:x}, length {:x}",
                    gb_gpa, gb_hpa, gb_length);
            }

            for i in hva_list {
                println!("hva_list {:x}", i);
                target_hva = i;
            }

            // extract answer from the img file
            let mut elf_data_ans: u64 = 0x9092908E908A40A9;
            let mut elf_data: u64;
            unsafe {
                elf_data = *(target_hva as *mut u64);
                println!("elf_data {:x}", elf_data);
            }

            assert_eq!(elf_data_ans, elf_data);

            elf_data_ans = 0x90F290EE90EA90E6;
            unsafe {
                elf_data = *((target_hva + 0x30) as *mut u64);
                println!("elf_data {:x}", elf_data);
            }

            assert_eq!(elf_data_ans, elf_data);

            elf_data_ans = 0x0;
            unsafe {
                elf_data = *((target_hva + 0x100) as *mut u64);
                println!("elf_data {:x}", elf_data);
            }

            assert_eq!(elf_data_ans, elf_data);
        }

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

            for i in &vm.vcpus {
                i.lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
                    = entry_point;
            }

            vm.vm_run();
            
            for i in &vm.vcpus {
                sum += i.lock().unwrap().vcpu_ctx.guest_ctx.gp_regs.x_reg[10];
            }
            vm.vm_destroy();

            assert_eq!(sum, sum_ans);
        }

        #[test]
        fn test_vmem_ro() { 
            let exit_reason_ans = 2; // g-stage page fault for no permission
            let mut exit_reason = 0;
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

            for i in &vm.vcpus {
                i.lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
                    = entry_point;
            }
            
            vm.vm_run();
            
            for i in &vm.vcpus {
                exit_reason = i.lock().unwrap().vcpu_ctx.host_ctx.gp_regs
                    .x_reg[0];
            }
            vm.vm_destroy();

            assert_eq!(exit_reason, exit_reason_ans);
        }

        #[test]
        fn test_vmem_nx() { 
            let exit_reason_ans = 2; // g-stage page fault for no permission
            let mut exit_reason = 0;
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

            for i in &vm.vcpus {
                i.lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
                    = entry_point;
            }
            
            vm.vm_run();
            
            for i in &vm.vcpus {
                exit_reason =
                    i.lock().unwrap().vcpu_ctx.host_ctx.gp_regs.x_reg[0];
            }
            vm.vm_destroy();

            assert_eq!(exit_reason, exit_reason_ans);
        }

        /* check the correctness of loading data from specific gpa */
        #[test]
        fn test_vmem_ld_data() { 
            let mut load_value = 0;
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/vmem_ld_data.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            /* Answer will be saved at 0x3000(gpa) */
            let answer: u64 = 0x1213141516171819;

            vm.vm_init();

            let target_address = 0x3000;

            // set entry point
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

            for i in &vm.vcpus {
                i.lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
                    = entry_point;
            }
            
            vm.vm_run();
            
            for i in &vm.vcpus {
                load_value =
                    i.lock().unwrap().vcpu_ctx.guest_ctx.gp_regs.x_reg[5];
            }
            vm.vm_destroy();

            println!("load value {:x}", load_value);

            assert_eq!(load_value, answer);
        }

        #[test]
        fn test_vmem_mapping() { 
            let exit_reason_ans = 0xdead;
            let mut exit_reason = 0;
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/vmem_ld_mapping.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            for i in &vm.vcpus {
                i.lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
                    = entry_point;
            }

            vm.vm_run();
            
            for i in &vm.vcpus {
                exit_reason =
                    i.lock().unwrap().vcpu_ctx.host_ctx.gp_regs.x_reg[0];
                println!("exit reason {:x}", exit_reason);
            }
            vm.vm_destroy();

            assert_eq!(exit_reason, exit_reason_ans);
        }

        #[test]
        fn test_vm_huge_mapping() { 
            println!("---------start test_vm_huge_mapping------------");
            let exit_reason_ans = 0xdead;
            let mut exit_reason = 0;
            let mut vm_config = test_vm_config_create();

            // cancel the three mmio regions
            let elf_path: &str 
                = "./tests/integration/vmem_ld_sd_over_loop.img";
            vm_config.kernel_img_path = String::from(elf_path);

            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            for i in &vm.vcpus {
                i.lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
                    = entry_point;
            }

            vm.vm_run();
            
            for i in &vm.vcpus {
                exit_reason =
                    i.lock().unwrap().vcpu_ctx.host_ctx.gp_regs.x_reg[0];
                println!("exit reason {:x}", exit_reason);
            }
            vm.vm_destroy();

            assert_eq!(exit_reason_ans, exit_reason);
        }

        #[test]
        fn test_vm_ld_sd_sum() { 
            println!("---------start test_vm_huge_mapping------------");
            let mut sum_ans = 0;
            let mut sum = 0;
            let mut vm_config = test_vm_config_create();

            // cancel the three mmio regions
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

            for i in &vm.vcpus {
                i.lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
                    = entry_point;
            }

            vm.vm_run();
            
            for i in &vm.vcpus {
                sum = i.lock().unwrap().vcpu_ctx.guest_ctx.gp_regs.x_reg[29];
                println!("sum {}", sum);
            }
            vm.vm_destroy();

            assert_eq!(sum_ans, sum);
        }

        #[test]
        fn test_vm_new() { 
            let vcpu_num = 1;
            let vm_config = test_vm_config_create();
            let vm = virtualmachine::VirtualMachine::new(vm_config);

            assert_eq!(vm.vcpu_num, vcpu_num);
        }

        // Check the num of the vcpu created
        #[test]
        fn test_vm_new_vcpu() {   
            let vcpu_num = 4;
            let mut vm_config = test_vm_config_create();
            vm_config.vcpu_count = vcpu_num;
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let mut sum = 0;

            for i in &vm.vcpus {
                sum = sum + i.lock().unwrap().vcpu_id;
            } 

            assert_eq!(sum, 6); // 0 + 1 + 2 + 3
        }

        #[test]
        fn test_ecall_putchar() { 
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/opensbi_putchar.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            for i in &vm.vcpus {
                i.lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
                    = entry_point;
            }

            vm.vm_run();

            let mut t0: u64 = 0;
            let mut t1: u64 = 0;

            // Sum up the chars in "Hello Ecall\n"
            let t0_ans: u64 = 1023;

            // all the ecall should should return 0 
            let t1_ans: u64 = 0;

            for i in &vm.vcpus {
                t0 = i.lock().unwrap().vcpu_ctx.guest_ctx.gp_regs.x_reg[5];
                t1 = i.lock().unwrap().vcpu_ctx.guest_ctx.gp_regs.x_reg[6];
            }

            vm.vm_destroy();

            assert_eq!(t0_ans, t0);
            assert_eq!(t1_ans, t1);
        }


        #[test]
        fn test_ecall_getchar() {
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/opensbi_getchar.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            for i in &vm.vcpus {
                i.lock().unwrap().vcpu_ctx.host_ctx.hyp_regs.uepc
                    = entry_point;
            }

            //println!("Emulation input received:");
            vm.vm_run();

            // t0 should be '\n' to end
            let mut t0: u64 = 0;
            let t0_ans: u64 = 10;

            // t1 should sum up the input
            let mut t1: u64 = 0;
            let t1_ans: u64 = 1508;

            for i in &vm.vcpus {
                t0 = i.lock().unwrap().vcpu_ctx.guest_ctx.gp_regs.x_reg[5];
                t1 = i.lock().unwrap().vcpu_ctx.guest_ctx.gp_regs.x_reg[6];
            }

            vm.vm_destroy();

            assert_eq!(t0_ans, t0);
            assert_eq!(t1_ans, t1);
        }
    }
}
