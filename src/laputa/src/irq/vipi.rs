use crate::init::cmdline::MAX_VCPU;
use std::sync::atomic::{AtomicU64, Ordering}; 
#[allow(unused)]
use crate::vcpu::utils::*;

#[allow(unused)]
pub struct VirtualIpi {
    pub id_map: Vec<AtomicU64>,
}

impl VirtualIpi {
    pub fn new(vcpu_num: u32) -> Self {
        let mut id_map: Vec<AtomicU64> = Vec::with_capacity(vcpu_num as usize);
        
        for _ in 0..vcpu_num {
            id_map.push(AtomicU64::new(0));
        }

        Self {
            id_map,
        }
    }

    // TODO: add 1 for vcpu id
    pub fn vcpu_regist(&self, vcpu_id: u32, vipi_id: u64) {
        self.id_map[vcpu_id as usize].store(vipi_id, Ordering::SeqCst);

        unsafe {
            csrw!(VCPUID, vipi_id);
        }
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

    // TODO: use one VIPI csr for now
    pub fn send_uipi(&self, vipi_id: u64) {
        if vipi_id < 64 {
            unsafe {
                csrs!(VIPI0, 1 << vipi_id);
            }
        } else {
            println!("Invalid vipi id");
        }
    }
}
