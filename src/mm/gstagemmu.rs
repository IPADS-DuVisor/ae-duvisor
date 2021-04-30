use crate::mm::hpmallocator;
use crate::mm::gparegion;
use crate::mm::mmio;
use core::mem;
use crate::mm::utils::*;

pub mod gsmmu_constants {
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
    // Access the pte by (page_table.region.hpm_vptr + offset) as u64
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

    pub fn is_leaf(&self) -> bool {
        if self.value & 0x1 == 0 || self.value & 0xE == 0 {
            return false;
        }

        true
    }
}

pub struct PageTableRegion {
    pub vaddr: u64,
    pub paddr: u64,
    pub length: u64,
    pub free_offset: u64,
}

impl PageTableRegion {
    pub fn new(allocator: &mut hpmallocator::HpmAllocator) -> Self {
        let region_wrap = allocator.hpm_alloc(PAGE_TABLE_REGION_SIZE);
        
        if region_wrap.is_none() {
            panic!("PageTableRegion::new : hpm_alloc failed");
        }

        let region = region_wrap.unwrap();

        if region.len() != 1 {
            panic!("PageTableRegion::new : Page table alloc failed for length {}", region.len());
        }

        let mut vaddr = 0;
        let mut paddr = 0;
        let mut length = 0;

        for i in &region {
            vaddr = i.hpm_vptr as u64;
            paddr = i.base_address as u64;
            length = i.length as u64;
        }

        Self {
            vaddr,
            paddr,
            length,
            free_offset: 0,
        }
    }

    pub fn va_to_hpa(&self, va: u64) -> Option<u64> {
        let va_base = self.vaddr;
        let hpa_base = self.paddr;

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
        let va_base = self.vaddr;
        let hpa_base = self.paddr;

        if hpa < hpa_base {
            return None;
        }

        let offset = hpa - hpa_base;

        if offset >= self.length {
            return None;
        }

        Some(offset + va_base)
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
        if total_length > self.length {
            panic!("PageTableRegion::page_table_alloc : length {} out of bound", length);
        }

        let offset = self.free_offset;
        let ret_ptr = (self.vaddr + offset) as *mut u64;

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
    pub gpa_blocks: Vec<gparegion::GpaBlock>, // gpa block list
    pub allocator: hpmallocator::HpmAllocator,
    pub mmio_manager : mmio::MmioManager,
}

impl GStageMmu {
    pub fn new(ioctl_fd: i32) -> Self {
        let gpa_blocks: Vec<gparegion::GpaBlock> = Vec::new();
        let mut allocator = hpmallocator::HpmAllocator::new(ioctl_fd);
        let mut page_table = PageTableRegion::new(&mut allocator);
        let mmio_manager = mmio::MmioManager::new();

        // create root table
        page_table.page_table_create(0);

        Self {
            page_table,
            gpa_blocks,
            allocator,
            mmio_manager,
        }
    }

    // TODO: add mem_size in gsmmu and check gpa
    pub fn check_gpa(&mut self, _gpa: u64) -> bool {
        return true;
    }

    pub fn gpa_block_query(&mut self, gpa: u64) -> Option<u64> {
        let mut start: u64;
        let mut end: u64;
        let hpa: u64;

        println!("gpa_block_query gpa: {:x}", gpa);

        for i in &self.gpa_blocks {
            start = i.gpa;
            end = start + i.length;
            println!("gpa_block_query gpa: {:x}, hpa: {:x}, length: {:x}",
                i.gpa, i.hpa, i.length);
            if gpa >= start &&  gpa < end {
                println!("find a gpa block: gpa: {:x}, hpa: {:x}, length: {:x}",
                    i.gpa, i.hpa, i.length);
                hpa = i.hpa + gpa - start;
                println!("gpa_block_query hpa: {:x}", hpa);
                return Some(hpa);
            }
        }

        return None;
    }

