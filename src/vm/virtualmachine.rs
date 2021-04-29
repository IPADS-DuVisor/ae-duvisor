use crate::vcpu::virtualcpu;
use crate::mm::gstagemmu;
use crate::plat::uhe::ioctl::ioctl_constants;
use crate::irq::delegation::delegation_constants;
use std::thread;
use std::sync::{Arc, Mutex};
use std::ffi::CString;
use ioctl_constants::*;
use delegation_constants::*;
#[allow(unused_imports)]
use crate::mm::utils;
use core::ffi::c_void;

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
}

// Export to vcpu
pub struct VmSharedState {
    pub vm_id: u32,
    pub ioctl_fd: i32,
    pub gsmmu: gstagemmu::GStageMmu,
}

impl VmSharedState {
    pub fn new(ioctl_fd: i32) -> Self {
        Self {
            vm_id: 0,
            ioctl_fd,
            gsmmu: gstagemmu::GStageMmu::new(ioctl_fd),
        }
    }
}

pub struct VirtualMachine {
    pub vm_state: Arc<Mutex<VmSharedState>>,
    pub vcpus: Vec<Arc<Mutex<virtualcpu::VirtualCpu>>>,
    pub vcpu_num: u32,
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

    pub fn new(vcpu_num: u32) -> Self {
        let vcpus: Vec<Arc<Mutex<virtualcpu::VirtualCpu>>> = Vec::new();

        // get ioctl fd of "/dev/laputa_dev" 
        let ioctl_fd = VirtualMachine::open_ioctl();

        let vm_state = VmSharedState::new(ioctl_fd);
        let vm_state_mutex = Arc::new(Mutex::new(vm_state));
        let mut vcpu_mutex: Arc<Mutex<virtualcpu::VirtualCpu>>;

        // Create vm struct instance
        let mut vm = Self {
            vcpus,
            vcpu_num,
            vm_state: vm_state_mutex.clone(),
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

    // Init vm & vcpu before vm_run()
    pub fn vm_init(&mut self) {
        let ioctl_fd = self.vm_state.lock().unwrap().ioctl_fd;

        // Delegate traps via ioctl
        VirtualMachine::hu_delegation(ioctl_fd);
        self.vm_state.lock().unwrap().gsmmu.allocator.set_ioctl_fd(ioctl_fd);
    }

    pub fn vm_img_load(&mut self, gpa_start: u64, length: u64) -> u64{
        let res = self.vm_state.lock().unwrap().
            gsmmu.gpa_region_add(gpa_start, length);
        if !res.is_ok() {
            panic!("vm_img_load failed");
        }

        let (hva, hpa) = res.unwrap();
        println!("New hpa: {:x}", hpa);
        
        unsafe {
            let ptr = hva as *mut i32;

            // set vm img code
            libc::memcpy(ptr as *mut c_void, gpa_start as *mut c_void,
                length as usize);

            println!("memcpy ptr {:x}", ptr as u64);
            println!("memcpy length {:x}", length);
        }

        println!("memcpy hva {:x}", hva);

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
            println!("ioctl result: {}", res);
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

    rusty_fork_test! {
        #[test]
        fn test_vm_add_all_gprs() { 
            println!("---------start vm------------");
            let nr_vcpu = 1;
            let sum_ans = 10;
            let mut sum = 0;
            let mut vm = virtualmachine::VirtualMachine::new(nr_vcpu);
            vm.vm_init();

            // set test code
            let start = vcpu_add_all_gprs as u64;
            let end = vcpu_add_all_gprs_end as u64;
            let length = end - start;
            let entry_point: u64 = vm.vm_img_load(start, length);

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
            let nr_vcpu = 1;
            let exit_reason_ans = 2; // g-stage page fault for no permission
            let mut exit_reason = 0;
            let mut vm = virtualmachine::VirtualMachine::new(nr_vcpu);
            vm.vm_init();
            let ro_address = 0x3000;

            // set test code
            let start = vmem_W_Ro as u64;
            let end = vmem_W_Ro_end as u64;
            let length = end - start;
            let entry_point: u64 = vm.vm_img_load(start, length);

            let res = vm.vm_state.lock().unwrap()
                .gsmmu.gpa_region_add(ro_address, utils::PAGE_SIZE);
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
            let nr_vcpu = 1;
            let exit_reason_ans = 2; // g-stage page fault for no permission
            let mut exit_reason = 0;
            let mut vm = virtualmachine::VirtualMachine::new(nr_vcpu);
            vm.vm_init();
            let nx_address = 0x3000;

            // set test code
            let start = vmem_X_nonX as u64;
            let end = vmem_X_nonX_end as u64;
            let length = end - start;
            let entry_point: u64 = vm.vm_img_load(start, length);

            let res = vm.vm_state.lock().unwrap()
                .gsmmu.gpa_region_add(nx_address, utils::PAGE_SIZE);
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

        #[test]
        fn test_vmem_mapping() { 
            let nr_vcpu = 1;
            let exit_reason_ans = 0xdead;
            let mut exit_reason = 0;
            let mut vm = virtualmachine::VirtualMachine::new(nr_vcpu);
            vm.vm_init();

            // set test code
            let start = vmem_ld_mapping as u64;
            let end = vmem_ld_mapping_end as u64;
            let length = end - start;
            let entry_point: u64 = vm.vm_img_load(start, length);

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
            let nr_vcpu = 1;
            let exit_reason_ans = 0xdead;
            let mut exit_reason = 0;
            let mut vm = virtualmachine::VirtualMachine::new(nr_vcpu);
            vm.vm_init();

            // set test code
            let start = vmem_ld_sd_over_loop as u64;
            let end = vmem_ld_sd_over_loop_end as u64;
            let length = end - start;
            let entry_point: u64 = vm.vm_img_load(start, length);

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
        fn test_vm_new() { 
            let vcpu_num = 4;
            let vm = VirtualMachine::new(vcpu_num);

            assert_eq!(vm.vcpu_num, vcpu_num);
        }

        // Check the num of the vcpu created
        #[test]
        fn test_vm_new_vcpu() {   
            let vcpu_num = 4;
            let vm = VirtualMachine::new(vcpu_num);
            let mut sum = 0;

            for i in &vm.vcpus {
                sum = sum + i.lock().unwrap().vcpu_id;
            } 

            assert_eq!(sum, 6); // 0 + 1 + 2 + 3
        }
    }
}
