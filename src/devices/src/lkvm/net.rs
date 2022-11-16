use std::sync::Arc;

use BusDevice;
use super::*;
use sys_util::Result;

#[link(name = "lkvm")]
extern "C" {
    fn lkvm_net_init(fd: i32, vplic_ptr: u64);

    fn lkvm_net_mmio_read(addr: u64, data: *mut u8, len: u32);
    
    fn lkvm_net_mmio_write(addr: u64, data: *const u8, len: u32);
}

pub struct LkvmNet {}

impl LkvmNet {
    pub fn init(ioctl_fd: i32, vplic_ptr: u64) -> Result<LkvmNet> {
            unsafe {
                lkvm_net_init(ioctl_fd, vplic_ptr);
            }
            Ok(LkvmNet {})
    }
}

impl BusDevice for LkvmNet {
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
