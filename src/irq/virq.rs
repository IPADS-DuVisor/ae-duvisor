use crate::vcpu::utils::*;

#[allow(unused)]
pub struct VirtualInterrupt {
    // UVTIMER (#16) is not controlled by vcpu.virq field
    // FIXME: use a bit array
    irq_pending: [bool; 16],
}

impl VirtualInterrupt {
    pub fn new() -> Self {
        let irq_pending = [false; 16];
        VirtualInterrupt {
            irq_pending,
        }
    }

    pub fn set_pending_irq(&mut self, irq: u64) {
        if irq >= 32 { panic!("set_pending_irq: irq {} out of range", irq); }
        self.irq_pending[irq as usize] = true;
    }
    
    pub fn unset_pending_irq(&mut self, irq: u64) {
        if irq >= 32 { panic!("set_pending_irq: irq {} out of range", irq); }
        self.irq_pending[irq as usize] = false;
    }

    pub fn flush_pending_irq(&mut self) {
        for i in 0..self.irq_pending.len() {
            if self.irq_pending[i] {
                unsafe {
                    csrs!(HUVIP, 1 << i);
                }
                self.irq_pending[i] = false;
            } else {
                unsafe {
                    csrc!(HUVIP, 1 << i);
                }
            }
        }
    }
}
