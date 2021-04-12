#[derive(Clone)]
pub struct HpmRegion {
    pub hpm_ptr: *mut u64, // VA
    base_address: u64, // HPA
    pub length: u64,
}

impl HpmRegion {
    pub fn new(hpm_ptr: *mut u64, base_address: u64, length: u64) -> HpmRegion {
        HpmRegion {
            hpm_ptr,
            base_address,
            length,
        }
    }

    pub fn va_to_hpa(&self, va: u64) -> u64 {
        let va_base = self.hpm_ptr as u64;
        let hpa_base = self.base_address;

        if va < va_base {
            panic!("HpmRegion::va_to_hpa : va {:x} out of bound", va);
        }

        let offset: u64 = va - va_base;

        if offset >= self.length {
            panic!("HpmRegion::va_to_hpa : va {:x} out of bound", va);
        }
        
        offset + hpa_base
    }

    pub fn hpa_to_va(&self, hpa: u64) -> u64 {
        let va_base = self.hpm_ptr as u64;
        let hpa_base = self.base_address;

        if hpa <= hpa_base {
            panic!("HpmRegion::hpa_to_va : hpa {:x} out of bound", hpa);
        }

        let offset = hpa - hpa_base;

        if offset >= self.length {
            panic!("HpmRegion::hpa_to_va : hpa {:x} out of bound", hpa);
        }

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

        // --- Just for now ---
        let mut base_address = 0x10000;

        for i in &self.hpm_region_list {
            // each hpm_region is separated by 0x1000
            base_address = base_address + i.length + 0x1000;
        }
        // --- End ---

        let hpm_region = HpmRegion::new(hpm_ptr, base_address, length);
        let hpm_region_return = hpm_region.clone();

        &self.hpm_region_list.push(hpm_region);
        
        hpm_region_return
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hpm_region_new() {
        let hpa: u64 = 0x3000;
        let va: u64 = 0x5000;
        let length: u64 = 0x1000;
        let hpm_ptr = va as *mut u64;
        let hpm_region = HpmRegion::new(hpm_ptr, hpa, length);

        assert_eq!(hpm_region.hpm_ptr as u64, va);
        assert_eq!(hpm_region.base_address, hpa);
        assert_eq!(hpm_region.length, length);
    }

    // Check new() of GStageMmu
    #[test]
    fn test_allocator_alloc() { 
        let length = 0x2000;
        let mut allocator = Allocator::new();

        allocator.hpm_alloc(length);

        let mut hpm_length: u64 = 0;

        for i in allocator.hpm_region_list {
            hpm_length = i.length;
        }

        assert_eq!(hpm_length, length);
    }
}