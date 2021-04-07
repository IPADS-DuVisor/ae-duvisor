use crate::mm::allocator;
use core::mem;

mod gsmmu_constants {
    pub const PAGE_TABLE_REGION_SIZE: u64 = 1u64 << 25; // 32MB
    pub const PAGE_SIZE: u64 = 1u64 << 12;
    pub const PAGE_SHIFT: u64 = 12;
    pub const PAGE_ORDER: u64 = 9;
    //pub const PGD_X_SHIFT: u64 = 2;

    // permission flag
    pub const GS_PAGE_ATTR_R: u64 = 1;
    pub const GS_PAGE_ATTR_W: u64 = 2;
    pub const GS_PAGE_ATTR_X: u64 = 4;

    // pte bit
    pub const PTE_VALID: u64 = 1u64 << 0;
    pub const PTE_READ: u64 = 1u64 << 1;
    pub const PTE_WRITE: u64 = 1u64 << 2;
    pub const PTE_EXECUTE: u64 = 1u64 << 3;
    pub const PTE_USER: u64 = 1u64 << 4;
    //pub const PTE_GLOBAL: u64 = 1u64 << 5;
    //pub const PTE_ACCESS: u64 = 1u64 << 6;
    //pub const PTE_DIRTY: u64 = 1u64 << 6;

    pub const PTE_PPN_SHIFT: u64 = 10;
}
pub use gsmmu_constants::*;

pub struct GpaRegion {
    pub base_address: u64,
    pub length: u64,
}

