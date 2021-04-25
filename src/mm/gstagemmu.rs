use crate::mm::hpmallocator;
use crate::mm::gparegion;
use core::mem;

mod gsmmu_constants {
    pub const PAGE_TABLE_REGION_SIZE: u64 = 1u64 << 25; // 32MB for now
    pub const PAGE_SIZE: u64 = 1u64 << 12;
    pub const PAGE_SHIFT: u64 = 12;
    pub const PAGE_ORDER: u64 = 9;

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

pub struct Pte {
    // The offset of this pte from the top of the root table (page_table.region.hpm_vptr)
    // Access the pte by (page_table.region.hpm_vptr + offset) as *mut u64
    pub offset: u64,
    pub value: u64,
    pub level: u32,
}

impl Pte {
    pub fn new(offset: u64, value: u64, level: u32) -> Self {
        Self {
            offset,
            value,
            level,
        }
    }
}

pub struct PageTableRegion {
    pub region: hpmallocator::HpmRegion,
    pub free_offset: u64,
}

impl PageTableRegion {
    pub fn new(allocator: &mut hpmallocator::HpmAllocator) -> Self {
        let region = allocator.hpm_alloc(PAGE_TABLE_REGION_SIZE);
        Self {
            region,
            free_offset: 0,
        }
    }

    pub fn page_table_create(&mut self, level: u64) -> *mut u64 {
        let mut size: u64 = PAGE_SIZE;

        // root page table takes 4 pages in SV39x4 & SV48x4
        if level == 0 {
            size = PAGE_SIZE * 4;
        }

        let ptr = self.page_table_alloc(size);
        ptr
    }

    // alloc page table from &self.region: HpmRegion
    pub fn page_table_alloc(&mut self, length: u64) -> *mut u64 {
        let u64_size: usize = mem::size_of::<u64>();
        assert_eq!(length % u64_size as u64, 0);

        let total_length: u64 = self.free_offset + length;
        if total_length > self.region.length {
            panic!("PageTableRegion::page_table_alloc : length {} out of bound", length);
        }

        let offset = self.free_offset;
        let ret_ptr = unsafe { self.region.hpm_vptr.add(offset as usize/ u64_size) };


        // clear the new page table
        let ptr = ret_ptr as *mut libc::c_void;
        unsafe { libc::memset(ptr, 0, length as usize); }

        // offset update
        self.free_offset += length;

        ret_ptr
    }
}

#[allow(unused)]
pub struct GStageMmu {
    pub page_table: PageTableRegion,
    gpa_regions: Vec<gparegion::GpaRegion>, // gpa region list
    allocator: hpmallocator::HpmAllocator,
}

impl GStageMmu {
    pub fn new() -> Self {
        let gpa_regions: Vec<gparegion::GpaRegion> = Vec::new();
        let mut allocator = hpmallocator::HpmAllocator::new();
        let mut page_table = PageTableRegion::new(&mut allocator);

        // create root table
        page_table.page_table_create(0);

        Self {
            page_table,
            gpa_regions,
            allocator,
        }
    }

    // For debug
    pub fn gsmmu_test(&mut self)  {
        self.map_page(0x1000, 0x2000, PTE_READ | PTE_EXECUTE);
        self.unmap_page(0x1000);
        self.map_range(0x1000, 0x2000, 0x2000, 
            PTE_READ | PTE_WRITE | PTE_EXECUTE);
        self.unmap_range(0x1000, 0x2000);
        self.map_query(0x1000);
        self.gpa_region_add(0x3000, 0x4000, 0x1000);
        self.map_protect(0x1000, PTE_READ | PTE_EXECUTE);
    }

    pub fn set_pte_flag(mut pte: u64, level: u64, flag: u64) -> u64 {
        pte = pte & 0xFFFFFFFFFFFFFC00;
       
        match level {
            3 => {
                pte = pte | PTE_VALID;

                if (flag & PTE_READ) != 0 {
                    pte = pte | PTE_READ;
                }

                if (flag & PTE_WRITE) != 0 {
                    pte = pte | PTE_WRITE;
                }

                if (flag & PTE_EXECUTE) != 0 {
                    pte = pte | PTE_EXECUTE;
                }
                
                if (flag & PTE_USER) != 0 {
                    pte = pte | PTE_USER;
                }

                if (flag & PTE_GLOBAL) != 0 {
                    pte = pte | PTE_GLOBAL;
                }

                if (flag & PTE_ACCESS) != 0 {
                    pte = pte | PTE_ACCESS;
                }

                if (flag & PTE_DIRTY) != 0 {
                    pte = pte | PTE_DIRTY;
                }
            },
            _ => {
                pte = pte | PTE_VALID;
            }
        }

        pte
    }

