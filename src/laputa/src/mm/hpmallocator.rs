use crate::plat::uhe::ioctl::ioctl_constants;
use ioctl_constants::*;

#[derive(Clone)]
pub struct HpmRegion {
    pub hpm_vptr: u64, /* VA */
    pub base_address: u64, /* HPA */
    pub length: u64,
    pub offset: u64,
}

impl HpmRegion {
    pub fn new(hpm_vptr: u64, base_address: u64, length: u64) -> Self {
        Self {
            hpm_vptr,
            base_address,
            length,
            offset: 0,
        }
    }

    pub fn va_to_hpa(&self, va: u64) -> Option<u64> {
        let va_base = self.hpm_vptr;
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
        let va_base = self.hpm_vptr;
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
    pub ioctl_fd: i32,
}

impl HpmAllocator {
    pub fn new(ioctl_fd: i32) -> Self {
        Self {
            hpm_region_list: Vec::new(),
            ioctl_fd,
        }
    }

    /* Call PMP for hpa region */
    pub fn pmp_alloc(&mut self) -> Option<HpmRegion> {
        let fd = self.ioctl_fd;
        let test_buf: u64; /* VA */
        let test_buf_pfn: u64; /* HPA */
        
        #[cfg(feature = "xilinx")]
        let test_buf_size: usize = 128 << 20; /* 128 MB for now */
        #[cfg(feature = "qemu")]
        let test_buf_size: usize = 1024 << 20; /* 512 MB for now */
        
        let version: u64 = 0;

        unsafe {
            let version_ptr = (&version) as *const u64;
            libc::ioctl(fd, IOCTL_LAPUTA_GET_API_VERSION, version_ptr);

            /* Get va */
            let addr = 0 as *mut libc::c_void;
            let mmap_ptr = libc::mmap(addr, test_buf_size, 
                libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0);
            assert_ne!(mmap_ptr, libc::MAP_FAILED);

            /* Get hpa */
            test_buf = mmap_ptr as u64;
            test_buf_pfn = test_buf;
            let test_buf_pfn_ptr = (&test_buf_pfn) as *const u64;
            libc::ioctl(fd, IOCTL_LAPUTA_QUERY_PFN, test_buf_pfn_ptr);
        }

        let hpm_vptr = test_buf as u64;
        let base_address = test_buf_pfn << 12;
        let length = test_buf_size as u64;

        self.ioctl_fd = fd;

        Some(HpmRegion::new(hpm_vptr, base_address, length))
    }

    pub fn find_hpm_region_by_length(&mut self, length: u64) 
        -> Option<&mut HpmRegion> {
        let mut rest: u64;

        for i in &mut self.hpm_region_list {
            rest = i.length - i.offset;

            if length <= rest {
                return Some(i);
            }
        }

        None
    }

    pub fn hpm_alloc(&mut self, length: u64) -> Option<Vec<HpmRegion>> {
        let target_hpm_region: &mut HpmRegion;
        let mut result: Vec<HpmRegion> = Vec::new();
        let result_va: u64;
        let result_pa: u64;
        let result_length: u64;

        /* Get 512 MB for now */
        if self.hpm_region_list.len() == 0 {
            let hpm_region = self.pmp_alloc().unwrap();
            self.hpm_region_list.push(hpm_region);
        }

        let target_wrap = self.find_hpm_region_by_length(length);

        if target_wrap.is_some() {
            target_hpm_region = target_wrap.unwrap();

            result_va = target_hpm_region.hpm_vptr + 
                target_hpm_region.offset;
            result_pa = target_hpm_region.base_address + 
                target_hpm_region.offset;
            result_length = length;

            result.push(HpmRegion::new(result_va, result_pa, result_length));

            /* Increase the offset */
            target_hpm_region.offset += length;

            return Some(result);
        }

        None
    }

    pub fn set_ioctl_fd(&mut self, ioctl_fd: i32) {
        self.ioctl_fd = ioctl_fd;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use rusty_fork::rusty_fork_test;
    
    rusty_fork_test! { 
        #[test]
        fn test_hpm_region_new() {
            let hpa: u64 = 0x3000;
            let va: u64 = 0x5000;
            let length: u64 = 0x1000;
            let hpm_vptr = va as u64;
            let hpm_region = HpmRegion::new(hpm_vptr, hpa, length);

            assert_eq!(hpm_region.hpm_vptr, va);
            assert_eq!(hpm_region.base_address, hpa);
            assert_eq!(hpm_region.length, length);
        }

        /* Check new() of GStageMmu */
        #[test]
        fn test_allocator_alloc() { 
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let length = 0x2000;
            let mut allocator = HpmAllocator::new(ioctl_fd);
            let result_wrap = allocator.hpm_alloc(length);
            assert!(result_wrap.is_some());

            let result = result_wrap.unwrap();
            let result_length = result.len();
            assert_eq!(1, result_length);

            let mut region_length = 0;

            for i in result {
                region_length = i.length;
            }

            assert_eq!(region_length, length);
        }

        /* Check hpa_to_va when hpa is out of bound */
        #[test]
        fn test_hpa_to_va_oob_invalid() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            /* Valid HPA: [base_addr, base_addr + 0x2000) */
            let length = 0x2000;
            let mut allocator = HpmAllocator::new(ioctl_fd);
            let result_wrap = allocator.hpm_alloc(length);
            assert!(result_wrap.is_some());

            let result = result_wrap.unwrap();
            let result_length = result.len();
            assert_eq!(1, result_length);

            let mut invalid_hpa;
            let mut res;

            for i in result {
                invalid_hpa = i.base_address;
                invalid_hpa += i.length * 2;
                res = i.hpa_to_va(invalid_hpa);
                if res.is_some() {
                    panic!("HPA {:x} should be out of bound", invalid_hpa);
                }
            }
        }

        /* Check hpa_to_va when hpa is equal to the upper boundary */
        #[test]
        fn test_hpa_to_va_oob_invalid_eq() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            /* Valid HPA: [base_addr, base_addr + 0x2000) */
            let length = 0x2000;
            let mut allocator = HpmAllocator::new(ioctl_fd);

            let result_wrap = allocator.hpm_alloc(length);
            assert!(result_wrap.is_some());

            let result = result_wrap.unwrap();
            let result_length = result.len();
            assert_eq!(1, result_length);

            let mut invalid_hpa;
            let mut res;

            for i in result {
                invalid_hpa = i.base_address;
                invalid_hpa += i.length;
                res = i.hpa_to_va(invalid_hpa);
                if res.is_some() {
                    panic!("HPA {:x} should be out of bound", invalid_hpa);
                }
            }
        }

