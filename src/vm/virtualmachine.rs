use crate::vcpu::virtualcpu;
use crate::mm::gparegion;
use crate::mm::allocator;
use std::thread;
use std::sync::{Arc, Mutex};

// Export to vcpu
pub struct VmSharedState {
    pub vm_id: u32,
}

impl VmSharedState {
    pub fn new() -> VmSharedState {
        VmSharedState {
            vm_id: 0,
        }
    }
}

pub struct VirtualMachine {
    pub vm_state: Arc<Mutex<VmSharedState>>,
    pub vcpus: Vec<Arc<Mutex<virtualcpu::VirtualCpu>>>,
    pub vcpu_num: u32,
}

impl VirtualMachine {
    pub fn new(vcpu_num: u32) -> VirtualMachine {
        let vcpus: Vec<Arc<Mutex<virtualcpu::VirtualCpu>>> = Vec::new();
        let vm_state = VmSharedState::new();
        let vm_state_mutex = Arc::new(Mutex::new(vm_state));
        let mut vcpu_mutex: Arc<Mutex<virtualcpu::VirtualCpu>>;

        // Create vm struct instance
        let mut vm = VirtualMachine {
            vcpus,
            vcpu_num,
            vm_state: vm_state_mutex.clone(),
        };

        // Create vcpu struct instance
        for i in 0..vcpu_num {
            let vcpu = virtualcpu::VirtualCpu::new(i, vm_state_mutex.clone());
            vcpu_mutex = Arc::new(Mutex::new(vcpu));
            vm.vcpus.push(vcpu_mutex);
        }

        // Return vm instance with vcpus
        vm
    }

    pub fn vm_run(&mut self) {
        let mut vcpu_handle: Vec<thread::JoinHandle<()>> = Vec::new();
        let mut handle: thread::JoinHandle<()>;
        let mut vcpu_mutex;

        // test for debug
        gparegion::import_print();
        allocator::import_print();
        let addr = unsafe { libc::malloc(4096) };
        println!("{:?}", addr);
        let mut gsmmu = gparegion::GSMMU::new();
        println!("gsmmu free_offset {:?}", gsmmu.page_table.free_offset);
        gsmmu.test_gsmmu();
        let ptr = gsmmu.page_table.root_table_create();
        println!("{:?}", ptr);
        let offset = gsmmu.page_table.free_offset;
        println!("{:?}", offset);
        gsmmu.map_page(gsmmu.page_table.region.hpm_ptr, 1, 0x1000, 0x2000, 0x7);

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
}

// Check the correctness of vm new()
#[cfg(test)]
mod tests {
    use super::*;

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
