#[allow(unused)]
pub struct VirtualInterrupt {
    // FIXME: use bit array
    irq_pending: [u64; 32],
}

impl VirtualInterrupt {
    pub fn new() -> Self {
        let irq_pending = [0; 32];
        VirtualInterrupt {
            irq_pending,
        }
    }

    pub fn set_pending_irq(&mut self, irq: u32) {
        if irq >= 32 { panic!("set_pending_irq: irq {} out of range", irq); }
        self.irq_pending[irq as usize] = 1;
    }
    
    pub fn unset_pending_irq(&mut self, irq: u32) {
        if irq >= 32 { panic!("set_pending_irq: irq {} out of range", irq); }
        self.irq_pending[irq as usize] = 0;
    }
}