impl GpaRegion {
    pub fn new(base_address: u64, length: u64) -> GpaRegion {
        GpaRegion {
            base_address,
            length,
        }
    }
}

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
        for _ in 0..num_byte {
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

    // For debug
    pub fn gsmmu_test(&mut self)  {
        self.map_page(0x1000, 0x2000, 0x5);
        self.gpa_region_add(0x3000, 0x4000);
        self.hpa_region_add(0x10000);
    }

    pub fn set_pte_flags(mut pte: u64, level: u64, flag: u64) -> u64 {
        // for ULH in HU
        pte = pte | PTE_USER;
        
        match level {
            3 => {
                pte = pte | PTE_VALID;

                if (flag & GS_PAGE_ATTR_R) != 0 {
                    pte = pte | PTE_READ;
                }

                if (flag & GS_PAGE_ATTR_W) != 0 {
                    pte = pte | PTE_WRITE;
                }

                if (flag & GS_PAGE_ATTR_X) != 0 {
                    pte = pte | PTE_EXECUTE;
                }
            },
            _ => {
                pte = pte | PTE_VALID;
            }
        }

        pte
    }

    //TODO: query_page

    pub fn gpa_to_offset(&mut self, gpa: u64) -> u64 {
        let mut page_table_va = self.page_table.region.hpm_ptr as u64;
        let mut page_table_hpa;// = self.page_table.region.va_to_hpa(page_table_va);
        let mut index: u64;

        for level in 0..3 {
            index = (gpa >> (39 - PAGE_ORDER * level)) & 0x1ff;
            if level == 0 {
                index = (gpa >> (39 - PAGE_ORDER * level)) & 0x7ff;
            }
            let pte_addr_va = page_table_va + index * 8;
            let mut pte = unsafe { *(pte_addr_va as *mut u64) };

            if pte & PTE_VALID == 0 {
                page_table_va = self.page_table.page_table_create() as u64;
                page_table_hpa = self.page_table.region.va_to_hpa(page_table_va);
                pte = page_table_hpa >> (PAGE_SHIFT - PTE_PPN_SHIFT);
                pte = GSMMU::set_pte_flags(pte, level, 0);
                unsafe {
                    *(pte_addr_va as *mut u64) = pte;         
                }
            } else {
                page_table_hpa = (pte >> PTE_PPN_SHIFT) << PAGE_SHIFT;
                page_table_va = self.page_table.region.hpa_to_va(page_table_hpa);
            }
        }

        index = (gpa >> 12) & 0x1ff;
        page_table_va + index * 8 - (self.page_table.region.hpm_ptr as u64)
    }

    // SV48x4
    pub fn map_page(&mut self, gpa: u64, hpa: u64, flag: u64) -> u32 {
        let offset = self.gpa_to_offset(gpa);
        let page_table_va = self.page_table.region.hpm_ptr as u64;
        let pte_addr = page_table_va + offset;

        assert_eq!(hpa & 0xfff, 0);

        let mut pte = hpa >> (PAGE_SHIFT - PTE_PPN_SHIFT);
        pte = GSMMU::set_pte_flags(pte, 3, flag);

        let pte_addr_ptr = pte_addr as *mut u64;
        unsafe {
            *pte_addr_ptr = pte;
        }

        0
    }

    pub fn gpa_region_add(&mut self, base_address: u64, length: u64) -> u32 {
        let gpa_region = GpaRegion::new(base_address, length);
        self.gpa_region.push(gpa_region);

        0
    }

    pub fn hpa_region_add(&mut self, length: u64) -> u32 {
        let hpa_region = self.allocator.hpm_alloc(length);
        self.hpa_region.push(hpa_region);

        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Check new() of GSMMU
    #[test]
    fn test_gsmmu_new() { 
        let gsmmu = GSMMU::new();

        // Check the root table has been created
        let free_offset = gsmmu.page_table.free_offset;
        assert_eq!(free_offset, 16384);

        // Check the root table has been cleared
        let mut root_ptr = gsmmu.page_table.region.hpm_ptr;
        unsafe {
            root_ptr = root_ptr.add(10);
        }
        let pte: u64 = unsafe { *root_ptr };
        assert_eq!(pte, 0);
    }

    // Check gpa_region add
    #[test]
    fn test_gpa_region_add() { 
        let mut gsmmu = GSMMU::new();
        let mut base_address: u64 = 0;
        let mut length: u64 = 0;

        gsmmu.gpa_region_add(0x1000, 0x2000);

        for i in gsmmu.gpa_region {
            base_address = i.base_address;
            length = i.length;
        }

        assert_eq!(base_address, 0x1000);
        assert_eq!(length, 0x2000);
    }

    // Check hpa_region add
    #[test]
    fn test_hpa_region_add() { 
        let mut gsmmu = GSMMU::new();
        let mut length: u64 = 0;

        gsmmu.hpa_region_add(0x1000);

        for i in gsmmu.hpa_region {
            length = i.length;
        }

        assert_eq!(length, 0x1000);
    }

    #[test]
    fn test_page_table_create() {
        let mut gsmmu = GSMMU::new();

        // Create a page table
        gsmmu.page_table.page_table_create();

        // Check the page table has been created
        let free_offset = gsmmu.page_table.free_offset;
        assert_eq!(free_offset, 16384 + 4096);

        // Check the page table has been cleared
        let root_ptr = gsmmu.page_table.region.hpm_ptr;
        let ptr: *mut u64;
        unsafe {
            ptr = root_ptr.add(512+10);
        }
        let pte: u64 = unsafe { *ptr };
        assert_eq!(pte, 0);
    }

    // Check the value of the L4 PTE
    #[test]
    fn test_map_page_pte() {
        let mut gsmmu = GSMMU::new();

        // Create a page table
        gsmmu.map_page(0x1000, 0x2000, 0x5);

        // Check the pte
        let root_ptr = gsmmu.page_table.region.hpm_ptr;
        let ptr: *mut u64;
        unsafe {
            ptr = root_ptr.add(512*6+1);
        }
        let pte: u64 = unsafe { *ptr };

        // PTE on L4 should be 0b1000 0001 1011
        // ppn = 0b10 with PTE_USER/EXECUTE/READ/VALID
        assert_eq!(pte, 2075);
    }

    // Check the location(index) of the new PTEs
    #[test]
    fn test_map_page_index() {
        let mut gsmmu = GSMMU::new();
        let root_ptr = gsmmu.page_table.region.hpm_ptr;
        let mut ptr: *mut u64;
        let mut pte: u64;

        // Change 4 PTEs
        gsmmu.map_page(0x1000, 0x2000, 0x7); 

        // Non-zero [0, 512*4, 512*5, 512*6+1]
        //let pte_index:[u64; 4] = [0, 512*4, 512*5, 512*6+1];
        let pte_index = vec![0, 512*4, 512*5, 512*6+1];

        // 4 PTEs should be set
        for i in &pte_index {
            unsafe {
                ptr = root_ptr.add(*i);
                pte = *ptr;
            }

            assert_ne!(pte, 0);
        }

        // All the other PTEs should be zero
        for i in (0..512*7).filter(|x: &usize| !pte_index.contains(x)) {
            unsafe {
                ptr = root_ptr.add(i as usize);
                pte = *ptr;
            }

            assert_eq!(pte, 0);
        }
    }
}