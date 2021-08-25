use crate::vcpu::virtualcpu;
#[allow(unused_imports)]
use crate::vcpu::vcpucontext::GpRegs;
use crate::mm::gstagemmu;
use crate::plat::uhe::ioctl::ioctl_constants;
use crate::irq::delegation::delegation_constants;
use std::thread;
use std::sync::{Arc, Mutex, RwLock};
use std::ffi::CString;
use ioctl_constants::*;
use delegation_constants::*;
use crate::mm::utils::*;
use crate::init::cmdline::VMConfig;
use crate::vm::image;
use crate::mm::gparegion::GpaRegion;
use crate::vm::dtb;
use crate::devices::plic::Plic;
use crate::devices::tty::Tty;
use std::fs::{OpenOptions};
#[allow(unused_imports)]
use std::net::{Ipv4Addr};
use crate::vcpu::utils::*;
use crate::irq::vipi::VirtualIpi;

extern crate irq_util;
use irq_util::IrqChip;

extern crate devices;
extern crate sys_util;
use sys_util::GuestMemory;

#[allow(unused)]
extern "C"
{
    fn vcpu_ecall_exit();
    fn vcpu_ecall_exit_end();
    fn vcpu_add_all_gprs();
    fn vcpu_add_all_gprs_end();
    fn vmem_ld_mapping();
    fn vmem_ld_mapping_end();
    fn vmem_W_Ro();
    fn vmem_W_Ro_end();
    fn vmem_X_nonX();
    fn vmem_X_nonX_end();
    fn vmem_ld_sd_over_loop();
    fn vmem_ld_sd_over_loop_end();
    fn vmem_ld_sd_sum();
    fn vmem_ld_sd_sum_end();
    fn vmem_ld_data();
    fn vmem_ld_data_end();
}

#[allow(unused)]
extern "C"
{
    fn getchar_emulation() -> i32;
}

/* Export to vcpu */
pub struct VmSharedState {
    pub vm_id: u32,
    pub ioctl_fd: i32,
    pub gsmmu: gstagemmu::GStageMmu,
}

impl VmSharedState {
    pub fn new(ioctl_fd: i32, mem_size: u64, mmio_regions: Vec<GpaRegion>)
        -> Self {
        Self {
            vm_id: 0,
            ioctl_fd,
            gsmmu: gstagemmu::GStageMmu::new(ioctl_fd, mem_size, mmio_regions),
        }
    }
}

pub struct VirtualMachine {
    pub vm_state: Arc<Mutex<VmSharedState>>,
    pub vcpus: Vec<Arc<virtualcpu::VirtualCpu>>,
    pub vcpu_num: u32,
    pub mem_size: u64,
    pub vm_image: image::VmImage,
    pub dtb_file: dtb::DeviceTree,
    pub initrd_path: String,
    pub irqchip: Arc<dyn IrqChip>,
    /* TODO: More consoles, not only tty */
    pub console: Arc<Mutex<Tty>>,
    pub io_thread: bool,
    pub mmio_bus: Arc<RwLock<devices::Bus>>,
    /* Record GPA <--> HVA mappings */
    pub guest_mem: GuestMemory,
    pub vipi: Arc<Mutex<VirtualIpi>>,
}

impl VirtualMachine {
    pub fn open_ioctl() -> i32 {
        let file_path = CString::new("/dev/laputa_dev").unwrap();
        let ioctl_fd;

        unsafe {
            ioctl_fd = (libc::open(file_path.as_ptr(), libc::O_RDWR)) as i32;
            if ioctl_fd == -1 {
                panic!("Open /dev/laputa_dev failed");
            }
        }

        ioctl_fd
    }
    
    fn create_block_dev(mmio_bus: &Arc<RwLock<devices::Bus>>,
        guest_mem: &GuestMemory, irqchip: &Arc<Plic>) {
        let root_image = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/blk-dev.img")
            .unwrap();

        let block_box = Box::new(devices::virtio::Block::new(root_image).unwrap());
        
        let mmio_blk = devices::virtio::MmioDevice::new(
            guest_mem.clone(), block_box, irqchip.clone()).unwrap();

        mmio_bus.write().unwrap().insert(Arc::new(Mutex::new(mmio_blk)), 
            0x10000000, 0x200).unwrap();
    }

    #[allow(unused)]
    fn create_network_dev(mmio_bus: &Arc<RwLock<devices::Bus>>,
        guest_mem: &GuestMemory, irqchip: &Arc<Plic>) {
        let net_box = Box::new(devices::virtio::Net::new(
                Ipv4Addr::new(192, 168, 254, 2), /* IP */
                Ipv4Addr::new(255, 255, 0, 0) /* NETMASK */
                ).unwrap());
        
        let mmio_net = devices::virtio::MmioDevice::new(
            guest_mem.clone(), net_box, irqchip.clone()).unwrap();

        mmio_bus.write().unwrap().insert(Arc::new(Mutex::new(mmio_net)), 
            0x10000200, 0x200).unwrap();
    }

