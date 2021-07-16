#[allow(unused)]
pub struct VirtualTimer {
    next_cycles: u64,
    delta_cycles: u64,
}

impl VirtualTimer {
    pub fn new(next_cycles: u64, delta_cycles: u64) -> VirtualTimer {
        VirtualTimer {
            next_cycles,
            delta_cycles,
        }
    }
}