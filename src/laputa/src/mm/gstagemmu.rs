use crate::mm::hpmallocator;
use crate::mm::gparegion;
use crate::mm::mmio;
use core::mem;
use crate::mm::utils::*;

extern crate sys_util;
use sys_util::GuestMemory;

#[allow(unused)]
extern "C"
{
    fn hufence_gvma_all();
}

/* Choose SV39x4 or SV48x4 */
pub const S2PT_MODE: i32 = S2PT_SV39_LEVEL_COUNT;

/* SV39x4 with 3-level s2pt */
const S2PT_SV39_LEVEL_COUNT: i32 = 3;

/* SV48x4 with 4-level s2pt */
#[allow(unused)]
const S2PT_SV48_LEVEL_COUNT: i32 = 4;

pub mod gsmmu_constants {
    /* Pte bit */
    pub const PTE_VALID: u64 = 1u64 << 0;
    pub const PTE_READ: u64 = 1u64 << 1;
    pub const PTE_WRITE: u64 = 1u64 << 2;
    pub const PTE_EXECUTE: u64 = 1u64 << 3;
    pub const PTE_USER: u64 = 1u64 << 4;
    pub const PTE_GLOBAL: u64 = 1u64 << 5;
    pub const PTE_ACCESS: u64 = 1u64 << 6;
    pub const PTE_DIRTY: u64 = 1u64 << 7;

    pub const PTE_VRWEU: u64 
        = PTE_VALID | PTE_READ | PTE_WRITE | PTE_EXECUTE | PTE_USER;

    pub const PTE_PPN_SHIFT: u64 = 10;
}
pub use gsmmu_constants::*;

pub struct Pte {
    /* 
     * The offset of this pte from the top of the root table
     * (page_table.region.hpm_vptr). Access the pte by 
     * (page_table.region.hpm_vptr + offset) as u64.
     */
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
        let region_wrap = allocator.hpm_alloc_s2pt(0, PAGE_TABLE_REGION_SIZE);
        
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

        /* Root page table takes 4 pages in SV39x4 & SV48x4 */
        if level == 0 {
            size = PAGE_SIZE * 4;
        }

        let ptr = self.page_table_alloc(size);
        ptr
    }

    /* Alloc page table from &self.region: HpmRegion */
    pub fn page_table_alloc(&mut self, length: u64) -> *mut u64 {
        let u64_size: usize = mem::size_of::<u64>();
        assert_eq!(length % u64_size as u64, 0);

        let total_length: u64 = self.free_offset + length;
        if total_length > self.length {
            panic!("PageTableRegion::page_table_alloc : length {} out of bound", length);
        }

        let offset = self.free_offset;
        let ret_ptr = (self.vaddr + offset) as *mut u64;

        /* Clear the new page table */
        let ptr = ret_ptr as *mut libc::c_void;
        unsafe { libc::memset(ptr, 0, length as usize); }

        /* Offset update */
        self.free_offset += length;

        ret_ptr
    }
}

#[allow(unused)]
pub struct GStageMmu {
    pub page_table: PageTableRegion,
    pub guest_mem: GuestMemory,
    pub allocator: hpmallocator::HpmAllocator,
    pub mmio_manager: mmio::MmioManager,
    pub mem_gpa_regions: Vec<gparegion::GpaRegion>,
}

impl GStageMmu {
    pub fn new(ioctl_fd: i32, mem_size: u64, guest_mem: GuestMemory,
        mmio_regions: Vec<gparegion::GpaRegion>) -> Self {    
        let mut allocator = hpmallocator::HpmAllocator::new(ioctl_fd);
        let mut page_table = PageTableRegion::new(&mut allocator);
        let mmio_manager = mmio::MmioManager::new(mmio_regions);

        let mem_gpa_regions = GStageMmu::init_gpa_regions(mem_size,
                &mmio_manager);

        /* Create root table */
        page_table.page_table_create(0);

        Self {
            page_table,
            guest_mem,
            allocator,
            mmio_manager,
            mem_gpa_regions,
        }
    }