    pub fn new(vm_config: VMConfig) -> Self {
        let vcpu_num = vm_config.vcpu_count;
        let mem_size = vm_config.mem_size << GB_SHIFT;
        let elf_path = &vm_config.kernel_img_path[..];
        let dtb_path = &vm_config.dtb_path[..];
        let mut mmio_regions = vm_config.mmio_regions;
        let mut vcpus: Vec<Arc<virtualcpu::VirtualCpu>> = Vec::new();
        let vm_image = image::VmImage::new(elf_path);
        let dtb_file = dtb::DeviceTree::new(dtb_path);
        let initrd_path = vm_config.initrd_path;
        let tty = Tty::new();

        /* Mmio default config for unit tests */
        #[cfg(test)]
        mmio_regions.push(GpaRegion {
            gpa: 0x0,
            length: 0x1000,
        });
        
        #[cfg(not(test))]
        /* 
         * Mmio default config for Linux VM
         * FIXME: read memory range from DTB
         */
        mmio_regions.push(GpaRegion {
            gpa: 0x0,
            length: 0x80000000,
        });

        #[cfg(test)]
        let io_thread = false;

        #[cfg(not(test))]
        let io_thread = true;

        /* Get ioctl fd of "/dev/laputa_dev" */
        let ioctl_fd = VirtualMachine::open_ioctl();

        let vm_state = VmSharedState::new(ioctl_fd, mem_size, mmio_regions);
        let vm_state_mutex = Arc::new(Mutex::new(vm_state));
        let console = Arc::new(Mutex::new(tty));

        let vipi = VirtualIpi::new();
        let vipi_mutex = Arc::new(Mutex::new(vipi));

        let mmio_bus = Arc::new(RwLock::new(devices::Bus::new()));
        let guest_mem = GuestMemory::new().unwrap();
        
        /* Create vcpu struct instances */
        for i in 0..vcpu_num {
            let vcpu = Arc::new(virtualcpu::VirtualCpu::new(i,
                    vm_state_mutex.clone(), console.clone(), 
                    guest_mem.clone(), mmio_bus.clone(), vipi_mutex.clone()));
            vcpus.push(vcpu);
        }
        
        let irqchip = Arc::new(Plic::new(&vcpus));
        
        VirtualMachine::create_block_dev(&mmio_bus, &guest_mem, &irqchip);

        /* 
         * The net device supports only one process which will 
         * crush the test cases. 
         */
        #[cfg(not(test))]
        VirtualMachine::create_network_dev(&mmio_bus, &guest_mem, &irqchip);
        
        for vcpu in &vcpus {
            vcpu.irqchip.set(irqchip.clone()).ok();
        }

        Self {
            vcpus,
            vcpu_num,
            vm_state: vm_state_mutex.clone(),
            mem_size,
            vm_image,
            dtb_file,
            initrd_path,
            irqchip,
            console,
            io_thread,
            mmio_bus,
            guest_mem,
            vipi: vipi_mutex.clone(),
        }
    }

