pub trait IrqChip: Send + Sync {
    /* 
     * Handle MMIO read and write requests from guest VM
     *
     * @addr: the MMIO fault address
     * @data: for an MMIO read, the value from target field is loaded into @data,
     * for an MMIO write, @data is stored in to the target field
     * @is_write: false for MMIO read, true for MMIO write
     */
    fn mmio_callback(&self, addr: u64, data: &mut u32, is_write: bool);

    /* 
     * Handle interrupt insertion from external devices (e.g., console, virtio devices)
     * Currently, only level-triggered interrupts are supported
     *
     * @irq: the IRQ number (0-1023] to be inserted by an external device
     * @level: false for clearing interrupt, true for setting interrupt
     */
    fn trigger_irq(&self, irq: u32, level: bool);
}
