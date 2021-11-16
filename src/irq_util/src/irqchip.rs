pub trait IrqChip: Send + Sync {
    fn mmio_callback(&self, addr: u64, data: &mut u32, is_write: bool);

    fn trigger_level_irq(&self, irq: u32, level: bool);
    
    fn trigger_edge_irq(&self, irq: u32);

    /* TODO: Vcpu should find running vcpus via plic, remove it */
    fn trigger_virtual_irq(&self, vcpu_id: u32) -> bool;
}

static mut total_cnt: usize = 0;
static mut total_time: usize = 0;
static mut ucause_cnt: [usize; 12] = [0; 12];
static mut ucause_time: [usize; 12] = [0; 12];
static mut irq_resp_cnt: usize = 0;
static mut irq_resp_time: usize = 0;
static mut shared_mem_hva: *mut u64 = 0 as *mut u64;
static mut NO_AVAIL_CNT: usize = 0;
static mut DEBUG_FLAG: bool = false;

pub struct SharedStat {}

impl SharedStat {
    pub fn set_debug_flag(val: bool) {
        unsafe {
            DEBUG_FLAG = val;
        }
    }

    pub fn get_debug_flag() -> bool {
        unsafe {
            return DEBUG_FLAG;
        }
    }

    pub fn get_shared_mem(idx: usize) -> u64 {
        unsafe {
            return *shared_mem_hva.add(idx);
        }
    }

    pub fn set_shared_mem(idx: usize, val: u64) {
        unsafe {
            *shared_mem_hva.add(idx) = val;
        }
    }

    pub fn set_shared_memory_hva(hva: u64) {
        unsafe {
            asm!("fence iorw, iorw");
            shared_mem_hva = hva as *mut u64;
            println!("--- shared_mem hva {:x} = {} {} {} {} {} {}",
                shared_mem_hva as u64,
                SharedStat::get_shared_mem(0), SharedStat::get_shared_mem(1),
                SharedStat::get_shared_mem(2), SharedStat::get_shared_mem(3),
                SharedStat::get_shared_mem(4), SharedStat::get_shared_mem(5));
        }
    }

    pub fn add_irq_resp_time(resp_time: usize) {
        unsafe {
            irq_resp_cnt += 1;
            irq_resp_time += resp_time;
        }
    }

    pub fn add_total_cnt(time: usize) {
        unsafe {
            total_time += time;
            total_cnt += 1;
        }
    }
    
    pub fn add_cnt(ucause: usize, time: usize) {
        unsafe {
            ucause_time[ucause] += time;
            ucause_cnt[ucause] += 1;
        }
    }
    
    pub fn cnt_no_avail() {
        unsafe {
            NO_AVAIL_CNT += 1;
        }
    }
    
    pub fn print_all() {
        unsafe {
            println!(">>> VM exit: time {}, cnt {}, avg {}, NO_AVAIL_CNT {} \n \
                \t\t time {} {} {} {}\n \
                \t\t {} {} {} {}\n \
                \t\t cnt {} {} {} {}\n \
                \t\t {} {} {} {}",
                total_time, total_cnt, total_time / total_cnt, NO_AVAIL_CNT,
                ucause_time[0], ucause_time[1], ucause_time[2], ucause_time[3],
                ucause_time[4], ucause_time[5], ucause_time[6], ucause_time[7],
                ucause_cnt[0], ucause_cnt[1], ucause_cnt[2], ucause_cnt[3],
                ucause_cnt[4], ucause_cnt[5], ucause_cnt[6], ucause_cnt[7]);
            //println!("  total {}, cnt {}, avg {}\n \
            //    \t\t {} {} {} {} {} {}",
            //    SharedStat::get_shared_mem(6), SharedStat::get_shared_mem(7),
            //    SharedStat::get_shared_mem(6) / SharedStat::get_shared_mem(7),
            //    SharedStat::get_shared_mem(0), SharedStat::get_shared_mem(1),
            //    SharedStat::get_shared_mem(2), SharedStat::get_shared_mem(3),
            //    SharedStat::get_shared_mem(4), SharedStat::get_shared_mem(5));
        }
    }

    pub fn reset_all() {
        unsafe {
            total_time = 0;
            total_cnt = 0;
            irq_resp_cnt = 0;
            irq_resp_time = 0;
            NO_AVAIL_CNT = 0;
            for i in 0..12 {
                ucause_time[i] = 0;
                ucause_cnt[i] = 0;
            }
            //for i in 0..8 {
            //    SharedStat::set_shared_mem(i, 0);
            //}
            asm!("fence iorw, iorw");
        }
    }
}