        /* Check hpa_to_va when hpa is valid */
        #[test]
        fn test_hpa_to_va_oob_valid() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            /* Valid HPA: [base_addr, base_addr + 0x2000) */
            let length = 0x2000;
            let mut allocator = HpmAllocator::new(ioctl_fd);

            let result_wrap = allocator.hpm_alloc(length);
            assert!(result_wrap.is_some());

            let result = result_wrap.unwrap();
            let result_length = result.len();
            assert_eq!(1, result_length);

            let mut valid_hpa;
            let mut res;

            for i in result {
                valid_hpa = i.base_address;
                valid_hpa += i.length / 2;
                res = i.hpa_to_va(valid_hpa);
                if res.is_none() {
                    panic!("HPA {:x} should be valid", valid_hpa);
                }
            }
        }

        /* Check hpa_to_va when hpa is equal to the lower bound */
        #[test]
        fn test_hpa_to_va_oob_valid_eq() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            /* Valid HPA: [base_addr, base_addr + 0x2000) */
            let length = 0x2000;
            let mut allocator = HpmAllocator::new(ioctl_fd);

            let result_wrap = allocator.hpm_alloc(length);
            assert!(result_wrap.is_some());

            let result = result_wrap.unwrap();
            let result_length = result.len();
            assert_eq!(1, result_length);

            let mut valid_hpa;
            let mut res;

            for i in result {
                valid_hpa = i.base_address;
                res = i.hpa_to_va(valid_hpa);
                if res.is_none() {
                    panic!("HPA {:x} should be valid", valid_hpa);
                }
            }
        }

        /* Check va_to_hpa when va is out of bound */
        #[test]
        fn test_va_to_hpa_oob_invalid() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let length = 0x2000;
            let mut allocator = HpmAllocator::new(ioctl_fd);

            let result_wrap = allocator.hpm_alloc(length);
            assert!(result_wrap.is_some());

            let result = result_wrap.unwrap();
            let result_length = result.len();
            assert_eq!(1, result_length);

            let mut invalid_va;
            let mut res;

            for i in result {
                invalid_va = i.hpm_vptr + length + 0x1000;
                res = i.va_to_hpa(invalid_va);
                if res.is_some() {
                    panic!("VA {:x} should be out of bound", invalid_va);
                }
            }
        }

        /* Check va_to_hpa when va is equal to the upper bound */
        #[test]
        fn test_va_to_hpa_oob_invalid_eq() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let length = 0x2000;
            let mut allocator = HpmAllocator::new(ioctl_fd);

            let result_wrap = allocator.hpm_alloc(length);
            assert!(result_wrap.is_some());

            let result = result_wrap.unwrap();
            let result_length = result.len();
            assert_eq!(1, result_length);

            let mut invalid_va;
            let mut res;

            for i in result {
                invalid_va = i.hpm_vptr + length;
                res = i.va_to_hpa(invalid_va);
                if res.is_some() {
                    panic!("VA {:x} should be out of bound", invalid_va);
                }
            }
        }

        /* Check va_to_hpa when va is valid */
        #[test]
        fn test_va_to_hpa_oob_valid() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let length = 0x2000;
            let mut allocator = HpmAllocator::new(ioctl_fd);

            let result_wrap = allocator.hpm_alloc(length);
            assert!(result_wrap.is_some());

            let result = result_wrap.unwrap();
            let result_length = result.len();
            assert_eq!(1, result_length);

            let mut valid_va;
            let mut res;

            for i in result {
                valid_va = i.hpm_vptr + length - 0x1000;
                res = i.va_to_hpa(valid_va);
                if res.is_none() {
                    panic!("VA {:x} should be valid", valid_va);
                }
            }
        }

        /* Check va_to_hpa when va is equal to the lower bound */
        #[test]
        fn test_va_to_hpa_oob_valid_eq() {
            let file_path = CString::new("/dev/laputa_dev").unwrap();
            let ioctl_fd;

            unsafe {
                ioctl_fd =
                    (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            }

            let length = 0x2000;
            let mut allocator = HpmAllocator::new(ioctl_fd);

            let result_wrap = allocator.hpm_alloc(length);
            assert!(result_wrap.is_some());

            let result = result_wrap.unwrap();
            let result_length = result.len();
            assert_eq!(1, result_length);

            let mut valid_va;
            let mut res;

            for i in result {
                valid_va = i.hpm_vptr;
                res = i.va_to_hpa(valid_va);
                if res.is_none() {
                    panic!("VA {:x} should be valid", valid_va);
                }
            }
        }      
    }
    
}
