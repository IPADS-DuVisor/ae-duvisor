use crate::vm::virtualmachine;
use crate::mm::gstagemmu::*;
use crate::plat::uhe::ioctl::ioctl_constants::*;

/* HPA of PLIC */
const PLIC_BASE: u64 = 0xc000000;

/* Offset of the first context */
const CONTEXT_OFFSET: u64 = 0x200000;

/* Distance between the contexts of each hart */
const CONTEXT_PER_HART: u64 = 0x3000;

/* Offset of vplic in each context */
const VPLIC_OFFSET: u64 = 0x2000;

const CONTEXT_BASE: u64 = PLIC_BASE + CONTEXT_OFFSET;
const CONTEXT_VPLIC_BASE: u64 = CONTEXT_BASE + VPLIC_OFFSET;

pub struct Vplic {
    pub ioctl_fd: i32,
    pub vinterrupt_ptr: *mut u32,
}

impl Vplic {
    pub fn new(ioctl_fd: i32) -> Self {
        let vinterrupt_ptr: *mut u32;

        unsafe {
            let addr = 0 as *mut libc::c_void;
            println!("Try to get vnterrupt");
            let mmap_ptr = libc::mmap(addr, 0x10000 as usize, 
                libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, ioctl_fd, 0);
            println!("Try to get vnterrupt end");

            vinterrupt_ptr = mmap_ptr as *mut u32;
            //let vinterrupt: u64 = *vinterrupt_ptr;
            println!("laputa: vinterrupt 0x{:x}", vinterrupt_ptr as u64);
        }

        Self {
            ioctl_fd,
            vinterrupt_ptr,
        }
    }

    pub fn read_vnterrupt(&self) -> u32 {
        let vinterrupt: u32;

        println!("read_vnterrupt");

        unsafe {
            vinterrupt = *(self.vinterrupt_ptr);
        }

        println!("Read vinterrupt 0x{:x}", vinterrupt);

        vinterrupt
    }

    pub fn write_vnterrupt(&self, value: u32) {
        let vinterrupt: u32;

        println!("write_vnterrupt");
        unsafe {
            *(self.vinterrupt_ptr) = value;

            let vinterrupt_ptr2: *mut u64 = self.vinterrupt_ptr as *mut u64;
            *vinterrupt_ptr2 = 0x1010101010101010;
            vinterrupt = *(self.vinterrupt_ptr);
        }

        println!("Write value 0x{:x}, vinterrupt 0x{:x}", value, vinterrupt);
    }
}

pub fn create_vplic(vm: &virtualmachine::VirtualMachine, ioctl_fd: i32) {
    let flag: u64 = PTE_VRWEU | PTE_DIRTY | PTE_ACCESS;
    let mut vplic_addr: u64;
    let mut guest_plic_addr: u64;
    let mut hartid: u64;

    println!("Create vplic");

    /* Map 0xc000000 to 0xc200000 */ //0xfffffff080401004  0x80200000 0x80400000
    for i in 0..0x200 {
        vm.vm_state.gsmmu.lock().unwrap().map_page(0xc000000 + i * 0x1000, 0xc000000 + i * 0x1000, flag);
        //vm.vm_state.gsmmu.lock().unwrap().map_page(0xc000000 + i * 0x1000, 0xc000000 + i * 0x1000, flag);
        //vm.vm_state.gsmmu.lock().unwrap().map_page(0x80200000 + i * 0x1000, 0xc000000 + i * 0x1000, flag);
        //vm.vm_state.gsmmu.lock().unwrap().map_page(0xfffffff080200000 + i * 0x1000, 0xc000000 + i * 0x1000, flag);
    }
    //vm.vm_state.gsmmu.lock().unwrap().map_page(0xc002000, 0xc002000, flag);

    for i in 0..vm.vcpu_num {
        hartid = i as u64 + 1;
        unsafe {
            libc::ioctl(ioctl_fd, IOCTL_LAPUTA_GET_CPUID, hartid);
        }
        println!("vcpuid: 0x{:x}, cpuid: 0x{:x}, hartid: 0x{:x}", i, i+1, hartid);
        vplic_addr = CONTEXT_VPLIC_BASE + i as u64 * CONTEXT_PER_HART;

        /* Guest should start from 0xc201000 due to the M-mode */
        //guest_plic_addr = 0x80400000 + 0x1000 + i as u64 * 0x2000;
        //guest_plic_addr = 0xfffffff080400000 + 0x1000 + i as u64 * 0x2000;
        //guest_plic_addr = 0xc200000 + 0x1000 + i as u64 * 0x2000;
        guest_plic_addr = 0xc200000 + 0x2000 + i as u64 * 0x3000;

        #[cfg(feature = "qemu")]
        vm.vm_state.gsmmu.lock().unwrap().map_page(guest_plic_addr, vplic_addr, flag);

        #[cfg(feature = "xilinx")]
        vm.vm_state.gsmmu.lock().unwrap().map_page(guest_plic_addr, vplic_addr, flag | PTE_ACCESS | PTE_DIRTY);

        println!("gpa: 0x{:x}, hpa: 0x{:x}", guest_plic_addr, vplic_addr);
    }
}

/* pub fn get_vnterrupt_page(fd: i32) {
    let mmap_ptr = libc::mmap(addr, 0x1000 as usize, 
        libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0);
    assert_ne!(mmap_ptr, libc::MAP_FAILED);
} */