    pub fn gpa_block_overlap(&mut self, gpa: u64, length: u64) -> bool {
        for offset in (0..length as usize).step_by(PAGE_SIZE as usize) {
            match self.guest_mem.query_region(gpa + offset as u64) {
                Some(_mmap) => { return true; }
                None => { continue; }
            }
        }
        
        return false;
    }

    pub fn init_gpa_regions(mem_size: u64, mmio_manager: &mmio::MmioManager)
        -> Vec<gparegion::GpaRegion> {
        let mut gpa_regions: Vec<gparegion::GpaRegion> = Vec::new();
        let mut gpa_region: gparegion::GpaRegion;
        let mut gpa_region_gpa: u64 = 0;
        let mut gpa_region_length: u64;

        /* Check whether gpa regions are overlapped and reorder them */
        if !mmio_manager.check_valid() {
            panic!("Invalid mmio config!");
        }

        for i in &mmio_manager.gpa_regions {
            if gpa_region_gpa < i.gpa {
                gpa_region_length = i.gpa - gpa_region_gpa;
                gpa_region = gparegion::GpaRegion::new(gpa_region_gpa,
                    gpa_region_length);
                gpa_regions.push(gpa_region);
            }

            gpa_region_gpa = i.gpa + i.length;
        }

        // TODO: memory does NOT start from 0!
        //if gpa_region_gpa < mem_size {
        //    gpa_region_length = mem_size - gpa_region_gpa;
        //    gpa_region = gparegion::GpaRegion::new(gpa_region_gpa,
        //        gpa_region_length);
        //    gpa_regions.push(gpa_region);
        //}
        gpa_region_length = mem_size;
        gpa_region = gparegion::GpaRegion::new(gpa_region_gpa,
            gpa_region_length);
        gpa_regions.push(gpa_region);

        gpa_regions
    }

    pub fn check_gpa(&mut self, gpa: u64) -> bool {
        for i in &self.mem_gpa_regions {
            let gpa_start = i.gpa;
            let gpa_end = gpa_start + i.length;

            dbgprintln!("check_gpa() - gpa {:x}, gpa_start {:x}, gpa_end {:x}",
                gpa, gpa_start, gpa_end);

            if gpa >= gpa_start && gpa < gpa_end {
                return true;
            }
        }

        return false;
    }

    pub fn check_mmio(&mut self, gpa: u64) -> bool {
        for i in &self.mmio_manager.gpa_regions {
            let gpa_start = i.gpa;
            let gpa_end = gpa_start + i.length;

            dbgprintln!("check_mmio(): gpa {:x}, gpa_start {:x}, gpa_end {:x}",
                gpa, gpa_start, gpa_end);

            if gpa >= gpa_start && gpa < gpa_end {
                return true;
            }
        }

        return false;
    }

    pub fn gpa_block_query(&mut self, gpa: u64) -> Option<(u64, u64)> {
        let hva: u64;
        let hpa: u64;

        dbgprintln!("gpa_block_query gpa: {:x}", gpa);

        let gpa_key = gpa & !PAGE_SIZE_MASK;

        match self.guest_mem.query_region(gpa_key) {
            Some(res) => { return Some((res.0, res.1)); },
            None => { return None; }
        }
    }

    /* For debug */
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
        let pt_level: u64 = (S2PT_MODE - 1) as u64;

        if level == pt_level {
            pte = pte | flag;
        }

        pte = pte | PTE_VALID;

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