    // For debug
    pub fn gsmmu_test(&mut self)  {
        self.map_page(0x1000, 0x2000, PTE_READ | PTE_EXECUTE);
        self.unmap_page(0x1000);
        self.map_range(0x1000, 0x2000, 0x2000, 
            PTE_READ | PTE_WRITE | PTE_EXECUTE);
        self.unmap_range(0x1000, 0x2000);
        self.map_query(0x1000);
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
        let mut page_table_va = self.page_table.vaddr as u64;
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
                page_table_va = 
                    self.page_table.page_table_create(level + 1) as u64;
                page_table_hpa_wrap = self.page_table.va_to_hpa(page_table_va);
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
                page_table_va_wrap = self.page_table.hpa_to_va(page_table_hpa);
                if page_table_va_wrap.is_none() {
                    return None;
                }
                page_table_va = page_table_va_wrap.unwrap();
            }
            offsets[level as usize] = pte_addr_va 
                - (self.page_table.vaddr as u64);
        }

        index = ((gpa >> 12) & 0x1ff) * 8;
        offsets[3] = page_table_va + index - (self.page_table.vaddr as u64);
        Some(offsets)
    }

    pub fn map_query(&mut self, gpa: u64) -> Option<Pte> {
        let mut page_table_va = self.page_table.vaddr;
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
                pte_offset = pte_addr_va - (self.page_table.vaddr as u64);
                pte_value = pte;
                pte_level = level as u32;

                if level == 3 {
                    break;
                }

                page_table_hpa = (pte >> PTE_PPN_SHIFT) << PAGE_SHIFT;
                page_table_va_wrap = self.page_table.hpa_to_va(page_table_hpa);
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

        let page_table_va = self.page_table.vaddr as u64;
        let pte_addr = (page_table_va + pte_offset) as *mut u64;
        pte_value = GStageMmu::set_pte_flag(pte_value, pte_level as u64, flag);
        unsafe {
            *pte_addr = pte_value;
        }

        Some(0)
    }

    // SV48x4
    pub fn map_page(&mut self, gpa: u64, hpa: u64, flag: u64) -> Option<u32> {
        println!("enter map_page - gpa: {:x}, hpa: {:x}, flag: {:x}", 
            gpa, hpa, flag);
        let offsets_wrap = self.gpa_to_ptregion_offset(gpa);
        if offsets_wrap.is_none() {
            return None;
        }
        let offset = offsets_wrap.unwrap()[3];
        let page_table_va = self.page_table.vaddr as u64;
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

    pub fn map_range(&mut self, gpa: u64, hpa: u64, length: u64, flag: u64)
        -> Option<u32> {
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

    fn is_empty_ptp(pte_addr :u64) -> bool {
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
            let page_table_va = self.page_table.vaddr as u64;
            let pte_addr = page_table_va + offset;
    
            let pte_addr_ptr = pte_addr as *mut u64;
            unsafe {
                *pte_addr_ptr = 0;
            }
            if !GStageMmu::is_empty_ptp(pte_addr) {
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

    pub fn gpa_block_add(&mut self, gpa: u64, mut length: u64)
        -> Result<(u64, u64), u64> {
        // gpa block should always be aligned to PAGE_SIZE
        length = page_size_round_up(length);

        let region_wrap = self.allocator.hpm_alloc(length);

        if region_wrap.is_none() {
            println!("gpa_block_add : hpm_alloc failed");
            return Err(0);
        }

        let region = region_wrap.unwrap();

        if region.len() != 1 {
            println!("gpa_block_add : gpa block alloc failed for length {}",
                region.len());
            return Err(0);
        }

        let mut hpa = 0;
        let mut hva = 0;

        for i in &region {
            hpa = i.base_address;
            hva = i.hpm_vptr;
        }

        let gpa_block = gparegion::GpaBlock::new(gpa, hpa, length);
        self.gpa_blocks.push(gpa_block);

        return Ok((hva, hpa));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use rusty_fork::rusty_fork_test;

    rusty_fork_test! { 
        // Check new() of GStageMmu
        #[test]
        fn test_gsmmu_new() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let gsmmu = GStageMmu::new(ioctl_fd);

            // Check the root table has been created
            let free_offset = gsmmu.page_table.free_offset;
            assert_eq!(free_offset, 16384);

            // Check the root table has been cleared
            let mut root_ptr = gsmmu.page_table.vaddr as *mut u64;
            unsafe {
                root_ptr = root_ptr.add(10);
            }
            let pte: u64 = unsafe { *root_ptr };
            assert_eq!(pte, 0);
        }

        // Check gpa_block_add
        #[test]
        fn test_gpa_block_add() { 
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);
            let mut gpa: u64 = 0;
            let mut length: u64 = 0;

            let result = gsmmu.gpa_block_add(0x1000, 0x4000);
            assert!(result.is_ok());

            let list_length = gsmmu.gpa_blocks.len();
            assert_eq!(1, list_length);

            for i in gsmmu.gpa_blocks {
                gpa = i.gpa;
                length = i.length;
            }

            assert_eq!(gpa, 0x1000);
            assert_eq!(length, 0x4000);
        }

        #[test]
        fn test_page_table_create() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);

            // Create a page table
            gsmmu.page_table.page_table_create(1);

            // Check the page table has been created
            let free_offset = gsmmu.page_table.free_offset;
            assert_eq!(free_offset, 16384 + 4096);

            // Check the page table has been cleared
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
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
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);

            // Create a page table
            gsmmu.map_page(0x1000, 0x2000, PTE_READ | PTE_EXECUTE);

            // Check the pte
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
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
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
            let mut ptr: *mut u64;
            let mut pte: u64;
            let gpa = 0x1000;
            let hpa = 0x2000;

            // Change 4 PTEs
            gsmmu.map_page(gpa, hpa, PTE_READ | PTE_WRITE | PTE_EXECUTE); 

            // Non-zero [0, 512*4, 512*5, 512*6+1]
            let pte_index = vec![0, 512*4, 512*5, 512*6+1];

            // non-zero answer
            let base_address = gsmmu.page_table.paddr;
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
                vec![(0, l0_pte), (512*4, l1_pte), (512*5, l2_pte), 
                    (512*6+1, l3_pte)];

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
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);

            // Create a page table
            gsmmu.map_range(0x1000, 0x2000, 0x2000, PTE_READ | PTE_EXECUTE);

            // Check the pte
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
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
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
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
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);

            // Create a page table
            gsmmu.map_range(0x1000, 0x2000, 0x2000, PTE_READ | PTE_EXECUTE);

            // Check the pte
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
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
        fn test_unmap_range() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);

            // Create a page table
            gsmmu.map_range(0x1000, 0x2000, 0x2000, PTE_READ | PTE_EXECUTE);

            // Check the pte
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
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
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);
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
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);
            let valid_gpa = 0x1000;
            let invalid_hpa = 0x2100;

            // Create a page table
            let result =
                gsmmu.map_page(valid_gpa, invalid_hpa, PTE_READ | PTE_EXECUTE);
            if result.is_some() {
                panic!("HPA: {:x} should be invalid", invalid_hpa);
            }
        }

        // Check map_page by invalid gpa
        #[test]
        fn test_map_page_invalid_gpa() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);
            let valid_hpa = 0x2000;
            let invalid_gpa = 0x1100;

            // Create a page table
            let result =
                gsmmu.map_page(invalid_gpa, valid_hpa, PTE_READ | PTE_EXECUTE);
            if result.is_some() {
                panic!("GPA: {:x} should be invalid", invalid_gpa);
            }
        }


        #[test]
        fn test_map_protect() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mut gsmmu = GStageMmu::new(ioctl_fd);
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
            let ptr: *mut u64;
            let mut pte: u64;

            // pte = 2063
            gsmmu.map_page(0x1000, 0x2000, PTE_READ | PTE_WRITE 
                | PTE_EXECUTE); 

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
}
