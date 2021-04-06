use crate::mm::allocator;
use core::mem;

mod gsmmu_constants {
    pub const PAGE_TABLE_REGION_SIZE: u64 = 1u64 << 25; // 32MB
    pub const PAGE_SIZE: u64 = 1u64 << 12;
    pub const PAGE_SHIFT: u64 = 12;
    pub const PAGE_ORDER: u64 = 9;
    pub const PGD_X_SHIFT: u64 = 2;
}
pub use gsmmu_constants::*;

pub fn import_print() {
    println!("gparegion.rs import");
}

pub struct GpaRegion {
    base_address: u64,
    length: u64,
}

impl GpaRegion {
    pub fn new(base_address: u64, length: u64) -> GpaRegion {
        GpaRegion {
            base_address,
            length,
        }
    }
}

/* pub struct HpaRegion<T: Copy = u64> {
    hpa_ptr: *mut T
    base_address: u64,
    length: u64,
}

// Check in VCPU
impl HpaRegion {
    pub fn new(base_address: u64, length: u64) -> HpaRegion {
        base_address,
        length,
    }
} */

pub struct PageTableRegion {
    pub region: allocator::HpmRegion,
    pub free_offset: u64,
}

impl PageTableRegion {
    // alloc 32MB HpmRegion
    pub fn new(allocator: &mut allocator::Allocator) -> PageTableRegion {
        let region = allocator.hpm_alloc(PAGE_TABLE_REGION_SIZE);
        PageTableRegion {
            region,
            free_offset: 0,
        }
    }

    pub fn page_table_create(&mut self) -> *mut u64 {
        let ptr = self.page_table_alloc(PAGE_SIZE);
        ptr
    }

    // root page table takes 4 pages in SV39x4 & SV48x4
    pub fn root_table_create(&mut self) -> *mut u64 {
        let ptr = self.page_table_alloc(PAGE_SIZE * 4);
        ptr
    }

    // alloc page table from &self.region: HpmRegion
    pub fn page_table_alloc(&mut self, length: u64) -> *mut u64 {
        let u64_size: usize = mem::size_of::<u64>();
        assert_eq!(length % u64_size as u64, 0);
        let offset = self.free_offset;
        let ret_ptr = unsafe { self.region.hpm_ptr.add(offset as usize/ u64_size) };

        // clear the new page table
        let num_byte = length / u64_size as u64;
        let mut ptr = ret_ptr;
        for i in 0..num_byte {
            unsafe {
                *ptr = 0;
                ptr = ptr.add(1);
            }
        }

        // offset update
        self.free_offset += length;

        ret_ptr
    }
}

pub struct GSMMU {
    pub page_table: PageTableRegion,
    gpa_region: Vec<GpaRegion>, // gpa region list
    hpa_region: Vec<allocator::HpmRegion>, // hpa region list
    allocator: allocator::Allocator,
}

/* impl Index<u64> for GSMMU {
    fn index(&self, index: u64) -> &u64 {
        assert!(index < self.length);

        unsafe { &*(self.ptr.add(index)) }
    }
} */

impl GSMMU {
    pub fn new() -> GSMMU {
        let gpa_region: Vec<GpaRegion> = Vec::new();
        let hpa_region: Vec<allocator::HpmRegion> = Vec::new();
        let mut allocator = allocator::Allocator::new();
        let mut page_table = PageTableRegion::new(&mut allocator);

        // create root table
        page_table.root_table_create();

        GSMMU {
            page_table,
            gpa_region,
            hpa_region,
            allocator,
        }
    }

    pub fn test_gsmmu(&self)  {
        println!("GSMMU test start");
        println!("{:?}", &self.page_table.region.hpm_ptr);
    }

    // SV48x4
    pub fn map_page(&self, level: u32, gpa: u64, hpa: u64, flag: u32) -> u32 {
        0
    }



    pub fn set_pte() -> u32 {
        0
    }
}