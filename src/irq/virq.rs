#[allow(unused)]
pub struct VirtualInterrupt {
    irq_pending: [u64; 16],
}

impl VirtualInterrupt {
    pub fn new() -> Self {
        let irq_pending = [0; 16];
        VirtualInterrupt {
            irq_pending,
        }
    }
}