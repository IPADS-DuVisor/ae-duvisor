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