    pub fn gpa_to_ptregion_offset(&mut self, gpa: u64) -> Option<[u64; 4]> {
        let mut page_table_va = self.page_table.region.hpm_vptr as u64;
        let mut page_table_va_wrap;
        let mut page_table_hpa;
        let mut page_table_hpa_wrap;
        let mut index: u64;
        let mut shift: u64;
        let mut offsets = [0; 4];

        for level in 0..3 {
            shift = 39 - PAGE_ORDER * level;
            index = (gpa >> shift) & 0x1ff;
            if level == 0 {
                index = (gpa >> shift) & 0x7ff;
            }
            let pte_addr_va = page_table_va + index * 8;
            let mut pte = unsafe { *(pte_addr_va as *mut u64) };

            if pte & PTE_VALID == 0 {
                page_table_va = self.page_table.page_table_create(level + 1) as u64;
                page_table_hpa_wrap = self.page_table.region.va_to_hpa(page_table_va);
                if page_table_hpa_wrap.is_none() {
                    return None;
                }
                page_table_hpa = page_table_hpa_wrap.unwrap();
                pte = (page_table_hpa >> PAGE_SHIFT) << PTE_PPN_SHIFT;
                pte = GStageMmu::set_pte_flag(pte, level, 0);
                unsafe {
                    *(pte_addr_va as *mut u64) = pte;         
                }
            } else {
                page_table_hpa = (pte >> PTE_PPN_SHIFT) << PAGE_SHIFT;
                page_table_va_wrap = self.page_table.region.hpa_to_va(page_table_hpa);
                if page_table_va_wrap.is_none() {
                    return None;
                }
                page_table_va = page_table_va_wrap.unwrap();
            }
            offsets[level as usize] = pte_addr_va - (self.page_table.region.hpm_vptr as u64);
        }

        index = ((gpa >> 12) & 0x1ff) * 8;
        offsets[3] = page_table_va + index - (self.page_table.region.hpm_vptr as u64);
        Some(offsets)
    }

    pub fn map_query(&mut self, gpa: u64) -> Option<Pte> {
        let mut page_table_va = self.page_table.region.hpm_vptr as u64;
        let mut page_table_hpa;
        let mut index: u64;
        let mut pte_offset: u64 = 0;
        let mut pte_value: u64 = 0;
        let mut pte_level: u32 = 0;
        let mut page_table_va_wrap;
        let mut shift;

        for level in 0..4 {
            shift = 39 - PAGE_ORDER * level;
            index = (gpa >> shift) & 0x1ff;
            if level == 0 {
                index = (gpa >> shift) & 0x7ff;
            }
            let pte_addr_va = page_table_va + index * 8;
            let pte = unsafe { *(pte_addr_va as *mut u64) };

            if pte & PTE_VALID == 0 {
                break;
            } else {
                pte_offset = pte_addr_va - (self.page_table.region.hpm_vptr as u64);
                pte_value = pte;
                pte_level = level as u32;

                if level == 3 {
                    break;
                }

                page_table_hpa = (pte >> PTE_PPN_SHIFT) << PAGE_SHIFT;
                page_table_va_wrap = self.page_table.region.hpa_to_va(page_table_hpa);
                if page_table_va_wrap.is_none() {
                    return None;
                }
                page_table_va = page_table_va_wrap.unwrap();
            }
        }

        if pte_value == 0 {
            return None;
        }

        return Some(Pte::new(pte_offset, pte_value, pte_level));
    }