        for level in 0..(S2PT_MODE - 1) {
            shift = (9 * S2PT_MODE + 3) as u64 - PAGE_ORDER * (level as u64);
            index = (gpa >> shift) & 0x1ff;
            if level == 0 {
                index = (gpa >> shift) & 0x7ff;
            }
            let pte_addr_va = page_table_va + index * 8;
            let mut pte = unsafe { *(pte_addr_va as *mut u64) };

            if pte & PTE_VALID == 0 {
                page_table_va = 
                    self.page_table.page_table_create(level as u64 + 1) as u64;
                page_table_hpa_wrap = self.page_table.va_to_hpa(page_table_va);
                if page_table_hpa_wrap.is_none() {
                    return None;
                }
                page_table_hpa = page_table_hpa_wrap.unwrap();
                pte = (page_table_hpa >> PAGE_SHIFT) << PTE_PPN_SHIFT;
                pte = GStageMmu::set_pte_flag(pte, level as u64, 0);
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
        offsets[S2PT_MODE as usize - 1] 
            = page_table_va + index - (self.page_table.vaddr as u64);
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

        for level in 0..S2PT_MODE {
            shift = (9 * S2PT_MODE + 3) as u64 - PAGE_ORDER * (level as u64);
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

                if level == (S2PT_MODE - 1) {
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

        if pte_level != (S2PT_MODE as u32 - 1)  {
            /* No mapping, and nothing changed */
            return None;
        }

        let page_table_va = self.page_table.vaddr as u64;
        let pte_addr = (page_table_va + pte_offset) as *mut u64;
        pte_value = GStageMmu::set_pte_flag(pte_value, pte_level as u64, flag);
        unsafe {
            *pte_addr = pte_value;
        }
        unsafe { 
            hufence_gvma_all();
        }

        Some(0)
    }

    pub fn map_page(&mut self, gpa: u64, hpa: u64, flag: u64) -> Option<u32> {
        let ret = self.__map_page(gpa, hpa, flag);
        unsafe { 
            hufence_gvma_all();
        }
        return ret;
    }
    
    pub fn __map_page(&mut self, gpa: u64, hpa: u64, flag: u64) -> Option<u32> {
        dbgprintln!("enter map_page - gpa: {:x}, hpa: {:x}, flag: {:x}", 
            gpa, hpa, flag);
        let offsets_wrap = self.gpa_to_ptregion_offset(gpa);
        if offsets_wrap.is_none() {
            return None;
        }
        let offset = offsets_wrap.unwrap()[(S2PT_MODE - 1) as usize];
        let page_table_va = self.page_table.vaddr as u64;
        let pte_addr = page_table_va + offset;

        if (hpa & 0xfff) != 0 {
            return None;
        }

        if (gpa & 0xfff) != 0 {
            return None;
        }

        let mut pte = hpa >> (PAGE_SHIFT - PTE_PPN_SHIFT);
        pte = GStageMmu::set_pte_flag(pte, (S2PT_MODE - 1) as u64, flag);

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
            self.__map_page(gpa + offset, hpa + offset, flag);
            offset += PAGE_SIZE;
            if offset >= length {
                break;
            }
        }
        unsafe { 
            hufence_gvma_all();
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

    /* Unmap L0/L1/L2 page table pages if there are no valid PTEs in them */
    fn __unmap_page(&mut self, gpa: u64) -> Option<u32> {
        if (gpa & 0xfff) != 0 {
            return None;
        }

        let offsets_wrap = self.gpa_to_ptregion_offset(gpa);
        if offsets_wrap.is_none() {
            return None;
        }
        let offsets = offsets_wrap.unwrap();
        for level in 0..S2PT_MODE {
            let offset = offsets[(S2PT_MODE - 1 - level) as usize];
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

    pub fn unmap_page(&mut self, gpa: u64) -> Option<u32> {
        let ret = self.__unmap_page(gpa);
        unsafe { 
            hufence_gvma_all();
        }
        ret
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
            self.__unmap_page(gpa + offset);
            offset += PAGE_SIZE;
            if offset >= length {
                break;
            }
        }
        unsafe { 
            hufence_gvma_all();
        }
        Some(0)
    }

    /* FIXME: gpa_start is hard-coded to 0x80000000 */
    pub fn gpa_block_add(&mut self, gpa: u64, mut length: u64)
        -> Result<(u64, u64), u64> {
        assert_eq!(gpa & 0xfff, 0);
        /* Gpa block should always be aligned to PAGE_SIZE */
        length = page_size_round_up(length);

        let gpa_start = 0x80000000;
        let region_wrap;
        if gpa == 0xf0000000 {
            region_wrap = self.allocator.hpm_alloc(length);
        } else {
            region_wrap = self.allocator.hpm_alloc_vm_mem(gpa - gpa_start, length);
        }

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
        if self.guest_mem.end_addr().offset() == 0 {
            let offset = gpa - gpa_start;
            self.guest_mem.lazy_init(hva - offset, gpa_start, hpa - offset, 2 << 30);
        }

        return Ok((hva, hpa));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use rusty_fork::rusty_fork_test;

    rusty_fork_test! { 
        /* Check new() of GStageMmu */
        #[test]
        fn test_gsmmu_new() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);

            /* Check the root table has been created */
            let free_offset = gsmmu.page_table.free_offset;
            assert_eq!(free_offset, 16384);

            /* Check the root table has been cleared */
            let mut root_ptr = gsmmu.page_table.vaddr as *mut u64;
            unsafe {
                root_ptr = root_ptr.add(10);
            }
            let pte: u64 = unsafe { *root_ptr };
            assert_eq!(pte, 0);
        }

        /* Check gpa_block_add */
        #[test]
        fn test_gpa_block_add() { 
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);

            let result = gsmmu.gpa_block_add(0x1000, 0x4000);
            assert!(result.is_ok());
        }

        #[test]
        fn test_page_table_create() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }
            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);

            /* Create a page table */
            gsmmu.page_table.page_table_create(1);

            /* Check the page table has been created */
            let free_offset = gsmmu.page_table.free_offset;
            assert_eq!(free_offset, 16384 + 4096);

            /* Check the page table has been cleared */
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
            let ptr: *mut u64;
            unsafe {
                ptr = root_ptr.add(512+10);
            }
            let pte: u64 = unsafe { *ptr };
            assert_eq!(pte, 0);
        }

        /* Check the value of the leaf PTE */
        #[test]
        fn test_map_page_pte() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);