    fn load_file_to_mem(dst: u64, src: u64, size: u64) {
        unsafe {
            std::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8,
                size as usize);

            dbgprintln!("copy_nonoverlapping: dst {:x}, src {:x}, size {:x}",
                dst, src, size);
        }
    }

    /* Init gpa block according to the elf file return for test */
    pub fn init_gpa_block_elf(&mut self) -> Vec<u64> {
        let mut hva_list: Vec<u64> = Vec::new();
        let mut offset: u64;
        let mut gpa: u64;
        let mut size: u64;
        let mut ph_data_ptr: u64;
        let img_data_ptr = self.vm_image.file_data.as_ptr() as u64;

        for i in &self.vm_image.elf_file.phdrs {
            /* Only PT_LOAD should be init */
            if i.progtype != elf::types::PT_LOAD {
                continue;
            }

            offset = i.offset;
            gpa = i.vaddr;
            size = i.filesz;
            ph_data_ptr = img_data_ptr + offset;

            let res = self.vm_state.lock().unwrap().gsmmu
                .gpa_block_add(gpa, page_size_round_up(size));
            if !res.is_ok() {
                panic!("gpa block add failed");
            }

            let (hva, _hpa) = res.unwrap();
            hva_list.push(hva);

            VirtualMachine::load_file_to_mem(hva, ph_data_ptr, size);
        }

        /* Return for test */
        hva_list
    }

    /* Load DTB data to DTB_GPA */
    pub fn init_gpa_block_dtb(&mut self) -> Option<(u64, u64)> {
        let dtb_gpa: u64 = dtb::DTB_GPA;
        let dtb_size: u64 = self.dtb_file.file_data.len() as u64;

        let res = self.vm_state.lock().unwrap().gsmmu.gpa_block_add(dtb_gpa,
                page_size_round_up(dtb_size));
        if !res.is_ok() {
            return None;
        }

        let (hva, _hpa) = res.unwrap();
        let dtb_data_ptr = self.dtb_file.file_data.as_ptr() as u64;
        VirtualMachine::load_file_to_mem(hva, dtb_data_ptr, dtb_size);

        println!("Load DTB to 0x{:x}, size 0x{:x}", dtb_gpa, dtb_size);

        return Some((dtb_gpa, hva));
    }

    /* Load initrd image to the location specified by DTB */
    pub fn init_gpa_block_initrd(&mut self) -> Option<(u64, u64)> {
        let initrd_gpa: u64 = self.dtb_file.meta_data.initrd_region.start;
        let initrd_end: u64 = self.dtb_file.meta_data.initrd_region.end;
        let initrd_size: u64 = initrd_end - initrd_gpa;
        
        if initrd_gpa == 0 || initrd_end == 0 {
            println!("No initrd config in DTB");
            return None;
        }

        let page_offset: u64 = initrd_gpa & PAGE_SIZE_MASK;
        let initrd_path: &str = &self.initrd_path[..];

        /* Read initrd data */
        let initrd_data_res = std::fs::read(initrd_path);
        if initrd_data_res.is_err() {
            return None;
        }

        let initrd_data = initrd_data_res.unwrap();
        let initrd_res = self.vm_state.lock().unwrap().gsmmu.gpa_block_add(
                initrd_gpa - page_offset,
                page_size_round_up(initrd_size + page_offset));
        if !initrd_res.is_ok() {
            return None;
        }

        let (hva, _hpa) = initrd_res.unwrap();
        let initrd_data_ptr = initrd_data.as_ptr() as u64;
        VirtualMachine::load_file_to_mem(hva + page_offset, initrd_data_ptr,
                initrd_data.len() as u64);

        dbgprintln!("Initrd load finish");

        println!("Load initrd to 0x{:x}, size 0x{:x}", initrd_gpa,
            initrd_end - initrd_gpa);

        return Some((initrd_gpa, hva + page_offset));
    }

    /* Init gpa block according to the kernel data image return for test */
    pub fn init_gpa_block_data(&mut self, gpa: u64) -> Vec<u64> {
        let mut hva_list: Vec<u64> = Vec::new();
        let img_data_ptr = self.vm_image.file_data.as_ptr() as u64;
        let size: u64 = self.vm_image.file_data.len() as u64;

        dbgprintln!("Loading IMAGE_TYPE_DATA ...");

        if size == 0 {
            println!("Zero size kernel is not allowed.");
            return hva_list;
        }

        let res = self.vm_state.lock().unwrap().gsmmu.gpa_block_add(gpa,
            page_size_round_up(size));
        if !res.is_ok() {
            println!("gpa block add failed");
            return hva_list;
        }

        let (hva, _hpa) = res.unwrap();
        hva_list.push(hva);
        
        VirtualMachine::load_file_to_mem(hva, img_data_ptr, size);

        println!("Load kernel to 0x{:x}, size 0x{:x}", gpa, size);

        /* Return for test */
        hva_list
    }

    /* Init vm & vcpu before vm_run(), return for test */
    pub fn vm_init(&mut self) -> Vec<u64> {
        let ioctl_fd = self.vm_state.lock().unwrap().ioctl_fd;
        let mut dtb_gpa: u64 = 0;
        let mut kernel_gpa: u64 = image::RISCV_RAM_GPA_START + image::KERNEL_OFFSET;

        /* Delegate traps via ioctl */
        VirtualMachine::hu_delegation(ioctl_fd);
        self.vm_state.lock().unwrap().gsmmu.allocator.set_ioctl_fd(ioctl_fd);

        /* Load DTB */
        let dtb_res = self.init_gpa_block_dtb();
        if dtb_res.is_none() {
            println!("Load DTB failed");
        } else {
            let (gpa, _hva) = dtb_res.unwrap();
            dtb_gpa = gpa
        }

        /* Set up init state as opensbi for kernel */
        for i in &mut self.vcpus {
            let vcpu_id = i.vcpu_id;
            i.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10] = vcpu_id as u64;

            /* Dtb should be pointed by a1 */
            i.vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[11] = dtb_gpa;

            /* Enable the access to timer from vm */
            i.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.hucounteren = 0xffffffff;

            /* Set up the entry point of vm */
            if self.vm_image.file_type == image::IMAGE_TYPE_ELF {
                kernel_gpa = self.vm_image.elf_file.ehdr.entry;
            }

            i.vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc = kernel_gpa;
        }

        /* Load initrd image and it must come after DTB-loading */
        let initrd_res = self.init_gpa_block_initrd();
        if initrd_res.is_none() {
            println!("Load initrd image failed");
        }

        if self.vm_image.file_type == image::IMAGE_TYPE_ELF {
            /* Init gpa block from the elf file, return for test */
            return self.init_gpa_block_elf();
        } else {
            return self.init_gpa_block_data(kernel_gpa);
        }
    }

    pub fn vm_img_load(&mut self, gpa_start: u64, length: u64) -> u64{
        let res = self.vm_state.lock().unwrap().
            gsmmu.gpa_block_add(gpa_start, length);
        if !res.is_ok() {
            panic!("vm_img_load failed");
        }

        let (hva, _hpa) = res.unwrap();
        dbgprintln!("New hpa: {:x}", _hpa);

        VirtualMachine::load_file_to_mem(hva, gpa_start, length);

        gpa_start
    }

    pub fn read_poll_startup(&mut self) -> thread::JoinHandle<()>{
        let handle: thread::JoinHandle<()>;
        let console = self.console.clone();

        println!("IO thread start polling");

        handle = thread::spawn(move || {
            loop {
                unsafe {
                    let input = getchar_emulation();
                    let input_char: u8 = (input & 0xff) as u8;

                    let res = console.lock().unwrap().recv_char(input_char as char);
                    if res == 1 {
                        println!("Full!");
                    }   
                }
            }
        });

        handle
    }

    pub fn vm_run(&mut self) {
        let mut vcpu_handle: Vec<thread::JoinHandle<()>> = Vec::new();
        let mut handle: thread::JoinHandle<()>;
        let io_handle: thread::JoinHandle<()>;
        let delta_time :i64 = unsafe { csrr!(TIME) as i64 };

        /* IO thread */
        if self.io_thread {
            io_handle = self.read_poll_startup();
            vcpu_handle.push(io_handle);
        }

        for i in &self.vcpus {
            let vcpu = i.clone();

            /* Start vcpu threads! */
            handle = thread::spawn(move || {
                vcpu.thread_vcpu_run(delta_time);
            });

            vcpu_handle.push(handle);
        }

        for i in vcpu_handle {
            i.join().unwrap();
        }
    }

    pub fn vm_destroy(&mut self) {
        unsafe {
            libc::close(self.vm_state.lock().unwrap().ioctl_fd);
        }
    }

    #[allow(unused)]
    pub fn hu_delegation(ioctl_fd: i32) {
        unsafe {
            let edeleg = ((1 << EXC_VIRTUAL_SUPERVISOR_SYSCALL) |
                (1 << EXC_INST_GUEST_PAGE_FAULT) | 
                (1 << EXC_VIRTUAL_INST_FAULT) |
                (1 << EXC_LOAD_GUEST_PAGE_FAULT) |
                (1 << EXC_STORE_GUEST_PAGE_FAULT)) as libc::c_ulong;
            let ideleg = 
                ((1 << IRQ_U_VTIMER) | (1 << IRQ_U_SOFT)) as libc::c_ulong;
            let deleg = [edeleg, ideleg];
            let deleg_ptr = (&deleg) as *const u64;

            /* Call ioctl */
            let res = libc::ioctl(ioctl_fd, IOCTL_LAPUTA_REQUEST_DELEG,
                deleg_ptr);
            dbgprintln!("ioctl result: {}", res);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::*;
    use rusty_fork::rusty_fork_test;
    use crate::mm::gstagemmu::gsmmu_constants;
    use gsmmu_constants::*;
    use crate::test::utils::configtest::test_vm_config_create;
    use libc::c_void;
    use crate::plat::opensbi::emulation::error_code::*;

    rusty_fork_test! {
        #[test]
        fn test_elf_parse() {
            let vm_config = test_vm_config_create();
            let vm = virtualmachine::VirtualMachine::new(vm_config);

            /* Answer */
            let entry_ans = 0x1000;
            let phnum_ans = 1;
            let offset_ans = 0x1000;
            let paddr_ans = 0x1000;
            let vaddr_ans = 0x1000;

            let elf_file = vm.vm_image.elf_file;
            let entry_point = elf_file.ehdr.entry;
            let phnum = elf_file.phdrs.len();

            assert_eq!(entry_ans, entry_point);
            assert_eq!(phnum_ans, phnum);

            let mut p_offset = 0;
            let mut p_paddr = 0;
            let mut p_vaddr = 0;
            for i in &elf_file.phdrs {
                p_offset = i.offset;
                p_paddr = i.paddr;
                p_vaddr = i.vaddr;
            }

            println!("test_elf_parse: offset {}, paddr {}, vaddr {}", p_offset,
                p_paddr, p_vaddr);
            
            assert_eq!(offset_ans, p_offset);
            assert_eq!(paddr_ans, p_paddr);
            assert_eq!(vaddr_ans, p_vaddr);
        }

        /* 
         * Test init_gpa_block_elf() by compare the data from hva with img 
         * file 
         */
        #[test]
        fn test_init_gpa_block_elf() {
            let vm_config = test_vm_config_create();
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);
            let hva_list: Vec<u64>;

            hva_list = vm.vm_init();

            let mut gb_gpa;
            let mut gb_hpa;
            let mut gb_length;
            let mut target_hva = 0;
            for i in &vm.vm_state.lock().unwrap().gsmmu.mem_gpa_regions {
                gb_gpa = i.gpa;
                gb_length = i.length;
                println!("gpa_regions - gpa {:x}, length {:x}", gb_gpa,
                    gb_length);
            }

            for i in &vm.vm_state.lock().unwrap().gsmmu.gpa_blocks {
                gb_gpa = i.gpa;
                gb_hpa = i.hpa;
                gb_length = i.length;
                println!("gpa_blocks - gpa {:x}, hpa {:x}, length {:x}",
                    gb_gpa, gb_hpa, gb_length);
            }

            for i in hva_list {
                println!("hva_list {:x}", i);
                target_hva = i;
            }

            /* Extract answer from the img file */
            let mut elf_data_ans: u64 = 0x9092908E908A40A9;
            let mut elf_data: u64;
            unsafe {
                elf_data = *(target_hva as *mut u64);
                println!("elf_data {:x}", elf_data);
            }

            assert_eq!(elf_data_ans, elf_data);

            elf_data_ans = 0x90F290EE90EA90E6;
            unsafe {
                elf_data = *((target_hva + 0x30) as *mut u64);
                println!("elf_data {:x}", elf_data);
            }

            assert_eq!(elf_data_ans, elf_data);

            elf_data_ans = 0x0;
            unsafe {
                elf_data = *((target_hva + 0x100) as *mut u64);
                println!("elf_data {:x}", elf_data);
            }

            assert_eq!(elf_data_ans, elf_data);
        }

        #[test]
        fn test_vm_new() { 
            let vcpu_num = 1;
            let vm_config = test_vm_config_create();
            let vm = virtualmachine::VirtualMachine::new(vm_config);

            assert_eq!(vm.vcpu_num, vcpu_num);
        }

        /* Check the num of the vcpu created */
        #[test]
        fn test_vm_new_vcpu() {   
            let vcpu_num = 4;
            let mut vm_config = test_vm_config_create();
            vm_config.vcpu_count = vcpu_num;
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let mut sum = 0;

            for i in &vm.vcpus {
                sum = sum + i.vcpu_id;
            }

            /* 0 + 1 + 2 + 3 */
            assert_eq!(sum, 6); 
        }

        #[test]
        fn test_ecall_getchar_sum() {
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/opensbi_getchar_sum.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                = entry_point;

            vm.vm_run();

            /* T0 should be '\n' to end */
            let t0: u64;
            let t0_ans: u64 = 10;
            t0 = vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[5];

            /* T1 should sum up the input */
            let t1: u64;
            let t1_ans: u64 = 1508;
            t1 = vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[6];

            vm.vm_destroy();

            assert_eq!(t0_ans, t0);
            assert_eq!(t1_ans, t1);
        }

        #[test]
        fn test_ecall_getchar_count() {
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/opensbi_getchar_count.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                = entry_point;

            vm.vm_run();

            /* T0 should be '\n' to end */
            let t0: u64;
            let t0_ans: u64 = 10;
            t0 = vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[5];

            /* T1 should sum up the input */
            let t1: u64;
            let t1_ans: u64 = 16;
            t1 = vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[6];

            vm.vm_destroy();

            assert_eq!(t0_ans, t0);
            assert_eq!(t1_ans, t1);
        }

        /* Check the magic number of DTB */
        #[test]
        fn test_dtb_check_magic() {
            let mut vm_config = test_vm_config_create();
            let dtb_path: &str = "./test-files-laputa/k210.dtb";
            let ans_magic: u32 = 0xedfe0dd0;
            vm_config.dtb_path = String::from(dtb_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            let dtb_res = vm.init_gpa_block_dtb();
            if dtb_res.is_none() {
                panic!("Load DTB failed");
            }

            let (_dtb_gpa, dtb_hva) = dtb_res.unwrap();
            let dtb_magic: u32;

            unsafe {
                dtb_magic = *(dtb_hva as *mut u32);
                dbgprintln!("DTB magic: 0x{:x}", dtb_magic);
            }

            assert_eq!(dtb_magic, ans_magic);
        }

        /* 
         * Check the correctness of DTB data in guest memory.
         * The test dtb is compiled from linux/arch/riscv/boot/dts/kendryte/
         */
        #[test]
        fn test_dtb_load_data_kendryte() {
            let mut vm_config = test_vm_config_create();
            let dtb_path: &str = "./test-files-laputa/k210.dtb";
            let ans_res = std::fs::read(dtb_path);
            let ans_data = ans_res.unwrap();
            vm_config.dtb_path = String::from(dtb_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            let dtb_res = vm.init_gpa_block_dtb();
            if dtb_res.is_none() {
                panic!("Load DTB failed");
            }

            let (_dtb_gpa, dtb_hva) = dtb_res.unwrap();
            let result: i32;
            unsafe {
                result = libc::memcmp(dtb_hva as *const c_void,
                        ans_data.as_ptr() as *const c_void,
                        ans_data.len());
            }

            assert_eq!(result, 0);
        }

        /* 
         * Check the correctness of DTB data in guest memory.
         * The test dtb is compiled from linux/arch/riscv/boot/dts/sifive/
         */
        #[test]
        fn test_dtb_load_data_sifive() {
            let mut vm_config = test_vm_config_create();
            let dtb_path: &str 
                = "./test-files-laputa/hifive-unleashed-a00.dtb";
            let ans_res = std::fs::read(dtb_path);
            let ans_data = ans_res.unwrap();
            vm_config.dtb_path = String::from(dtb_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            let dtb_res = vm.init_gpa_block_dtb();
            if dtb_res.is_none() {
                panic!("Load DTB failed");
            }

            let (_dtb_gpa, dtb_hva) = dtb_res.unwrap();
            let result: i32;
            unsafe {
                result = libc::memcmp(dtb_hva as *const c_void,
                        ans_data.as_ptr() as *const c_void,
                        ans_data.len());
            }

            assert_eq!(result, 0);
        }

        /* Test the correctness of the data from dtb */
        #[test]
        fn test_dtb_load_data_vmlinux() {
            let mut vm_config = test_vm_config_create();
            let dtb_path: &str = "./test-files-laputa/vmlinux.dtb";
            let ans_res = std::fs::read(dtb_path);
            let ans_data = ans_res.unwrap();
            vm_config.dtb_path = String::from(dtb_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            let dtb_res = vm.init_gpa_block_dtb();
            if dtb_res.is_none() {
                panic!("Load DTB failed");
            }

            let (_dtb_gpa, dtb_hva) = dtb_res.unwrap();
            let result: i32;
            unsafe {
                result = libc::memcmp(dtb_hva as *const c_void,
                        ans_data.as_ptr() as *const c_void,
                        ans_data.len());
            }

            assert_eq!(result, 0);
        }

        /* Check the result of unsupported sbi ecall */
        #[test]
        fn test_ecall_unsupported() { 
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/ecall_emulation_unsupported.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            /* Answer will be saved at 0x3000(gpa) */
            let mut retval: u64;
            let answer: [u64; 3] = [SBI_ERR_NOT_SUPPORTED as u64; 3];

            vm.vm_init();

            /* The return value will be stored on this gpa */
            let target_address = 0x3000;

            /* Set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            let res = vm.vm_state.lock().unwrap()
                .gsmmu.gpa_block_add(target_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!");
            }

            /* Get the hva of 0x3000(gpa) */
            let (hva, hpa) = res.unwrap();
            dbgprintln!("hva {:x}, hpa {:x}", hva, hpa);

            /* Map the page on g-stage */
            let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                    | PTE_EXECUTE;
            vm.vm_state.lock().unwrap().gsmmu.map_page(target_address, hpa, 
                    flag);

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            
            vm.vm_run();

            /* Check the return value stored by the vm */
            unsafe {
                for i in 0..3 {
                    retval = *((hva + 16 * i) as *mut u64);
                    assert_eq!(answer[i as usize], retval);
                }
            }

            vm.vm_destroy();
        }

        /* Check the result of remote fence sbi */
        #[test]
        fn test_ecall_remote_fence() { 
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/ecall_emulation_remote_fence.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            /* Answer will be saved at 0x3000(gpa) */
            let mut retval: u64;
            let answer: [u64; 2] = [0, 0];

            vm.vm_init();

            /* The return value will be stored on this gpa */
            let target_address = 0x3000;

            /* Set entry point */
            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            let res = vm.vm_state.lock().unwrap()
                .gsmmu.gpa_block_add(target_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!");
            }

            /* Get the hva of 0x3000(gpa) */
            let (hva, hpa) = res.unwrap();
            dbgprintln!("hva {:x}, hpa {:x}", hva, hpa);

            /* Map the page on g-stage */
            let flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                    | PTE_EXECUTE;
            vm.vm_state.lock().unwrap().gsmmu.map_page(target_address, hpa, 
                    flag);

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                    = entry_point;
            
            vm.vm_run();

            /* Check the return value store by the vm */
            unsafe {
                for i in 0..2 {
                    retval = *((hva + 8 * i) as *mut u64);
                    assert_eq!(answer[i as usize], retval);
                }
            }

            vm.vm_destroy();
        }

        #[test]
        fn test_vm_huge_mapping() { 
            println!("---------start test_vm_huge_mapping------------");
            let exit_reason_ans = 0xdead;
            let exit_reason;
            let mut vm_config = test_vm_config_create();
            vm_config.mmio_regions.pop();
            vm_config.mmio_regions.pop();
            vm_config.mmio_regions.pop();

            /* Cancel the three mmio regions */
            vm_config.mmio_regions = Vec::new();

            let elf_path: &str 
                = "./tests/integration/vmem_ld_sd_over_loop.img";
            vm_config.kernel_img_path = String::from(elf_path);

            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                = entry_point;

            vm.vm_run();
            
            exit_reason = vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.gp_regs
                .x_reg[0];
            println!("exit reason {:x}", exit_reason);

            vm.vm_destroy();

            assert_eq!(exit_reason_ans, exit_reason);
        }

        #[test]
        fn test_vm_ld_sd_sum() { 
            println!("---------start test_vm_ld_sd_sum------------");
            let mut sum_ans = 0;
            let sum;
            let mut vm_config = test_vm_config_create();
            vm_config.mmio_regions.pop();
            vm_config.mmio_regions.pop();
            vm_config.mmio_regions.pop();

            /* Cancel the three mmio regions */
            vm_config.mmio_regions = Vec::new();
            
            let elf_path: &str = "./tests/integration/vmem_ld_sd_sum.img";
            vm_config.kernel_img_path = String::from(elf_path);

            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            /* Sum up 0..100 twice */
            for i in 0..100 {
                sum_ans += i;
            }
            sum_ans *= 2;

            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                = entry_point;

            vm.vm_run();
            
            sum = vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs
                .x_reg[29];
            println!("sum {}", sum);

            vm.vm_destroy();

            assert_eq!(sum_ans, sum);
        }

        #[test]
        fn test_ecall_putchar() { 
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/opensbi_putchar.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                = entry_point;

            vm.vm_run();

            let t0: u64;
            let t1: u64;

            /* Sum up the chars in "Hello Ecall\n" */
            let t0_ans: u64 = 1023;

            /* All the ecall should should return 0 */ 
            let t1_ans: u64 = 0;

            t0 = vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs
                .x_reg[5];
            t1 = vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs
                .x_reg[6];

            vm.vm_destroy();

            assert_eq!(t0_ans, t0);
            assert_eq!(t1_ans, t1);
        }

        #[test]
        fn test_vm_add_all_gprs() { 
            println!("---------start vm------------");
            let sum_ans = 10;
            let mut sum = 0;
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/vcpu_add_all_gprs.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);
            
            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            /* A1 could be set to the address of fdt, so clear it */
            vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[11] = 0;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                = entry_point;

            vm.vm_run();
                
            sum += vm.vcpus[0].vcpu_ctx.lock().unwrap().guest_ctx.gp_regs.x_reg[10];

            vm.vm_destroy();

            assert_eq!(sum, sum_ans);
        }

        #[test]
        fn test_vmem_mapping() { 
            let exit_reason_ans = 0xdead;
            let exit_reason;
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/vmem_ld_mapping.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                = entry_point;

            vm.vm_run();
            
            exit_reason = vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.gp_regs
                .x_reg[0];
            println!("exit reason {:x}", exit_reason);

            vm.vm_destroy();

            assert_eq!(exit_reason, exit_reason_ans);
        }

        #[test]
        fn test_vmem_ro() { 
            let exit_reason_ans = 2; /* G-stage page fault for no permission */
            let exit_reason;
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/vmem_W_Ro.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let ro_address = 0x3000;

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            let res = vm.vm_state.lock().unwrap()
                .gsmmu.gpa_block_add(ro_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!")
            }

            let (_hva, hpa) = res.unwrap();
            let mut flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE 
                | PTE_EXECUTE;

            vm.vm_state.lock().unwrap().gsmmu.map_page(ro_address, hpa, flag);

            /* Read-only */
            flag = PTE_USER | PTE_VALID | PTE_READ;
            vm.vm_state.lock().unwrap().gsmmu.map_protect(ro_address, flag);

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                = entry_point;
                
            vm.vm_run();
                
            exit_reason = vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.gp_regs
                .x_reg[0];
            vm.vm_destroy();

            assert_eq!(exit_reason, exit_reason_ans);
        }

        #[test]
        fn test_vmem_nx() { 
            let exit_reason_ans = 2; /* G-stage page fault for no permission */
            let exit_reason;
            let mut vm_config = test_vm_config_create();
            let elf_path: &str = "./tests/integration/vmem_X_nonX.img";
            vm_config.kernel_img_path = String::from(elf_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            vm.vm_init();

            let nx_address = 0x3000;

            let entry_point: u64 = vm.vm_image.elf_file.ehdr.entry;

            let res = vm.vm_state.lock().unwrap()
                .gsmmu.gpa_block_add(nx_address, PAGE_SIZE);
            if !res.is_ok() {
                panic!("gpa region add failed!")
            }

            let (_hva, hpa) = res.unwrap();
            let mut flag: u64 = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE
                | PTE_EXECUTE;

            vm.vm_state.lock().unwrap().gsmmu.map_page(nx_address, hpa, flag);

            /* Non-execute */
            flag = PTE_USER | PTE_VALID | PTE_READ | PTE_WRITE;
            vm.vm_state.lock().unwrap().gsmmu.map_protect(nx_address, flag);

            vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.hyp_regs.uepc
                = entry_point;
            
            vm.vm_run();
            
            exit_reason = vm.vcpus[0].vcpu_ctx.lock().unwrap().host_ctx.gp_regs
            .x_reg[0];

            vm.vm_destroy();

            assert_eq!(exit_reason, exit_reason_ans);
        }

        /* Test the correctness of the data from initrd */
        #[test]
        fn test_initrd_load_data_vmlinux() {
            let mut vm_config = test_vm_config_create();
            let dtb_path: &str = "./test-files-laputa/vmlinux.dtb";
            vm_config.dtb_path = String::from(dtb_path);
            let initrd_path: &str = "./test-files-laputa/rootfs-vm.img";
            vm_config.initrd_path = String::from(initrd_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            let ans_res = std::fs::read(initrd_path);
            if ans_res.is_err() {
                panic!("Ans initrd load failed");
            }
            let ans_data = ans_res.unwrap();

            let dtb_res = vm.init_gpa_block_dtb();
            if dtb_res.is_none() {
                panic!("Load DTB failed");
            }

            let initrd_res = vm.init_gpa_block_initrd();
            if initrd_res.is_none() {
                panic!("Load initrd failed");
            }

            let (_initrd_gpa, initrd_hva) = initrd_res.unwrap();
            let result: i32;
            unsafe {
                result = libc::memcmp(initrd_hva as *const c_void,
                        ans_data.as_ptr() as *const c_void,
                        ans_data.len());
            }

            assert_eq!(result, 0);
        }

        /* Test the initrd-loading function with a fake rootfs file */
        #[test]
        fn test_initrd_load_data_fake_file() {
            let mut vm_config = test_vm_config_create();
            let dtb_path: &str = "./test-files-laputa/vmlinux.dtb";
            vm_config.dtb_path = String::from(dtb_path);
            let initrd_path: &str = "./test-files-laputa/rootfs-vm.img";
            let wrong_path: &str = "./test-files-laputa/fake.img";
            vm_config.initrd_path = String::from(wrong_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            let ans_res = std::fs::read(initrd_path);
            if ans_res.is_err() {
                panic!("Ans initrd load failed");
            }
            let ans_data = ans_res.unwrap();

            let dtb_res = vm.init_gpa_block_dtb();
            if dtb_res.is_none() {
                panic!("Load DTB failed");
            }
            let initrd_res = vm.init_gpa_block_initrd();

            if initrd_res.is_none() {
                panic!("Load initrd failed");
            }

            let (_initrd_gpa, initrd_hva) = initrd_res.unwrap();
            let result: i32;
            unsafe {
                result = libc::memcmp(initrd_hva as *const c_void,
                        ans_data.as_ptr() as *const c_void,
                        ans_data.len());
            }

            assert_ne!(result, 0);
        }

        /* 
        * Test the initrd-loading function with a legal but wrong rootfs 
        * file
        */
        #[test]
        fn test_initrd_load_data_wrong_file() {
            let mut vm_config = test_vm_config_create();
            let dtb_path: &str = "./test-files-laputa/vmlinux.dtb";
            vm_config.dtb_path = String::from(dtb_path);
            let initrd_path: &str = "./test-files-laputa/rootfs-vm.img";
            let wrong_path: &str = "./test-files-laputa/rootfs-vm-wrong.img";
            vm_config.initrd_path = String::from(wrong_path);
            let mut vm = virtualmachine::VirtualMachine::new(vm_config);

            let ans_res = std::fs::read(initrd_path);
            if ans_res.is_err() {
                panic!("Ans initrd load failed");
            }
            let ans_data = ans_res.unwrap();

            let dtb_res = vm.init_gpa_block_dtb();
            if dtb_res.is_none() {
                panic!("Load DTB failed");
            }
            let initrd_res = vm.init_gpa_block_initrd();

            if initrd_res.is_none() {
                panic!("Load initrd failed");
            }

            let (_initrd_gpa, initrd_hva) = initrd_res.unwrap();
            let result: i32;
            unsafe {
                result = libc::memcmp(initrd_hva as *const c_void,
                        ans_data.as_ptr() as *const c_void,
                        ans_data.len());
            }

            assert_ne!(result, 0);
        }
    }
}
