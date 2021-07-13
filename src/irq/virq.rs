use crate::vcpu::utils::*;
use crate::irq::delegation::delegation_constants::IRQ_VS_SOFT;

#[allow(unused)]
pub struct VirtualInterrupt {
    /* 
     * UVTIMER (#16) is not controlled by vcpu.virq field
     * FIXME: use a bit array
     */
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
                unsafe { csrs!(HUVIP, 1 << i); }
            } else {
                unsafe { csrc!(HUVIP, 1 << i); }
            }
        }
    }

    pub fn sync_pending_irq(&mut self) {
        let huvip: u64;
        unsafe { huvip = csrr!(HUVIP); }
        
        let real_vipi = ((huvip >> IRQ_VS_SOFT) & 0x1) == 0x1;
        let pending_vipi = self.irq_pending[IRQ_VS_SOFT as usize];
        if real_vipi && !pending_vipi {
            self.irq_pending[IRQ_VS_SOFT as usize] = true;
        } else if !real_vipi && pending_vipi {
            self.irq_pending[IRQ_VS_SOFT as usize] = false;
        }
    }
}
