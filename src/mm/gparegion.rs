use crate::mm::allocator;
use core::mem;

mod gsmmu_constants {
    pub const PAGE_TABLE_REGION_SIZE: u64 = 1u64 << 25; // 32MB
    pub const PAGE_SIZE: u64 = 1u64 << 12;
    pub const PAGE_SHIFT: u64 = 12;
    pub const PAGE_ORDER: u64 = 9;
    pub const PGD_X_SHIFT: u64 = 2;

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
    pub const PTE_GLOBAL: u64 = 1u64 << 5;
    pub const PTE_ACCESS: u64 = 1u64 << 6;
    pub const PTE_DIRTY: u64 = 1u64 << 6;

    pub const PTE_PPN_SHIFT: u64 = 10;
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

    pub fn set_pte_flags(mut pte: u64, level: u64, flag: u64) -> u64 {
        // for ULH in HU
        pte = pte | PTE_USER;
        
        match level {
            4 => {
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

    // gsmmu.map_page(gsmmu.page_table.region.hpm_ptr, 1, gpa, hpa, flag)
    // SV48x4
    // current_ptp VA
    pub fn map_page(&mut self, current_ptp: *mut u64, level: u64, gpa: u64, hpa: u64, flag: u64) -> u32 {
        let mut shift = 0;
        let mut index = 0;
        let mut next_level_ptp: *mut u64;
        let mut pte: u64;

        shift = (4 - level) * PAGE_ORDER + PAGE_SHIFT;

        println!("map_page current_ptp {:?}", current_ptp);

        match level {
            1 => {
                let index_mask = (1u64 << (PAGE_ORDER + PGD_X_SHIFT)) - 1;
                index = (gpa >> shift) & index_mask;
                unsafe { 
                    pte = *(current_ptp.add(index as usize)); 
                }

                if pte & PTE_VALID == 0 { // L1 pte is not valid
                    next_level_ptp = self.page_table.page_table_create();
                    let next_level_ptp_hpa = self.page_table.region.va_to_hpa(next_level_ptp as u64);
                    pte = (next_level_ptp_hpa >> PAGE_SHIFT) << PTE_PPN_SHIFT;
                    println!("map_page level 1 next_level_ptp {:?}", next_level_ptp);
                    println!("map_page level 1 next_level_ptp_hpa {:?}", next_level_ptp_hpa);
                    println!("map_page level 1 gpa {} shift {} mask {}", gpa, shift, index_mask);
                    println!("map_page level 1 index {}", index);
                    println!("map_page level 1 pte {:?}", pte);
                } else { // L1 pte is valid
                    let ppn = pte >> PTE_PPN_SHIFT & ((1u64 << 44) - 1);
                    next_level_ptp = (ppn << PAGE_SHIFT) as *mut u64;
                    next_level_ptp = (self.page_table.region.hpa_to_va(next_level_ptp as u64)) as *mut u64;
                }

                pte = GSMMU::set_pte_flags(pte, 1, flag);
                unsafe { 
                    *current_ptp.add(index as usize) = pte; 
                    println!("pte {:?}", *(current_ptp.add(index as usize)));
                };
                self.map_page(next_level_ptp, 2, gpa, hpa, flag);
            },
            2 => {
                let index_mask = (1u64 << PAGE_ORDER) - 1;
                index = (gpa >> shift) & index_mask;
                unsafe { 
                    pte = *(current_ptp.add(index as usize)); 
                }

                if pte & PTE_VALID == 0 { // L2 pte is not valid
                    next_level_ptp = self.page_table.page_table_create();
                    let next_level_ptp_hpa = self.page_table.region.va_to_hpa(next_level_ptp as u64);
                    pte = (next_level_ptp_hpa >> PAGE_SHIFT) << PTE_PPN_SHIFT;
                    println!("map_page level 2 next_level_ptp {:?}", next_level_ptp);
                    println!("map_page level 2 next_level_ptp_hpa {:?}", next_level_ptp_hpa);
                    println!("map_page level 2 gpa {} shift {} mask {}", gpa, shift, index_mask);
                    println!("map_page level 2 index {}", index);
                    println!("map_page level 2 pte {:?}", pte);
                } else { // L2 pte is valid
                    let ppn = pte >> PTE_PPN_SHIFT & ((1u64 << 44) - 1);
                    next_level_ptp = (ppn << PAGE_SHIFT) as *mut u64;
                    next_level_ptp = (self.page_table.region.hpa_to_va(next_level_ptp as u64)) as *mut u64;
                }

                pte = GSMMU::set_pte_flags(pte, 2, flag);
                unsafe { 
                    *current_ptp.add(index as usize) = pte; 
                    println!("pte {:?}", *(current_ptp.add(index as usize)));
                };
                self.map_page(next_level_ptp, 3, gpa, hpa, flag);
            },
            3 => {
                let index_mask = (1u64 << PAGE_ORDER) - 1;
                index = (gpa >> shift) & index_mask;
                unsafe { 
                    pte = *(current_ptp.add(index as usize)); 
                }

                if pte & PTE_VALID == 0 { // L3 pte is not valid
                    next_level_ptp = self.page_table.page_table_create();
                    let next_level_ptp_hpa = self.page_table.region.va_to_hpa(next_level_ptp as u64);
                    pte = (next_level_ptp_hpa >> PAGE_SHIFT) << PTE_PPN_SHIFT;
                    println!("map_page level 3 next_level_ptp {:?}", next_level_ptp);
                    println!("map_page level 3 next_level_ptp_hpa {:?}", next_level_ptp_hpa);
                    println!("map_page level 3 gpa {} shift {} mask {}", gpa, shift, index_mask);
                    println!("map_page level 3 index {}", index);
                    println!("map_page level 3 pte {:?}", pte);
                } else { // L3 pte is valid
                    let ppn = pte >> PTE_PPN_SHIFT & ((1u64 << 44) - 1);
                    next_level_ptp = (ppn << PAGE_SHIFT) as *mut u64;
                    next_level_ptp = (self.page_table.region.hpa_to_va(next_level_ptp as u64)) as *mut u64;
                }

                pte = GSMMU::set_pte_flags(pte, 3, flag);
                unsafe { 
                    *current_ptp.add(index as usize) = pte; 
                    println!("pte {:?}", *(current_ptp.add(index as usize)));
                };
                self.map_page(next_level_ptp, 4, gpa, hpa, flag);

            },
            4 => {
                let index_mask = (1u64 << PAGE_ORDER) - 1;
                index = (gpa >> shift) & index_mask;
                unsafe { 
                    pte = *(current_ptp.add(index as usize)); 
                }

                if pte & PTE_VALID == 0 { // L4 pte is not valid
                    /* next_level_ptp = self.page_table.page_table_create();
                    let next_level_ptp_hpa = self.page_table.region.va_to_hpa(next_level_ptp as u64); */
                    pte = hpa >> 2;
                    /* println!("map_page level 4 next_level_ptp {:?}", next_level_ptp);
                    println!("map_page level 4 next_level_ptp_hpa {:?}", next_level_ptp_hpa); */
                    println!("map_page level 4 current_ptp {:?}", current_ptp);
                    println!("map_page level 4 gpa {} shift {} mask {}", gpa, shift, index_mask);
                    println!("map_page level 4 index {}", index);
                    println!("map_page level 4 pte {:?}", pte);
                }

                pte = GSMMU::set_pte_flags(pte, 4, flag);
                unsafe { 
                    *current_ptp.add(index as usize) = pte; 
                    println!("pte {:?}", *(current_ptp.add(index as usize)));
                };
            },
            _ => panic!(),
        }

        0
    }

    pub fn set_pte() -> u32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Check new() of GSMMU
    #[test]
    fn test_gsmmu_new() { 
        let mut gsmmu = GSMMU::new();

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

        /* gsmmu.test_gsmmu();
        let ptr = gsmmu.page_table.root_table_create();
        println!("{:?}", ptr);
        let offset = gsmmu.page_table.free_offset;
        println!("{:?}", offset);
        gsmmu.map_page(gsmmu.page_table.region.hpm_ptr, 1, 0x1000, 0x2000, 0x7); */
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
        let mut root_ptr = gsmmu.page_table.region.hpm_ptr;
        let mut ptr: *mut u64;
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
        gsmmu.map_page(gsmmu.page_table.region.hpm_ptr, 1, 0x1000, 0x2000, 0x5);

        // Check the pte
        let mut root_ptr = gsmmu.page_table.region.hpm_ptr;
        let mut ptr: *mut u64;
        unsafe {
            ptr = root_ptr.add(512*6+1);
        }
        println!("test_map_page ptr {:?}", ptr);
        let pte: u64 = unsafe { *ptr };

        // PTE on L4 should be 0b1000 0001 1011
        // ppn = 0b10 with PTE_USER/EXECUTE/READ/VALID
        assert_eq!(pte, 2075);
    }

    // Check the location(index) of the new PTEs
    #[test]
    fn test_map_page_index() {
        let mut gsmmu = GSMMU::new();
        let mut root_ptr = gsmmu.page_table.region.hpm_ptr;
        let mut ptr: *mut u64;
        let mut pte: u64;

        // Change 4 PTEs
        gsmmu.map_page(gsmmu.page_table.region.hpm_ptr, 1, 0x1000, 0x2000, 0x7); 

        // Non-zero [0, 512*4, 512*5, 512*6+1]
        let pte_index:[u64; 4] = [0, 512*4, 512*5, 512*6+1];

        for i in 0..512*7 {
            unsafe {
                ptr = root_ptr.add(i);
                pte = *ptr;
            }

            let mut flag: u64 = 0;

            for j in 0..4 {
                let k = i as u64;
                if k == pte_index[j] {
                    flag += 1;
                }
            }

            if flag == 0 {
                assert_eq!(pte, 0);
            } else {
                assert_ne!(pte, 0);
            }
        }
    }
}