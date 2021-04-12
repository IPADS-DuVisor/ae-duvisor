#[derive(Clone)]
pub struct HpmRegion {
    pub hpm_ptr: *mut u64, // VA
    pub base_address: u64, // HPA
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

    pub fn va_to_hpa(&self, va: u64) -> Option<u64> {
        let va_base = self.hpm_ptr as u64;
        let hpa_base = self.base_address;

        if va < va_base {
            return None;
        }

        let offset: u64 = va - va_base;

        if offset >= self.length {
            return None;
        }
        
        Some(offset + hpa_base)
    }

    pub fn hpa_to_va(&self, hpa: u64) -> Option<u64> {
        let va_base = self.hpm_ptr as u64;
        let hpa_base = self.base_address;

        if hpa < hpa_base {
            return None;
        }

        let offset = hpa - hpa_base;

        if offset >= self.length {
            return None;
        }

        Some(offset + va_base)
    }
}

pub struct HpmAllocator {
    hpm_region_list: Vec<HpmRegion>,
}

impl HpmAllocator {
    pub fn new() -> HpmAllocator {
        HpmAllocator {
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
        let mut allocator = HpmAllocator::new();

        allocator.hpm_alloc(length);

        let mut hpm_length: u64 = 0;

        for i in allocator.hpm_region_list {
            hpm_length = i.length;
        }

        assert_eq!(hpm_length, length);
    }

    // Check hpa_to_va when hpa is out of bound
    #[test]
    fn test_hpa_to_va_oob_invalid() {
        // Valid HPA: [base_addr, base_addr + 0x2000)
        let length = 0x2000;
        let mut allocator = HpmAllocator::new();

        allocator.hpm_alloc(length);

        let mut result;
        let mut invalid_hpa;

        for i in allocator.hpm_region_list {
            invalid_hpa = i.base_address;
            invalid_hpa += i.length * 2;
            result = i.hpa_to_va(invalid_hpa);
            if result.is_some() {
                panic!("HPA {:x} should be out of bound", invalid_hpa);
            }
        }
    }

    // Check hpa_to_va when hpa is equal to the upper boundary
    #[test]
    fn test_hpa_to_va_oob_invalid_eq() {
        // Valid HPA: [base_addr, base_addr + 0x2000)
        let length = 0x2000;
        let mut allocator = HpmAllocator::new();

        allocator.hpm_alloc(length);

        let mut result;
        let mut invalid_hpa;

        for i in allocator.hpm_region_list {
            invalid_hpa = i.base_address;
            invalid_hpa += i.length;
            result = i.hpa_to_va(invalid_hpa);
            if result.is_some() {
                panic!("HPA {:x} should be out of bound", invalid_hpa);
            }
        }
    }

    // Check hpa_to_va when hpa is valid
    #[test]
    fn test_hpa_to_va_oob_valid() {
        // Valid HPA: [base_addr, base_addr + 0x2000)
        let length = 0x2000;
        let mut allocator = HpmAllocator::new();

        allocator.hpm_alloc(length);

        let mut result;
        let mut valid_hpa;

        for i in allocator.hpm_region_list {
            valid_hpa = i.base_address;
            valid_hpa += i.length / 2; 
            result = i.hpa_to_va(valid_hpa);
            if result.is_none() {
                panic!("HPA {:x} should be valid", valid_hpa);
            }
        }
    }

    // Check hpa_to_va when hpa is equal to the lower bound
    #[test]
    fn test_hpa_to_va_oob_valid_eq() {
        // Valid HPA: [base_addr, base_addr + 0x2000)
        let length = 0x2000;
        let mut allocator = HpmAllocator::new();

        allocator.hpm_alloc(length);

        let mut result;
        let mut valid_hpa;

        for i in allocator.hpm_region_list {
            valid_hpa = i.base_address;
            result = i.hpa_to_va(valid_hpa);
            if result.is_none() {
                panic!("HPA {:x} should be valid", valid_hpa);
            }
        }
    }

    // Check va_to_hpa when va is out of bound
    #[test]
    fn test_va_to_hpa_oob_invalid() {
        let length = 0x2000;
        let mut allocator = HpmAllocator::new();

        allocator.hpm_alloc(length);

        let mut result;
        let mut invalid_va;

        for i in allocator.hpm_region_list {
            invalid_va = i.hpm_ptr as u64 + length + 0x1000;
            result = i.va_to_hpa(invalid_va);
            if result.is_some() {
                panic!("VA {:x} should be out of bound", invalid_va);
            }
        }
    }

    // Check va_to_hpa when va is equal to the upper bound
    #[test]
    fn test_va_to_hpa_oob_invalid_eq() {
        let length = 0x2000;
        let mut allocator = HpmAllocator::new();

        allocator.hpm_alloc(length);

        let mut result;
        let mut invalid_va;

        for i in allocator.hpm_region_list {
            invalid_va = i.hpm_ptr as u64 + length;
            result = i.va_to_hpa(invalid_va);
            if result.is_some() {
                panic!("VA {:x} should be out of bound", invalid_va);
            }
        }
    }

    // Check va_to_hpa when va is valid
    #[test]
    fn test_va_to_hpa_oob_valid() {
        let length = 0x2000;
        let mut allocator = HpmAllocator::new();

        allocator.hpm_alloc(length);

        let mut result;
        let mut valid_va;

        for i in allocator.hpm_region_list {
            valid_va = i.hpm_ptr as u64 + length - 0x1000;
            result = i.va_to_hpa(valid_va);
            if result.is_none() {
                panic!("VA {:x} should be valid", valid_va);
            }
        }
    }

    // Check va_to_hpa when va is equal to the lower bound
    #[test]
    fn test_va_to_hpa_oob_valid_eq() {
        let length = 0x2000;
        let mut allocator = HpmAllocator::new();

        allocator.hpm_alloc(length);

        let mut result;
        let mut valid_va;

        for i in allocator.hpm_region_list {
            valid_va = i.hpm_ptr as u64;
            result = i.va_to_hpa(valid_va);
            if result.is_none() {
                panic!("VA {:x} should be valid", valid_va);
            }
        }
    }
}