            /* Create a page table */
            gsmmu.map_page(0x1000, 0x2000, PTE_READ | PTE_EXECUTE);

            /* Check the pte */
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
            let ptr: *mut u64;
            unsafe {
                let offset = 512 * (2 + S2PT_MODE) + 1;
                ptr = root_ptr.add(offset as usize);
            }
            let pte: u64 = unsafe { *ptr };

            /* The leaf PTE should be 0b1000 0000 1011 */
            /* PPN = 0b10 with PTE_EXECUTE/READ/VALID */
            assert_eq!(pte, 2059);
        }

        /* Check the location(index) of the new PTEs */
        #[test]
        fn test_map_page_index() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
            let mut ptr: *mut u64;
            let mut pte: u64;
            let gpa = 0x1000;
            let hpa = 0x2000;

            /* Change S2PT_MODE PTEs */
            gsmmu.map_page(gpa, hpa, PTE_READ | PTE_WRITE | PTE_EXECUTE); 

            /* Non-zero [0, 512*4, 512*5, 512*6+1] */
            let mut pte_index = Vec::new();
            pte_index.push(0);
            pte_index.push(512*4);

            /* Non-zero answer */
            let base_address = gsmmu.page_table.paddr;
            let l0_pte = ((base_address + 0x4000) >> 2) | PTE_VALID;
            let l1_pte = ((base_address + 0x5000) >> 2) | PTE_VALID;
            let l2_pte = ((base_address + 0x6000) >> 2) | PTE_VALID;
            let leaf_pte = (hpa >> 2)
                | PTE_VALID | PTE_READ | PTE_WRITE | PTE_EXECUTE;

            /*
             * Start from 0x10000 and the root table takes 0x4000
             * HPA = 0x10000 + 0x4000 -> l0_pte: 0b0101 00|00 0000 0001 = 20481
             * HPA = 0x14000 + 0x1000 -> l1_pte: 0b0101 01|00 0000 0001 = 21505
             * HPA = 0x15000 + 0x1000 -> l2_pte: 0b0101 10|00 0000 0001 = 22529
             * HPA = 0x2000 -> leaf_pte: 0b0000 10|00 0000 1111 = 2063
             */
            let mut pte_index_ans = Vec::new();
            pte_index_ans.push((0, l0_pte));
            pte_index_ans.push((512*4, l1_pte));

            match S2PT_MODE {
                3 => {
                    pte_index_ans.push((512*5+1, leaf_pte));
                    pte_index.push(512*5+1);
                }
                4 => {
                    pte_index_ans.push((512*5, l2_pte));
                    pte_index_ans.push((512*6+1, leaf_pte));
                    pte_index.push(512*5);
                    pte_index.push(512*6+1);
                }
                _ => {
                    panic!("Unsupported S2PT_MODE");
                }
            }

            /* Check the PTEs */
            for (i, j) in &pte_index_ans {
                unsafe {
                    ptr = root_ptr.add(*i);
                    pte = *ptr;
                }

                assert_eq!(pte, *j as u64);
            }

            /* All the other PTEs should be zero */
            let size: usize = 512 * (3 + S2PT_MODE) as usize;
            for i in (0..size).filter(|x: &usize| !pte_index.contains(x)) {
                unsafe {
                    ptr = root_ptr.add(i as usize);
                    pte = *ptr;
                }

                assert_eq!(pte, 0);
            }
        }
        
        /* Check the value of the L4 PTE created by map_range */
        #[test]
        fn test_map_range_pte() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);

            /* Create a page table */
            gsmmu.map_range(0x1000, 0x2000, 0x2000, PTE_READ | PTE_EXECUTE);

            /* Check the pte */
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
            let mut ptr: *mut u64;
            let mut pte: u64;
            unsafe {
                let offset = 512 * (2 + S2PT_MODE) + 1;
                ptr = root_ptr.add(offset as usize);
            }
            pte = unsafe { *ptr };

            /* The leaf PTE should be 0b1000 0000 1011 */
            /* PPN = 0b10 with PTE_EXECUTE/READ/VALID */
            assert_eq!(pte, 2059);

            unsafe {
                let offset = 512 * (2 + S2PT_MODE) + 2;
                ptr = root_ptr.add(offset as usize);
            }
            pte = unsafe { *ptr };

            /* The leaf PTE should be 0b1100 0000 1011 */
            /* PPN = 0b10 with PTE_EXECUTE/READ/VALID */
            assert_eq!(pte, 3083);
        }

        /* Check the location(index) of the new PTEs created by map_range */
        #[test]
        fn test_map_range_index() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
            let mut ptr: *mut u64;
            let mut pte: u64;

            gsmmu.map_range(0x1000, 0x2000, 0x2000, PTE_READ | PTE_EXECUTE);

            /* Non-zero [0, 512*4, 512*5, 512*6+1, 512*6+2] */
            let pte_index;

            match S2PT_MODE {
                3 => {
                    pte_index = vec![0, 512*4, 512*5+1, 512*5+2];
                }
                4 => {
                    pte_index = vec![0, 512*4, 512*5, 512*6+1, 512*6+2];
                }
                _ => {
                    panic!("Unsupported S2PT_MODE");
                }
            }

            for i in &pte_index {
                unsafe {
                    ptr = root_ptr.add(*i);
                    pte = *ptr;
                }

                assert_ne!(pte, 0);
            }

            /* All the other PTEs should be zero */
            let size = 512 * (3 + S2PT_MODE) as usize;
            for i in (0..size).filter(|x: &usize| !pte_index.contains(x)) {
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
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);

            /* Create a page table */
            gsmmu.map_range(0x1000, 0x2000, 0x2000, PTE_READ | PTE_EXECUTE);

            /* Check the pte */
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
            let ptr: *mut u64;
            unsafe {
                let offset = 512 * (S2PT_MODE + 2) + 1;
                ptr = root_ptr.add(offset as usize);
            }
            let mut pte: u64 = unsafe { *ptr };

            /* The leaf PTE should be 0b1000 0000 1011 */
            /* PPN = 0b10 with PTE_EXECUTE/READ/VALID */
            assert_eq!(pte, 2059);

            /* Unmap the page */
            gsmmu.unmap_page(0x1000);

            /* Get the pte again after unmap */
            pte = unsafe { *ptr };

            /* Should be cleared */
            assert_eq!(pte, 0);
        }

        #[test]
        fn test_unmap_range() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);

            /* Create a page table */
            gsmmu.map_range(0x1000, 0x2000, 0x2000, PTE_READ | PTE_EXECUTE);

            /* Check the pte */
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
            let mut ptr: *mut u64;
            let mut pte: u64;
            unsafe {
                let offset = 512 * (S2PT_MODE + 2) + 1;
                ptr = root_ptr.add(offset as usize);
            }
            pte = unsafe { *ptr };

            /* The leaf PTE should be 0b1000 0000 1011 */
            /* PPN = 0b10 with PTE_EXECUTE/READ/VALID */
            assert_eq!(pte, 2059);

            unsafe {
                let offset = 512 * (S2PT_MODE + 2) + 2;
                ptr = root_ptr.add(offset as usize);
            }
            pte = unsafe { *ptr };

            /* The leaf PTE should be 0b1100 0000 1011 */
            /* PPN = 0b10 with PTE_EXECUTE/READ/VALID */
            assert_eq!(pte, 3083);

            /* Unmap the range */
            gsmmu.unmap_range(0x1000, 0x2000);

            /* Check the 2 PTEs again after unmap_range */
            unsafe {
                let offset = 512 * (S2PT_MODE + 2) + 1;
                ptr = root_ptr.add(offset as usize);
            }
            pte = unsafe { *ptr };
            assert_eq!(pte, 0);

            unsafe {
                let offset = 512 * (S2PT_MODE + 2) + 2;
                ptr = root_ptr.add(offset as usize);
            }
            pte = unsafe { *ptr };
            assert_eq!(pte, 0);
        }

        #[test]
        fn test_map_query() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);
            let mut pte: Pte;
            let mut pte_offset: u64 = 0;
            let mut pte_value: u64 = 0;
            let mut pte_level: u32 = 0;

            /* None */
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

            /* Some(Pte)  */
            gsmmu.map_page(0x1000, 0x2000, PTE_READ | PTE_EXECUTE);
            query = gsmmu.map_query(0x1000);

            if query.is_some() {
                pte = query.unwrap();
                pte_offset = pte.offset;
                pte_value = pte.value;
                pte_level = pte.level;
            }

            let offset = (512 * (S2PT_MODE + 2) + 1) * 8;
            assert_eq!(pte_offset, offset as u64);
            assert_eq!(pte_value, 2059);
            assert_eq!(pte_level as i32, S2PT_MODE - 1);
        }

        /* Check map_page by invalid hpa */
        #[test]
        fn test_map_page_invalid_hpa() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);
            let valid_gpa = 0x1000;
            let invalid_hpa = 0x2100;

            /* Create a page table */
            let result =
                gsmmu.map_page(valid_gpa, invalid_hpa, PTE_READ | PTE_EXECUTE);
            if result.is_some() {
                panic!("HPA: {:x} should be invalid", invalid_hpa);
            }
        }

        /* Check map_page by invalid gpa */
        #[test]
        fn test_map_page_invalid_gpa() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);
            let valid_hpa = 0x2000;
            let invalid_gpa = 0x1100;

            /* Create a page table */
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
            let mem_size = 2 << 30;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let mmio_regions: Vec<gparegion::GpaRegion> = Vec::new();
            let guest_mem = GuestMemory::new().unwrap();
            let mut gsmmu = GStageMmu::new(ioctl_fd, mem_size, guest_mem, mmio_regions);
            let root_ptr = gsmmu.page_table.vaddr as *mut u64;
            let ptr: *mut u64;
            let mut pte: u64;

            /* PTE = 2063 */
            gsmmu.map_page(0x1000, 0x2000, PTE_READ | PTE_WRITE
                | PTE_EXECUTE); 

            unsafe {
                let offset = 512 * (S2PT_MODE + 2) + 1;
                ptr = root_ptr.add(offset as usize);
            }
            pte = unsafe { *ptr };

            assert_eq!(pte, 2063);

            /* PTE = 2059 */
            gsmmu.map_protect(0x1000, PTE_READ | PTE_EXECUTE);

            pte = unsafe { *ptr };

            assert_eq!(pte, 2059);
        }
    }
}
