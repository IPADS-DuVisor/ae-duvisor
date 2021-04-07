pub fn import_print() {
    println!("allocator.rs import");
}

#[derive(Clone)]
pub struct HpmRegion {
    pub hpm_ptr: *mut u64, // VA
    base_address: u64, // HPA
    length: u64,
}

impl HpmRegion {
    pub fn new(hpm_ptr: *mut u64, base_address: u64, length: u64) -> HpmRegion {
        //let hpm_ptr = unsafe { libc::malloc(length) };
        HpmRegion {
            hpm_ptr,
            base_address,
            length,
        }
    }

    pub fn va_to_hpa(&self, va: u64) -> u64 {
        let va_base = self.hpm_ptr as u64;
        let hpa_base = self.base_address;
        let offset = va - va_base;
        offset + hpa_base
    }

    pub fn hpa_to_va(&self, hpa: u64) -> u64 {
        let va_base = self.hpm_ptr as u64;
        let hpa_base = self.base_address;
        let offset = hpa - hpa_base;
        offset + va_base
    }
}

pub struct Allocator {
    hpm_region_list: Vec<HpmRegion>,
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {
            hpm_region_list: Vec::new(),
        }
    }

    // Use malloc for now
    pub fn hpm_alloc(&mut self, length: u64) -> HpmRegion {
        let ptr = unsafe { libc::malloc(length as usize) };
        let hpm_ptr = ptr as *mut u64;
        let base_address = 0x10000;
        let hpm_region = HpmRegion::new(hpm_ptr, base_address, length);
        let hpm_region_return = hpm_region.clone();
        &self.hpm_region_list.push(hpm_region);
        hpm_region_return
    }
}