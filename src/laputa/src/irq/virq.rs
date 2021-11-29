use crate::vcpu::utils::*;
use std::sync::atomic::{AtomicU16, Ordering};
use crate::irq::delegation::delegation_constants::*;

#[allow(unused)]
pub struct VirtualInterrupt {
    /* 
     * UVTIMER (#16) is not controlled by vcpu.virq field
     * FIXME: use a bit array
     */
    irq_pending: AtomicU16,
    irq_pending_mask: AtomicU16,
}

static mut vipi_cnt: usize = 0;
static mut clear_vipi_cnt: usize = 0;
static mut timer_cnt: usize = 0;
static mut clear_timer_cnt: usize = 0;
static mut flush_timer_cnt: usize = 0;

impl VirtualInterrupt {
    pub fn new() -> Self {
        VirtualInterrupt {
            irq_pending: AtomicU16::new(0),
            irq_pending_mask: AtomicU16::new(0),
        }
    }

    pub fn set_pending_irq(&self, irq: u64) {
        if irq >= 16 { panic!("set_pending_irq: irq {} out of range", irq); }
        self.irq_pending.fetch_or(1 << irq, Ordering::SeqCst);
        unsafe { asm!("fence iorw, iorw"); }
        self.irq_pending_mask.fetch_or(1 << irq, Ordering::SeqCst);
    }
    
    pub fn unset_pending_irq(&self, irq: u64) {
        if irq >= 16 { panic!("set_pending_irq: irq {} out of range", irq); }
        self.irq_pending.fetch_and(!(1 << irq), Ordering::SeqCst);
        unsafe { asm!("fence iorw, iorw"); }
        self.irq_pending_mask.fetch_or(1 << irq, Ordering::SeqCst);
    }
    
    pub fn flush_pending_irq(&self) {
        let pending_mask = self.irq_pending_mask.load(Ordering::SeqCst);
        if pending_mask != 0 {
            let prev_mask = self.irq_pending_mask.swap(0, Ordering::SeqCst);
            /* Leave IRQ_U_SOFT for hardware UIPI */
            let pending = self.irq_pending.load(Ordering::SeqCst);
            let val = pending & prev_mask;
            for i in 1..16 {
                if (val & (1 << i)) != 0 {
                    unsafe { csrs!(HUVIP, 1 << i); }
                //} else if (!prev_mask & (1 << i)) == 0 {
                } else if (prev_mask & (1 << i)) != 0 {
                    unsafe { csrc!(HUVIP, 1 << i); }
                }
            }
        }
    }

    pub fn sync_pending_irq(&self) {
        let huvip: u64;
        unsafe { huvip = csrr!(HUVIP); }

        let real_vipi = ((huvip >> IRQ_VS_SOFT) & 0x1) == 0x1;
        let pending = self.irq_pending.load(Ordering::SeqCst);
        let pending_vipi = ((pending >> IRQ_VS_SOFT) & 0x1) == 0x1;
        if real_vipi && !pending_vipi {
            if self.irq_pending_mask.fetch_or(1 << IRQ_VS_SOFT, Ordering::SeqCst)
                & (1 << IRQ_VS_SOFT) == 0 {
                    self.irq_pending.fetch_or(1 << IRQ_VS_SOFT, Ordering::SeqCst);
                    println!("sync_pending_irq real_vipi {:x} pending_vipi {:x}",
                        huvip, pending);
            }
        } else if !real_vipi && pending_vipi {
            if self.irq_pending_mask.fetch_or(1 << IRQ_VS_SOFT, Ordering::SeqCst)
                & (1 << IRQ_VS_SOFT) == 0 {
                    self.irq_pending.fetch_and(!(1 << IRQ_VS_SOFT), Ordering::SeqCst);
            }
        }
    }
}
