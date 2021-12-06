use std::sync::Arc;

use BusDevice;
use super::*;
use sys_util::Result;

#[link(name = "lkvm")]
extern "C" {
    fn lkvm_blk_init();

    fn lkvm_blk_mmio_read(addr: u64, data: *mut u8, len: u32);
    
    fn lkvm_blk_mmio_write(addr: u64, data: *const u8, len: u32);
}

pub struct LkvmBlk {}

impl LkvmBlk {
    pub fn init() -> Result<LkvmBlk> {
            unsafe {
                lkvm_blk_init();
            }
            Ok(LkvmBlk {})
    }
}

impl BusDevice for LkvmBlk {
    fn read(&mut self, offset: u64, data: &mut [u8]) {
        unsafe {
            lkvm_blk_mmio_read(offset, data.as_mut_ptr(), data.len() as u32);
        }
    }

    fn write(&mut self, offset: u64, data: &[u8]) {
        unsafe {
            lkvm_blk_mmio_write(offset, data.as_ptr(), data.len() as u32);
        }
    }
}