    pub fn map_protect(&mut self, gpa: u64, flag: u64) -> Option<u32> {
        let pte: Pte;
        let mut pte_offset: u64 = 0;
        let mut pte_value: u64 = 0;
        let mut pte_level: u32 = 0;

        let query = self.map_query(gpa);
        if query.is_some() {
            pte = query.unwrap();
            pte_offset = pte.offset;
            pte_value = pte.value;
            pte_level = pte.level;
        }

        if pte_level != 3 {
            return None; // No mapping, and nothing changed
        }

        let page_table_va = self.page_table.region.hpm_vptr as u64;
        let pte_addr = (page_table_va + pte_offset) as *mut u64;
        pte_value = GStageMmu::set_pte_flag(pte_value, pte_level as u64, flag);
        unsafe {
            *pte_addr = pte_value;
        }

        Some(0)
    }

    // SV48x4
    pub fn map_page(&mut self, gpa: u64, hpa: u64, flag: u64) -> Option<u32> {
        let offsets_wrap = self.gpa_to_ptregion_offset(gpa);
        if offsets_wrap.is_none() {
            return None;
        }
        let offset = offsets_wrap.unwrap()[3];
        let page_table_va = self.page_table.region.hpm_vptr as u64;
        let pte_addr = page_table_va + offset;

        if (hpa & 0xfff) != 0 {
            return None;
        }

        if (gpa & 0xfff) != 0 {
            return None;
        }

        let mut pte = hpa >> (PAGE_SHIFT - PTE_PPN_SHIFT);
        pte = GStageMmu::set_pte_flag(pte, 3, flag);

        let pte_addr_ptr = pte_addr as *mut u64;
        unsafe {
            *pte_addr_ptr = pte;
        }

        Some(0)
    }

    pub fn map_range(&mut self, gpa: u64, hpa: u64, length: u64, flag: u64) -> Option<u32> {
        if (hpa & 0xfff) != 0 {
            return None;
        }

        if (gpa & 0xfff) != 0 {
            return None;
        }
        
        if (length & 0xfff) != 0 {
            return None;
        }

        let mut offset: u64 = 0;

        loop {
            self.map_page(gpa + offset, hpa + offset, flag);
            offset += PAGE_SIZE;
            if offset >= length {
                break;
            }
        }

        Some(0)
    }

    fn is_empty_page(pte_addr :u64) -> bool {
        let pte_addr = pte_addr & (!0xfff);
        let mut index = 0;
        let mut empty_flag = true;
        let mut pte_val;
        while index < PAGE_SIZE {
            let pte_addr_ptr = ( pte_addr + index ) as *mut u64;
            pte_val = unsafe { *pte_addr_ptr };
            if pte_val != 0 {
                empty_flag = false;
                break;
            }
            index += 8;
        }
        empty_flag
    }

    // Unmap L0/L1/L2 page table pages if there are no valid PTEs in them
    pub fn unmap_page(&mut self, gpa: u64) -> Option<u32> {
        if (gpa & 0xfff) != 0 {
            return None;
        }

        let offsets_wrap = self.gpa_to_ptregion_offset(gpa);
        if offsets_wrap.is_none() {
            return None;
        }
        let offsets = offsets_wrap.unwrap();
        for level in 0..4 {
            let offset = offsets[(3 - level) as usize];
            let page_table_va = self.page_table.region.hpm_vptr as u64;
            let pte_addr = page_table_va + offset;
    
            let pte_addr_ptr = pte_addr as *mut u64;
            unsafe {
                *pte_addr_ptr = 0;
            }
            if !GStageMmu::is_empty_page(pte_addr) {
                break;
            }
        }

        Some(0)
    }

    pub fn unmap_range(&mut self, gpa: u64, length: u64) -> Option<u32> {
        if (gpa & 0xfff) != 0 {
            return None;
        }
        
        if (length & 0xfff) != 0 {
            return None;
        }

        let mut offset: u64 = 0;

        loop {
            self.unmap_page(gpa + offset);
            offset += PAGE_SIZE;
            if offset >= length {
                break;
            }
        }

        Some(0)
    }

