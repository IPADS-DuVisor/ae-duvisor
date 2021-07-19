pub trait IrqChip: Send + Sync {
    fn mmio_callback(&self, addr: u64, data: &mut u32, is_write: bool);

    fn trigger_level_irq(&self, irq: u32, level: bool);
    
    fn trigger_edge_irq(&self, irq: u32);
}
