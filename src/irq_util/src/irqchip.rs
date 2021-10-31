pub trait IrqChip: Send + Sync {
    fn mmio_callback(&self, addr: u64, data: &mut u32, is_write: bool);

    fn trigger_level_irq(&self, irq: u32, level: bool);
    
    fn trigger_edge_irq(&self, irq: u32);

    /* TODO: Vcpu should find running vcpus via plic, remove it */
    fn trigger_virtual_irq(&self, vcpu_id: u32) -> bool;
}

static mut total_cnt: usize = 0;
static mut ucause_cnt: [usize; 6] = [0; 6];

pub struct SharedStat {}

impl SharedStat {
    pub fn add_cnt(ucause: usize) {
        unsafe {
            total_cnt += 1;
            ucause_cnt[ucause] += 1;
        }
    }
    
    pub fn print_all() {
        unsafe {
            println!(">>> VM exit: total count {} \n \
                \t\t ucause count {} {} {} {} {} {}\n",
                total_cnt,
                ucause_cnt[0], ucause_cnt[1], ucause_cnt[2],
                ucause_cnt[3], ucause_cnt[4], ucause_cnt[5]);
        }
    }

    pub fn reset_all() {
        unsafe {
            total_cnt = 0;
            for i in 0..6 {
                ucause_cnt[i] = 0;
            }
        }
    }
}