    pub fn gpa_region_add(&mut self, gpa: u64, hpa: u64, length: u64) {
        let gpa_region = gparegion::GpaRegion::new(gpa, hpa, length);
        self.gpa_regions.push(gpa_region);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Check new() of GStageMmu
    #[test]
    fn test_gsmmu_new() { 
        let gsmmu = GStageMmu::new();

        // Check the root table has been created
        let free_offset = gsmmu.page_table.free_offset;
        assert_eq!(free_offset, 0x4000);

        // Check the root table has been cleared
        let mut root_ptr = gsmmu.page_table.region.hpm_vptr;
        unsafe {
            root_ptr = root_ptr.add(10);
        }
        let pte: u64 = unsafe { *root_ptr };
        assert_eq!(pte, 0);
    }

    // Check gpa_region add
    #[test]
    fn test_gpa_region_add() { 
        let mut gsmmu = GStageMmu::new();
        let mut gpa: u64 = 0;
        let mut hpa: u64 = 0;
        let mut length: u64 = 0;

        gsmmu.gpa_region_add(0x1000, 0x4000, 0x2000);

        for i in gsmmu.gpa_regions {
            gpa = i.gpa;
            hpa = i.hpa;
            length = i.length;
        }

        assert_eq!(gpa, 0x1000);
        assert_eq!(hpa, 0x4000);
        assert_eq!(length, 0x2000);
    }

    #[test]
    fn test_page_table_create() {
        let mut gsmmu = GStageMmu::new();

        // Create a page table
        gsmmu.page_table.page_table_create(1);

        // Check the page table has been created
        let free_offset = gsmmu.page_table.free_offset;
        assert_eq!(free_offset, 0x4000 + 0x1000);

        // Check the page table has been cleared
        let root_ptr = gsmmu.page_table.region.hpm_vptr;
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
        let mut gsmmu = GStageMmu::new();

        // Create a page table
        gsmmu.map_page(0x1000, 0x2000, PTE_READ | PTE_EXECUTE);

        // Check the pte
        let root_ptr = gsmmu.page_table.region.hpm_vptr;
        let ptr: *mut u64;
        unsafe {
            ptr = root_ptr.add(512*6+1);
        }
        let pte: u64 = unsafe { *ptr };

        // PTE on L4 should be 0b1000 0000 1011
        // ppn = 0b10 with PTE_EXECUTE/READ/VALID
        assert_eq!(pte, 2059);
    }

    // Check the location(index) of the new PTEs
    #[test]
    fn test_map_page_index() {
        let mut gsmmu = GStageMmu::new();
        let root_ptr = gsmmu.page_table.region.hpm_vptr;
        let mut ptr: *mut u64;
        let mut pte: u64;
        let gpa = 0x1000;
        let hpa = 0x2000;

        // Change 4 PTEs
        gsmmu.map_page(gpa, hpa, PTE_READ | PTE_WRITE | PTE_EXECUTE); 

        // Non-zero [0, 512*4, 512*5, 512*6+1]
        let pte_index = vec![0, 512*4, 512*5, 512*6+1];

        // non-zero answer
        let base_address = gsmmu.page_table.region.base_address;
        let l0_pte = ((base_address + 0x4000) >> 2) | PTE_VALID;
        let l1_pte = ((base_address + 0x5000) >> 2) | PTE_VALID;
        let l2_pte = ((base_address + 0x6000) >> 2) | PTE_VALID;
        let l3_pte = (hpa >> 2) 
            | PTE_VALID | PTE_READ | PTE_WRITE | PTE_EXECUTE;

        // Start from 0x10000 and the root table takes 0x4000
        // HPA = 0x10000 + 0x4000 -> l0_pte: 0b0101 00|00 0000 0001 = 20481
        // HPA = 0x14000 + 0x1000 -> l1_pte: 0b0101 01|00 0000 0001 = 21505
        // HPA = 0x15000 + 0x1000 -> l2_pte: 0b0101 10|00 0000 0001 = 22529
        // HPA = 0x2000 -> l3_pte: 0b0000 10|00 0000 1111 = 2063
        let pte_index_ans = 
            vec![(0, l0_pte), (512*4, l1_pte), (512*5, l2_pte), (512*6+1, l3_pte)];

        // 4 PTEs should be set
        for (i, j) in &pte_index_ans {
            unsafe {
                ptr = root_ptr.add(*i);
                pte = *ptr;
            }

            assert_eq!(pte, *j as u64);
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

    // Check the value of the L4 PTE created by map_range
    #[test]
    fn test_map_range_pte() {
        let mut gsmmu = GStageMmu::new();

        // Create a page table
        gsmmu.map_range(0x1000, 0x2000, 0x2000, PTE_READ | PTE_EXECUTE);

        // Check the pte
        let root_ptr = gsmmu.page_table.region.hpm_vptr;
        let mut ptr: *mut u64;
        let mut pte: u64;
        unsafe {
            ptr = root_ptr.add(512*6+1);
        }
        pte = unsafe { *ptr };

        // PTE on L4 should be 0b1000 0000 1011
        // ppn = 0b10 with PTE_EXECUTE/READ/VALID
        assert_eq!(pte, 2059);

        unsafe {
            ptr = root_ptr.add(512*6+2);
        }
        pte = unsafe { *ptr };

        // PTE on L4 should be 0b1100 0000 1011
        // ppn = 0b10 with PTE_EXECUTE/READ/VALID
        assert_eq!(pte, 3083);
    }

    // Check the location(index) of the new PTEs created by map_range
    #[test]
    fn test_map_range_index() {
        let mut gsmmu = GStageMmu::new();
        let root_ptr = gsmmu.page_table.region.hpm_vptr;
        let mut ptr: *mut u64;
        let mut pte: u64;

        gsmmu.map_range(0x1000, 0x2000, 0x2000, PTE_READ | PTE_EXECUTE);

        // Non-zero [0, 512*4, 512*5, 512*6+1, 512*6+2]
        let pte_index = vec![0, 512*4, 512*5, 512*6+1, 512*6+2];

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

    #[test]
    fn test_unmap_page() {
        let mut gsmmu = GStageMmu::new();

        // Create a page table
        gsmmu.map_range(0x1000, 0x2000, 0x2000, PTE_READ | PTE_EXECUTE);

        // Check the pte
        let root_ptr = gsmmu.page_table.region.hpm_vptr;
        let ptr: *mut u64;
        unsafe {
            ptr = root_ptr.add(512*6+1);
        }
        let mut pte: u64 = unsafe { *ptr };

        // PTE on L4 should be 0b1000 0000 1011
        // ppn = 0b10 with PTE_EXECUTE/READ/VALID
        assert_eq!(pte, 2059);

        // Unmap the page
        gsmmu.unmap_page(0x1000);

        // Get the pte again after unmap
        pte = unsafe { *ptr };

        // Should be cleared
        assert_eq!(pte, 0);
    }

    #[test]
    fn test_cascaded_map_page_unmap_page() {
        let mut gsmmu = GStageMmu::new();
        let gpa : u64 = 0x1000;
        let hpa : u64 = 0x2000; 
        // Create a page table
        gsmmu.map_page(gpa, 0x2000, PTE_READ | PTE_EXECUTE);

        // construct expected pte value for each level
        let root_ptr = gsmmu.page_table.region.hpm_vptr as u64;
        let root_ptr_pa_wrap = gsmmu.page_table.region.va_to_hpa(root_ptr);
        assert!(!root_ptr_pa_wrap.is_none());
        let root_ptr_pa = root_ptr_pa_wrap.unwrap();

        let expected_ptes : [u64; 4] = [
            ((root_ptr_pa + 4*PAGE_SIZE) >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID, 
            ((root_ptr_pa + 5*PAGE_SIZE) >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID, 
            ((root_ptr_pa + 6*PAGE_SIZE) >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID, 
            ((hpa >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID | PTE_READ | PTE_EXECUTE)
        ];

        let offsets_wrap = gsmmu.gpa_to_ptregion_offset(gpa);
        assert!(!offsets_wrap.is_none());
        let offsets = offsets_wrap.unwrap();

        for level in 0..4 {
            let offset = offsets[level as usize];
            let pte_addr = root_ptr + offset;
            let pte_addr_ptr = pte_addr as *mut u64;
            let pte_val = unsafe { *pte_addr_ptr };
            assert_eq!(pte_val, expected_ptes[level]);
        }

        gsmmu.unmap_page(0x1000);

        for level in 0..4 {
            let offset = offsets[level as usize];
            let pte_addr = root_ptr + offset;
            let pte_addr_ptr = pte_addr as *mut u64;
            let pte_val = unsafe { *pte_addr_ptr };
            assert_eq!(pte_val, 0);
        }
    }

    #[test]
    fn test_cascaded_map_range_unmap_range() {
        let mut gsmmu = GStageMmu::new();
        let gpa : u64 = 0x1000;
        let hpa : u64 = 0x2000;
        // Create a page table
        gsmmu.map_range(gpa, hpa, 2 * PAGE_SIZE, PTE_READ | PTE_EXECUTE);

        // construct expected pte value for each level
        let root_ptr = gsmmu.page_table.region.hpm_vptr as u64;
        let root_ptr_pa_wrap = gsmmu.page_table.region.va_to_hpa(root_ptr);
        assert!(!root_ptr_pa_wrap.is_none());
        let root_ptr_pa = root_ptr_pa_wrap.unwrap();

        let expected_ptes : [u64; 4] = [
            ((root_ptr_pa + 4*PAGE_SIZE) >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID, 
            ((root_ptr_pa + 5*PAGE_SIZE) >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID, 
            ((root_ptr_pa + 6*PAGE_SIZE) >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID, 
            ((hpa >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID | PTE_READ | PTE_EXECUTE)
        ];

        let offsets_wrap = gsmmu.gpa_to_ptregion_offset(gpa);
        assert!(!offsets_wrap.is_none());
        let offsets = offsets_wrap.unwrap();

        for level in 0..4 {
            let offset = offsets[level as usize];
            let pte_addr = root_ptr + offset;
            let pte_addr_ptr = pte_addr as *mut u64;
            let pte_val = unsafe { *pte_addr_ptr };
            assert_eq!(pte_val, expected_ptes[level]);
        }

        gsmmu.unmap_range(gpa, 2 * PAGE_SIZE);

        for level in 0..4 {
            let offset = offsets[level as usize];
            let pte_addr = root_ptr + offset;
            let pte_addr_ptr = pte_addr as *mut u64;
            let pte_val = unsafe { *pte_addr_ptr };
            assert_eq!(pte_val, 0);
        }
    }

    #[test]
    fn test_cascaded_map_range_unmap_page() {
        let mut gsmmu = GStageMmu::new();
        let gpa : u64 = 0x1000;
        let hpa : u64 = 0x2000;
        // Create a page table
        gsmmu.map_range(gpa, hpa, 2 * PAGE_SIZE, PTE_READ | PTE_EXECUTE);

        // construct expected pte value for each level
        let root_ptr = gsmmu.page_table.region.hpm_vptr as u64;
        let root_ptr_pa_wrap = gsmmu.page_table.region.va_to_hpa(root_ptr);
        assert!(!root_ptr_pa_wrap.is_none());
        let root_ptr_pa = root_ptr_pa_wrap.unwrap();

        let expected_ptes : [u64; 4] = [
            ((root_ptr_pa + 4*PAGE_SIZE) >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID, 
            ((root_ptr_pa + 5*PAGE_SIZE) >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID, 
            ((root_ptr_pa + 6*PAGE_SIZE) >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID,
            ((hpa >> PAGE_SHIFT << PTE_PPN_SHIFT) | PTE_VALID | PTE_READ | PTE_EXECUTE)
        ];

        let offsets_wrap = gsmmu.gpa_to_ptregion_offset(gpa);
        assert!(!offsets_wrap.is_none());
        let offsets = offsets_wrap.unwrap();

        for level in 0..4 {
            let offset = offsets[level as usize];
            let pte_addr = root_ptr + offset;
            let pte_addr_ptr = pte_addr as *mut u64;
            let pte_val = unsafe { *pte_addr_ptr };
            assert_eq!(pte_val, expected_ptes[level]);
        }

        gsmmu.unmap_page(gpa);

        for level in 0..4 {
            let offset = offsets[level as usize];
            let pte_addr = root_ptr + offset;
            let pte_addr_ptr = pte_addr as *mut u64;
            let pte_val = unsafe { *pte_addr_ptr };
            if level != 3 {
                assert_eq!(pte_val, expected_ptes[level]);
            } else {
                assert_eq!(pte_val, 0);
            }
        }

        gsmmu.unmap_page(gpa + PAGE_SIZE);

        for level in 0..4 {
            let offset = offsets[level as usize];
            let pte_addr = root_ptr + offset;
            let pte_addr_ptr = pte_addr as *mut u64;
            let pte_val = unsafe { *pte_addr_ptr };
            assert_eq!(pte_val, 0);
        }
    }


    #[test]
    fn test_unmap_range() {
        let mut gsmmu = GStageMmu::new();

        // Create a page table
        gsmmu.map_range(0x1000, 0x2000, 0x2000, PTE_READ | PTE_EXECUTE);

        // Check the pte
        let root_ptr = gsmmu.page_table.region.hpm_vptr;
        let mut ptr: *mut u64;
        let mut pte: u64;
        unsafe {
            ptr = root_ptr.add(512*6+1);
        }
        pte = unsafe { *ptr };

        // PTE on L4 should be 0b1000 0000 1011
        // ppn = 0b10 with PTE_EXECUTE/READ/VALID
        assert_eq!(pte, 2059);

        unsafe {
            ptr = root_ptr.add(512*6+2);
        }
        pte = unsafe { *ptr };

        // PTE on L4 should be 0b1100 0000 1011
        // ppn = 0b10 with PTE_EXECUTE/READ/VALID
        assert_eq!(pte, 3083);

        // Unmap the range
        gsmmu.unmap_range(0x1000, 0x2000);

        // Check the 2 PTEs again after unmap_range
        unsafe {
            ptr = root_ptr.add(512*6+1);
        }
        pte = unsafe { *ptr };
        assert_eq!(pte, 0);

        unsafe {
            ptr = root_ptr.add(512*6+2);
        }
        pte = unsafe { *ptr };
        assert_eq!(pte, 0);
    }

    #[test]
    fn test_map_query() {
        let mut gsmmu = GStageMmu::new();
        let mut pte: Pte;
        let mut pte_offset: u64 = 0;
        let mut pte_value: u64 = 0;
        let mut pte_level: u32 = 0;

        // None
        let mut query = gsmmu.map_query(0x1000);

        if query.is_some() {
            pte = query.unwrap();
            pte_offset = pte.offset;
            pte_value = pte.value;
            pte_level = pte.level;
        }

        assert_eq!(pte_offset, 0);
        assert_eq!(pte_value, 0);
        assert_eq!(pte_level, 0);

        // Some(Pte) 
        gsmmu.map_page(0x1000, 0x2000, PTE_READ | PTE_EXECUTE);
        query = gsmmu.map_query(0x1000);

        if query.is_some() {
            pte = query.unwrap();
            pte_offset = pte.offset;
            pte_value = pte.value;
            pte_level = pte.level;
        }

        assert_eq!(pte_offset, (512 * 6 + 1) * 8);
        assert_eq!(pte_value, 2059);
        assert_eq!(pte_level, 3);
    }

    // Check map_page by invalid hpa
    #[test]
    fn test_map_page_invalid_hpa() {
        let mut gsmmu = GStageMmu::new();
        let valid_gpa = 0x1000;
        let invalid_hpa = 0x2100;

        // Create a page table
        let result = gsmmu.map_page(valid_gpa, invalid_hpa, PTE_READ | PTE_EXECUTE);
        if result.is_some() {
            panic!("HPA: {:x} should be invalid", invalid_hpa);
        }
    }

    // Check map_page by invalid gpa
    #[test]
    fn test_map_page_invalid_gpa() {
        let mut gsmmu = GStageMmu::new();
        let valid_hpa = 0x2000;
        let invalid_gpa = 0x1100;

        // Create a page table
        let result = gsmmu.map_page(invalid_gpa, valid_hpa, PTE_READ | PTE_EXECUTE);
        if result.is_some() {
            panic!("GPA: {:x} should be invalid", invalid_gpa);
        }
    }

    #[test]
    fn test_map_protect() {
        let mut gsmmu = GStageMmu::new();
        let root_ptr = gsmmu.page_table.region.hpm_vptr;
        let ptr: *mut u64;
        let mut pte: u64;

        // pte = 2063
        gsmmu.map_page(0x1000, 0x2000, PTE_READ | PTE_WRITE | PTE_EXECUTE); 

        unsafe {
            ptr = root_ptr.add(512*6+1);
        }
        pte = unsafe { *ptr };

        assert_eq!(pte, 2063);

        // pte = 2059
        gsmmu.map_protect(0x1000, PTE_READ | PTE_EXECUTE);

        pte = unsafe { *ptr };

        assert_eq!(pte, 2059);
    }

}