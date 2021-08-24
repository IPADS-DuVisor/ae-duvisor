use crate::init::cmdline::MAX_VCPU;

#[allow(unused)]
pub struct VirtualIpi {
    pub target_vcpu: [i8; MAX_VCPU as usize],
}

impl VirtualIpi {
    pub fn new() -> Self {
        Self {
            target_vcpu: [0; MAX_VCPU as usize],
        }
    }

    /* TODO: Get cpu mask for the target vcpus */
    pub fn send_vipi(&mut self, vcpu_id: u8) {
        for i in 0..MAX_VCPU {
            if i != vcpu_id as u32 {
                self.target_vcpu[i as usize] = 1;
            }
        }
    }
}
