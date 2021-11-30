use std::sync::Arc;

use BusDevice;
use super::*;
use sys_util::{EventFd, GuestAddress, GuestMemory, Result};
use irq_util::IrqChip;

#[link(name = "lkvm")]
extern "C" {
    fn lkvm_net_init();

    fn lkvm_net_mmio_read(addr: u64, data: *mut u8, len: u32);
    
    fn lkvm_net_mmio_write(addr: u64, data: *const u8, len: u32);
}

pub struct LkvmDevice {
    mem: GuestMemory,
    irqchip: Arc<dyn IrqChip>,
}

impl LkvmDevice {
    pub fn init(mem: GuestMemory, irqchip: Arc<dyn IrqChip>) {
        unsafe {
            lkvm_net_init();
        }
    }
}

impl BusDevice for LkvmDevice {
    fn read(&mut self, offset: u64, data: &mut [u8]) {
        unsafe {
            lkvm_net_mmio_read(offset, data.as_mut_ptr(), data.len() as u32);
        }
    }

    fn write(&mut self, offset: u64, data: &[u8]) {
        unsafe {
            lkvm_net_mmio_write(offset, data.as_ptr(), data.len() as u32);
        }
    }
}
