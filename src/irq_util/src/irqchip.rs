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
static mut PRODUCER_IDX: usize = 0;
static mut CONSUMER_IDX: usize = 1;

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
            if shared_mem_hva as u64 == 0 { return 0; }
            return *shared_mem_hva.add(idx);
        }
    }

    pub fn set_shared_mem(idx: usize, val: u64) {
        unsafe {
            if shared_mem_hva as u64 == 0 { return; }
            *shared_mem_hva.add(idx) = val;
        }
    }

    pub fn add_shared_mem(idx: usize, val: u64) {
        unsafe {
            if shared_mem_hva as u64 == 0 { return; }
            *shared_mem_hva.add(idx) += val;
        }
    }

    pub fn get_rx_pkt() {
        unsafe {
            if CONSUMER_IDX > 50000 || CONSUMER_IDX > PRODUCER_IDX { return; }
            let prev_time = *shared_mem_hva.add(6 + CONSUMER_IDX - 1);
            let cur_time: u64;
            asm!("csrr {}, 0xC01", out(reg) cur_time);
            *shared_mem_hva.add(50006 + CONSUMER_IDX - 1) =
                cur_time - prev_time;
            CONSUMER_IDX += 1;
        }
    }

    pub fn add_rx_pkt(time: u64) {
        unsafe {
            if PRODUCER_IDX >= 50000 { return; }
            *shared_mem_hva.add(6 + PRODUCER_IDX) = time;
            PRODUCER_IDX += 1;
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
            println!("\t VIPI sender {} {} {} {} \n \
                \t\t {} {} {} {}",
                SharedStat::get_shared_mem(110020),
                SharedStat::get_shared_mem(110021),
                SharedStat::get_shared_mem(110022),
                SharedStat::get_shared_mem(110023),
                SharedStat::get_shared_mem(110024),
                SharedStat::get_shared_mem(110025),
                SharedStat::get_shared_mem(110026),
                SharedStat::get_shared_mem(110027));
            println!("\t receiver {} {} {} {} \n \
                \t\t {} {} {} {}",
                SharedStat::get_shared_mem(32 + 0),
                SharedStat::get_shared_mem(32 + 1),
                SharedStat::get_shared_mem(32 + 2),
                SharedStat::get_shared_mem(32 + 3),
                SharedStat::get_shared_mem(32 + 4),
                SharedStat::get_shared_mem(32 + 5),
                SharedStat::get_shared_mem(32 + 6),
                SharedStat::get_shared_mem(32 + 7));
            //println!("\t block READ cnt {} time {} len {} avg {}",
            //    SharedStat::get_shared_mem(110005), SharedStat::get_shared_mem(110006),
            //    SharedStat::get_shared_mem(110007),
            //    if SharedStat::get_shared_mem(110006) == 0 { 0 } else {
            //        SharedStat::get_shared_mem(110007)
            //            / SharedStat::get_shared_mem(110006)
            //    });
            //println!("\t block WRITE cnt {} time {} len {} avg {}",
            //    SharedStat::get_shared_mem(110010), SharedStat::get_shared_mem(110011),
            //    SharedStat::get_shared_mem(110012),
            //    if SharedStat::get_shared_mem(110011) == 0 { 0 } else {
            //        SharedStat::get_shared_mem(110012)
            //            / SharedStat::get_shared_mem(110011)
            //    });
            //println!("\t block FLUSH cnt {} time {} len {} avg {}",
            //    SharedStat::get_shared_mem(110015), SharedStat::get_shared_mem(110016),
            //    SharedStat::get_shared_mem(110017),
            //    if SharedStat::get_shared_mem(110016) == 0 { 0 } else {
            //        SharedStat::get_shared_mem(110017)
            //            / SharedStat::get_shared_mem(110016)
            //    });
            //println!("\t host irq {} {} loop notify {} {} rx time len {} {} \n \
            //    \t tap cnt {} time {} {} len {} avg {}",
            //    SharedStat::get_shared_mem(0), SharedStat::get_shared_mem(1),
            //    SharedStat::get_shared_mem(2), SharedStat::get_shared_mem(3),
            //    SharedStat::get_shared_mem(4), SharedStat::get_shared_mem(5),
            //    SharedStat::get_shared_mem(110000), SharedStat::get_shared_mem(110001),
            //    SharedStat::get_shared_mem(110003), SharedStat::get_shared_mem(110002),
            //    SharedStat::get_shared_mem(110002) / SharedStat::get_shared_mem(110000));
            //let mut sum_rx = 0;
            //let mut nr_rx = 0;
            //let mut nr_zero = 0;
            //for i in 0..50000 {
            //    let prev_time = SharedStat::get_shared_mem(6 + i);
            //    let elem = SharedStat::get_shared_mem(50006 + i);
            //    if elem != 0 {
            //        sum_rx += elem;
            //        nr_rx += 1;
            //    } else if prev_time != 0 {
            //        nr_zero += 1;
            //    }
            //    if i % 10000 == 0 && nr_rx != 0 {
            //        println!("\t\t [{}] time_rx {} nr_rx {} nr_zero {} avg {}",
            //            i, sum_rx, nr_rx, nr_zero, sum_rx / nr_rx);
            //        sum_rx = 0;
            //        nr_rx = 0;
            //        nr_zero = 0;
            //    }
            //}
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
            PRODUCER_IDX = 0;
            CONSUMER_IDX = 1;
            for i in 0..131072 {
                SharedStat::set_shared_mem(i, 0);
            }
            asm!("fence iorw, iorw");
        }
    }
}
