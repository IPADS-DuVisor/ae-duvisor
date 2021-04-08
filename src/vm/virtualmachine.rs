use crate::vcpu::virtualcpu;
use crate::mm::gstagemmu;
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
    pub gsmmu: gstagemmu::GStageMmu,
}

impl VirtualMachine {
    pub fn new(vcpu_num: u32) -> VirtualMachine {
        let vcpus: Vec<Arc<Mutex<virtualcpu::VirtualCpu>>> = Vec::new();
        let vm_state = VmSharedState::new();
        let vm_state_mutex = Arc::new(Mutex::new(vm_state));
        let mut vcpu_mutex: Arc<Mutex<virtualcpu::VirtualCpu>>;
        let gsmmu = gstagemmu::GStageMmu::new();

        // Create vm struct instance
        let mut vm = VirtualMachine {
            vcpus,
            vcpu_num,
            vm_state: vm_state_mutex.clone(),
            gsmmu,
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

        // For debug
        self.gsmmu.gsmmu_test();

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
