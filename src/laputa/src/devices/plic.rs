use std::vec::Vec;
use std::sync::{Arc, Weak, Mutex, RwLock};
use std::sync::atomic::Ordering;
use crate::mm::utils::dbgprintln;
use crate::vcpu::virtualcpu::VirtualCpu;
use crate::irq::vipi::VirtualIpi;
use crate::irq::delegation::delegation_constants::*;

extern crate irq_util;
use irq_util::IrqChip;
use irq_util::SharedStat;
extern crate spin;
use spin::mutex::TicketMutex;

const MAX_DEVICES: usize = 32;

const PLIC_BASE_ADDR: u64 = 0xc000000;

const PRIORITY_BASE: u64 = 0;
const PRIORITY_PER_ID: u64 = 4;

const ENABLE_BASE: u64 = 0x2000;
const ENABLE_PER_HART: u64 = 0x80;

const CONTEXT_BASE: u64 = 0x200000;
const CONTEXT_PER_HART: u64 = 0x1000;
const CONTEXT_THRESHOLD: u64 = 0;
const CONTEXT_CLAIM: u64 = 4;

const PRIORITY_END: u64 = ENABLE_BASE - 1;
const ENABLE_END: u64 = CONTEXT_BASE - 1;
const CONTEXT_END: u64 = REG_SIZE - 1;

const REG_SIZE: u64 = 0x1000000;

static mut irq_set_time: usize = 0;

struct PlicState {
    /* Static configuration */
    num_irq: u32,
    num_irq_word: u32,
    max_prio: u32,
    /* Global IRQ state */
    irq_priority: [u8; MAX_DEVICES],
    irq_level: [u32; MAX_DEVICES / 32],
}

struct PlicContext {
    /* Static configuration */
    vcpu: Weak<VirtualCpu>,
    /* Local IRQ state */
    irq_priority_threshold: u8,
    irq_enable: [u32; MAX_DEVICES / 32],
    irq_pending: [u32; MAX_DEVICES / 32],
    irq_pending_priority: [u32; MAX_DEVICES],
    irq_claimed: [u32; MAX_DEVICES / 32],
    irq_autoclear: [u32; MAX_DEVICES / 32],
}

pub struct Plic {
    plic_state: TicketMutex<PlicState>,
    plic_contexts: Vec<TicketMutex<PlicContext>>,
}

impl PlicState {
    pub fn new() -> Self {
        let num_irq = MAX_DEVICES as u32;
        let mut num_irq_word = num_irq / 32 as u32;
        if num_irq_word * 32 < num_irq {
            num_irq_word = num_irq_word + 1;
        }
        let max_prio = (1 << PRIORITY_PER_ID) - 1;
        let irq_priority = [0; MAX_DEVICES];
        let irq_level = [0; MAX_DEVICES / 32];
        
        PlicState {
            num_irq,
            num_irq_word,
            max_prio,
            irq_priority,
            irq_level,
        }
    }
}

impl PlicContext {
    pub fn new(vcpu: Weak<VirtualCpu>) -> Self {
        let irq_priority_threshold: u8 = 0;
        let irq_enable = [0; MAX_DEVICES / 32];
        let irq_pending = [0; MAX_DEVICES / 32];
        let irq_pending_priority = [0; MAX_DEVICES];
        let irq_claimed = [0; MAX_DEVICES / 32];
        let irq_autoclear = [0; MAX_DEVICES / 32];
        
        PlicContext {
            vcpu,
            irq_priority_threshold,
            irq_enable,
            irq_pending,
            irq_pending_priority,
            irq_claimed,
            irq_autoclear,
        }
    }
}

static mut PLIC_TIME_START: usize = 0;
static mut PLIC_TIME: usize = 0;
static mut PLIC_CNT: usize = 0;
static mut PLIC_TIME_TOTAL: [usize; 8] = [0; 8];
static mut PLIC_CNT_TOTAL: [usize; 8] = [0; 8];

impl Plic {
    pub fn new(vcpus: &Vec<Arc<VirtualCpu>>) -> Self {
        let plic_state = TicketMutex::<_>::new(PlicState::new());
        let nr_ctx = vcpus.len() * 2;
        let mut plic_contexts: Vec<TicketMutex<PlicContext>> = 
            Vec::with_capacity(nr_ctx as usize);
        for i in 0..nr_ctx {
            let vcpu = Arc::downgrade(&vcpus[i / 2]);
            let ctx = PlicContext::new(vcpu);
            plic_contexts.push(TicketMutex::new(ctx));
        }

        Plic {
            plic_state,
            plic_contexts,
        }
    }
    
    fn select_local_pending_irq(&self, ctx: &mut PlicContext, state: &PlicState) -> u32 {
        let mut best_irq_prio: u8 = 0;
        let mut irq: u32;
        let mut best_irq: u32 = 0;
        
        for i in 0..state.num_irq_word {
            if ctx.irq_pending[i as usize] == 0 { continue; }

            for j in 0..32 {
                irq = i * 32 + j;
                if (state.num_irq <= irq) ||
                    (ctx.irq_pending[i as usize] & (1 << j)) == 0 ||
                        (ctx.irq_claimed[i as usize] & (1 << j)) != 0 {
                            continue;
                }

                if (best_irq == 0 && (ctx.irq_pending_priority[irq as usize] > 0)) ||
                    (best_irq_prio < ctx.irq_pending_priority[irq as usize] as u8) {
                        best_irq = irq;
                        best_irq_prio = ctx.irq_pending_priority[irq as usize] as u8;
                }
                dbgprintln!("selecting irq: {} {}, best_irq_prio: {:x}, prio: {:x}", 
                    irq, best_irq, best_irq_prio, ctx.irq_pending_priority[irq as usize]);
            }
        }

        best_irq
    }

    fn update_local_irq(&self, ctx: &mut PlicContext, state: &PlicState) {
        let best_irq: u32 = self.select_local_pending_irq(ctx, state);
        dbgprintln!("update_local_irq best_irq: {}", best_irq);

        let vcpu = ctx.vcpu.upgrade().unwrap();
        if best_irq == 0 {
            /* Unset irq */
            vcpu.virq.unset_pending_irq(IRQ_VS_EXT);
        } else {
            /* Set irq */
            vcpu.virq.set_pending_irq(IRQ_VS_EXT);
            
            let vcpu_state = vcpu.is_running.load(Ordering::SeqCst);
            if vcpu_state == 1 {
                let vipi_id = vcpu.vipi.vcpu_id_map[vcpu.vcpu_id as usize]
                    .load(Ordering::SeqCst);

                VirtualIpi::set_vipi(vipi_id);
            //} else if vcpu_state == 2 {
            } else {
                let vcpu_id = vcpu.vcpu_id as usize;
                let mut guard = vcpu.vm.wfi_mutex[vcpu_id].lock().unwrap();
                *guard = true;
                vcpu.vm.wfi_cv[vcpu_id].notify_one();
            }
        }
    }

    fn write_global_priority(&self, offset: u64, data: u32) {
        let irq: u32 = (offset >> 2) as u32;
        let mut state = self.plic_state.lock();
        if irq == 0 || irq >= state.num_irq { return; }

        let val = data & ((1 << PRIORITY_PER_ID) - 1);
        state.irq_priority[irq as usize] = val as u8;
    }

    fn read_global_priority(&self, offset: u64, data: &mut u32) {
        let irq: u32 = (offset >> 2) as u32;
        let state = self.plic_state.lock();
        if irq == 0 || irq >= state.num_irq { return; }

        *data = state.irq_priority[irq as usize] as u32;
    }
    
    fn write_local_enable(&self, ctx_id: usize, offset: u64, data: u32) {
        let mut irq_prio: u8;
        let (mut irq, mut irq_mask): (u32, u32);
        let irq_word: u32 = (offset >> 2) as u32;
        
        let state = self.plic_state.lock();
        if state.num_irq_word < irq_word { return; }

        let mut ctx = self.plic_contexts[ctx_id].lock();
        let (old_val, mut new_val, xor_val): (u32, u32, u32);
        old_val = ctx.irq_enable[irq_word as usize];
        new_val = data;

        /* 
         * Bit 0 of word 0, which represents the non-existent interrupt source 0, 
         * is hardwired to zero.
         */
        if irq_word == 0 {
            new_val = new_val & !0x1;
        }

        ctx.irq_enable[irq_word as usize] = new_val;

        xor_val = old_val ^ new_val;
        for i in 0..32 {
            irq = irq_word * 32 + i;
            irq_mask = 1 << i;
            irq_prio = state.irq_priority[irq as usize];
            if (xor_val & irq_mask) == 0 {
                continue;
            }
            if (new_val & irq_mask) != 0 && 
                (state.irq_level[irq_word as usize] & irq_mask) != 0 {
                ctx.irq_pending[irq_word as usize] |= irq_mask;
                ctx.irq_pending_priority[irq as usize] = irq_prio as u32;
            } else if (new_val & irq_mask) == 0 {
                ctx.irq_pending[irq_word as usize] &= !irq_mask;
                ctx.irq_pending_priority[irq as usize] = 0;
                ctx.irq_claimed[irq_word as usize] &= !irq_mask;
            }
        }

        self.update_local_irq(&mut *ctx, &*state);
    }
    
    fn read_local_enable(&self, ctx_id: usize, offset: u64, data: &mut u32) {
        let irq_word: u32 = (offset >> 2) as u32;
        
        let state = self.plic_state.lock();
        if state.num_irq_word < irq_word { return; }
        
        let ctx = self.plic_contexts[ctx_id].lock();
        *data = ctx.irq_enable[irq_word as usize]
    }
    
    fn write_local_context(&self, ctx_id: usize, offset: u64, data: u32) {
        let mut irq_update = false;
        let state = self.plic_state.lock();
        let mut ctx = self.plic_contexts[ctx_id].lock();

        match offset {
            CONTEXT_THRESHOLD => {
                let val = data & ((1 << PRIORITY_PER_ID) - 1);
                if val <= state.max_prio {
                    ctx.irq_priority_threshold = val as u8;
                } else {
                    irq_update = true;
                }
            }
            CONTEXT_CLAIM => {}
            _ => { irq_update = true; }
        }

        if irq_update {
            self.update_local_irq(&mut *ctx, &*state);
        }
    }
    
    fn read_local_context(&self, ctx_id: usize, offset: u64, data: &mut u32) {
        let state = self.plic_state.lock();
        let mut ctx = self.plic_contexts[ctx_id].lock();
        
        match offset {
            CONTEXT_THRESHOLD => {
                *data = ctx.irq_priority_threshold as u32;
            }
            CONTEXT_CLAIM => {
                let best_irq: u32 = self.select_local_pending_irq(&mut *ctx, &*state);
                let best_irq_word: u32 = best_irq / 32;
                let best_irq_mask: u32 = 1 << (best_irq % 32);

                /* Unset irq */
                let vcpu = ctx.vcpu.upgrade().unwrap();
                vcpu.virq.unset_pending_irq(IRQ_VS_EXT);

                if best_irq != 0 {
                    if (ctx.irq_autoclear[best_irq_word as usize] & 
                        best_irq_mask) != 0 {
                        ctx.irq_pending[best_irq_word as usize] &= !best_irq_mask;
                        ctx.irq_pending_priority[best_irq as usize] = 0;
                        ctx.irq_claimed[best_irq_word as usize] &= !best_irq_mask;
                        ctx.irq_autoclear[best_irq_word as usize] &= !best_irq_mask;
                    } else {
                        ctx.irq_claimed[best_irq_word as usize] |= best_irq_mask;
                    }
                }
                self.update_local_irq(&mut *ctx, &*state);
                
                *data = best_irq;
            }
            _ => {}
        }
    }
    
    fn plic_trigger_irq(&self, irq: u32, level: bool, edge: bool) {
        let mut time_start: usize = 0;
        unsafe {
            if irq == 3 {
                asm!("csrr {}, 0xC01", out(reg) time_start);
                PLIC_TIME_START = time_start;
            }
        }
        let mut state = self.plic_state.lock();
        dbgprintln!("trigger_irq: irq {} num_irq {} level {}",
            irq, state.num_irq, level);
        unsafe {
            if irq == 3 {
                let cur_time: usize;
                asm!("csrr {}, 0xC01", out(reg) cur_time);
                PLIC_TIME_TOTAL[0] += cur_time - time_start;
                PLIC_CNT_TOTAL[0] += 1;
                time_start = cur_time;
            }
        }
        if state.num_irq <= irq { return; }

        let irq_prio: u8 = state.irq_priority[irq as usize];
        let irq_word: u8 = (irq / 32) as u8;
        let irq_mask: u32 = 1 << (irq % 32);

        if level {
            state.irq_level[irq_word as usize] |= irq_mask;
        } else {
            state.irq_level[irq_word as usize] &= !irq_mask;
        }
        dbgprintln!("\t\ttrigger_irq: irq_prio {:x} irq_word {:x} irq_mask {:x}",
            irq_prio, irq_word, irq_mask);

        unsafe {
            if irq == 3 {
                let cur_time: usize;
                asm!("csrr {}, 0xC01", out(reg) cur_time);
                PLIC_TIME_TOTAL[1] += cur_time - time_start;
                PLIC_CNT_TOTAL[1] += 1;
                time_start = cur_time;
            } 
        }
        for i in 0..self.plic_contexts.len() {
            unsafe {
                if irq == 3 {
                    let cur_time: usize;
                    asm!("csrr {}, 0xC01", out(reg) cur_time);
                    PLIC_TIME_TOTAL[2] += cur_time - time_start;
                    PLIC_CNT_TOTAL[2] += 1;
                    time_start = cur_time;
                }
            }
            let mut irq_marked: bool = false;
            let mut ctx = self.plic_contexts[i].lock();
            unsafe {
                if irq == 3 {
                    let cur_time: usize;
                    asm!("csrr {}, 0xC01", out(reg) cur_time);
                    PLIC_TIME_TOTAL[3] += cur_time - time_start;
                    PLIC_CNT_TOTAL[3] += 1;
                    time_start = cur_time;
                }
            }

            if (ctx.irq_enable[irq_word as usize] & irq_mask) != 0 {
                if level {
                    ctx.irq_pending[irq_word as usize] |= irq_mask;
                    ctx.irq_pending_priority[irq as usize] = irq_prio as u32;
                    if edge {
                        ctx.irq_autoclear[irq_word as usize] |= irq_mask;
                    }
                    dbgprintln!("\t\ttrigger_irq irq_pending: {:x}, irq_mask: {:x}", 
                        ctx.irq_pending[irq_word as usize], irq_mask);
                } else {
                    ctx.irq_pending[irq_word as usize] &= !irq_mask;
                    ctx.irq_pending_priority[irq as usize] = 0;
                    ctx.irq_claimed[irq_word as usize] &= !irq_mask;
                    ctx.irq_autoclear[irq_word as usize] &= !irq_mask;
                }
                unsafe {
                    if irq == 3 {
                        let cur_time: usize;
                        asm!("csrr {}, 0xC01", out(reg) cur_time);
                        PLIC_TIME_TOTAL[4] += cur_time - time_start;
                        PLIC_CNT_TOTAL[4] += 1;
                        time_start = cur_time;
                        
                        irq_set_time = cur_time;
                    }
                }
                self.update_local_irq(&mut *ctx, &*state);
                unsafe {
                    if irq == 3 {
                        let cur_time: usize;
                        asm!("csrr {}, 0xC01", out(reg) cur_time);
                        PLIC_TIME_TOTAL[5] += cur_time - time_start;
                        PLIC_CNT_TOTAL[5] += 1;
                        time_start = cur_time;
                    }
                }
                irq_marked = true;
            }
            dbgprintln!("\t\ttrigger_irq: i {} irq_enable {:x} irq_marked {}",
                i, ctx.irq_enable[irq_word as usize], irq_marked);

            if irq_marked { break; }
        }
        unsafe {
            if irq == 3 {
                let cur_time: usize;
                asm!("csrr {}, 0xC01", out(reg) cur_time);
                PLIC_TIME_TOTAL[6] += cur_time - time_start;
                PLIC_CNT_TOTAL[6] += 1;
                time_start = cur_time;
                PLIC_TIME += cur_time - PLIC_TIME_START;
                //PLIC_CNT += 1;
                if PLIC_CNT >= 100000 {
                    println!("--- PLIC_CNT_TOTAL {}, PLIC_TIME_TOTAL {}, avg {}\n \
                    plic_time {} {} {} {}\n \
                    \t\t {} {} {} {}\n \
                    plic_cnt {} {} {} {}\n \
                    \t\t {} {} {} {}",
                    PLIC_CNT, PLIC_TIME, PLIC_CNT / PLIC_TIME,
                    PLIC_TIME_TOTAL[0], PLIC_TIME_TOTAL[1], PLIC_TIME_TOTAL[2], PLIC_TIME_TOTAL[3],
                    PLIC_TIME_TOTAL[4], PLIC_TIME_TOTAL[5], PLIC_TIME_TOTAL[6], PLIC_TIME_TOTAL[7],
                    PLIC_CNT_TOTAL[0], PLIC_CNT_TOTAL[1], PLIC_CNT_TOTAL[2], PLIC_CNT_TOTAL[3],
                    PLIC_CNT_TOTAL[4], PLIC_CNT_TOTAL[5], PLIC_CNT_TOTAL[6], PLIC_CNT_TOTAL[7]);
                    PLIC_TIME = 0;
                    PLIC_CNT = 0;
                    for i in 0..8 {
                        PLIC_TIME_TOTAL[i] = 0;
                        PLIC_CNT_TOTAL[i] = 0;
                    }
                }
            }
        }
    }
}

impl IrqChip for Plic {
    fn mmio_callback(&self, addr: u64, data: &mut u32, is_write: bool) {
        let ctx_id: u64;

        let mut offset = addr & !0x3;
        offset = offset - PLIC_BASE_ADDR;

        if is_write {
            match offset {
                PRIORITY_BASE..=PRIORITY_END => {
                    dbgprintln!("write_global_priority offset {:x}, data {:x}", 
                        offset, *data);
                    self.write_global_priority(offset, *data);
                }
                ENABLE_BASE..=ENABLE_END => {
                    ctx_id = (offset - ENABLE_BASE) / ENABLE_PER_HART;
                    offset = offset - (ctx_id * ENABLE_PER_HART + ENABLE_BASE);
                    if (ctx_id as usize) < self.plic_contexts.len() {
                        dbgprintln!("write_local_enable ctx_id {} offset {:x}, data {:x}", 
                            ctx_id, offset, *data);
                        self.write_local_enable(ctx_id as usize, offset, *data);
                    }
                } 
                CONTEXT_BASE..=CONTEXT_END => {
                    ctx_id = (offset - CONTEXT_BASE) / CONTEXT_PER_HART;
                    offset = offset - (ctx_id * CONTEXT_PER_HART + CONTEXT_BASE);
                    if (ctx_id as usize) < self.plic_contexts.len() {
                        dbgprintln!("write_local_context ctx_id {} offset {:x}, data {:x}", 
                            ctx_id, offset, *data);
                        self.write_local_context(ctx_id as usize, offset, *data);
                    }
                }
                _ => {
                    panic!("Invalid offset: {:?}", offset)
                }
            }
        } else {
            match offset {
                PRIORITY_BASE..=PRIORITY_END => {
                    self.read_global_priority(offset, data);
                }
                ENABLE_BASE..=ENABLE_END => {
                    ctx_id = (offset - ENABLE_BASE) / ENABLE_PER_HART;
                    offset = offset - (ctx_id * ENABLE_PER_HART + ENABLE_BASE);
                    if (ctx_id as usize) < self.plic_contexts.len() {
                        self.read_local_enable(ctx_id as usize, offset, data);
                    }
                } 
                CONTEXT_BASE..=CONTEXT_END => {
                    ctx_id = (offset - CONTEXT_BASE) / CONTEXT_PER_HART;
                    offset = offset - (ctx_id * CONTEXT_PER_HART + CONTEXT_BASE);
                    if (ctx_id as usize) < self.plic_contexts.len() {
                        self.read_local_context(ctx_id as usize, offset, data);
                        if offset == CONTEXT_CLAIM && *data == 3 {
                            unsafe {
                                let cur_time: usize;
                                asm!("csrr {}, 0xC01", out(reg) cur_time);
                                SharedStat::add_irq_resp_time(cur_time - irq_set_time);
                            }
                        }
                    }
                }
                _ => {
                    panic!("Invalid offset: {:?}", offset)
                }
            }
        }
    }

    /* Only support level-triggered IRQs */
    fn trigger_level_irq(&self, irq: u32, level: bool) {
        self.plic_trigger_irq(irq, level, false);
    }
    
    fn trigger_edge_irq(&self, irq: u32) {
        self.plic_trigger_irq(irq, true, true);
    }

    fn trigger_virtual_irq(&self, vcpu_id: u32) -> u16 {
        let ctx_id = (vcpu_id * 2) as usize;
        let vcpu = self.plic_contexts[ctx_id].lock()
            .vcpu.upgrade().unwrap();
        vcpu.virq.set_pending_irq(IRQ_VS_SOFT);

        vcpu.is_running.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use rusty_fork::rusty_fork_test;
    use crate::vm::*;
    use crate::test::utils::configtest::test_vm_config_create;
    use crate::devices::plic::*;
    use std::thread;

    rusty_fork_test! {
        #[test]
        fn test_plic_init() {
            let mut vm_config = test_vm_config_create();
            vm_config.vcpu_count = 2;
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let _plic = Plic::new(&vm.vcpus);
        }
        
        #[test]
        fn test_plic_local_enable() {
            let mut vm_config = test_vm_config_create();
            vm_config.vcpu_count = 2;
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let plic = Arc::new(Plic::new(&vm.vcpus));
        
            let get_enable_offset = |ctx_id: u64, offset: u64| -> u64 {
                PLIC_BASE_ADDR + ENABLE_BASE + ctx_id * ENABLE_PER_HART + offset
            };
            let local_enable_succeed = 
                |mut write: u32, mut read: u32, ctx_id: u64, offset: u64| {
                    plic.mmio_callback(get_enable_offset(ctx_id, offset), 
                        &mut write, true);
                    plic.mmio_callback(get_enable_offset(ctx_id, offset), 
                        &mut read, false);
                    /* IRQ #0 is hardwired to 0 */
                    assert_eq!(read, write & !0x1);
            };
            local_enable_succeed(0xff, 0xdead, 0, 0);
            local_enable_succeed(0xf, 0xdead, 1, 0);
            local_enable_succeed(0xff, 0xdead, 2, 0);
            local_enable_succeed(0xf, 0xdead, 3, 0);
            
            let local_enable_failed = 
                |mut write: u32, mut read: u32, ctx_id: u64, offset: u64| {
                    plic.mmio_callback(get_enable_offset(ctx_id, offset), 
                        &mut write, true);
                    plic.mmio_callback(get_enable_offset(ctx_id, offset), 
                        &mut read, false);
                    /* Only 32 IRQs supported, write to offset > 0x8 is ignored */
                    assert_eq!(read, 0xdead);
            };
            local_enable_failed(0xff, 0xdead, 0, 0x8);
            local_enable_failed(0xf, 0xdead, 1, 0x8);
            local_enable_failed(0xff, 0xdead, 2, 0x8);
            local_enable_failed(0xf, 0xdead, 3, 0x8);
        }
        
        #[test]
        fn test_plic_local_context() {
            let mut vm_config = test_vm_config_create();
            vm_config.vcpu_count = 2;
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let plic = Arc::new(Plic::new(&vm.vcpus));
        
            let get_threshold_offset = |ctx_id: u64| -> u64 {
                PLIC_BASE_ADDR + CONTEXT_BASE + 
                    ctx_id * CONTEXT_PER_HART + CONTEXT_THRESHOLD
            };
            let get_claim_offset = |ctx_id: u64| -> u64 {
                PLIC_BASE_ADDR + CONTEXT_BASE + 
                    ctx_id * CONTEXT_PER_HART + CONTEXT_CLAIM
            };
            let local_context_succeed = 
                |mut write: u32, mut read: u32, ctx_id: u64| {
                    plic.mmio_callback(get_threshold_offset(ctx_id), &mut write, true);
                    plic.mmio_callback(get_threshold_offset(ctx_id), &mut read, false);
                    assert_eq!(read, write & ((1 << PRIORITY_PER_ID) - 1));
                    
                    plic.mmio_callback(get_claim_offset(ctx_id), &mut write, true);
                    plic.mmio_callback(get_claim_offset(ctx_id), &mut read, false);
                    /* Write to CLAIM is ignored */
                    assert_eq!(read, 0);
            };
            local_context_succeed(0xff, 0, 0);
            local_context_succeed(0, 0, 1);
            local_context_succeed(0x7, 0, 2);
            local_context_succeed(0xf, 0, 3);
            
            let get_global_prio_offset = |irq: u32| -> u64 {
                PLIC_BASE_ADDR + PRIORITY_BASE + (irq as u64) * PRIORITY_PER_ID
            };
            let get_enable_offset = |ctx_id: u64, offset: u64| -> u64 {
                PLIC_BASE_ADDR + ENABLE_BASE + ctx_id * ENABLE_PER_HART + offset
            };
            let local_claim_succeed = 
                |irq: u32, mut read: u32, ctx_id: u64| {
                    /* Init global priority & local enable */
                    let mut mask = 0xffffffff;
                    plic.mmio_callback(get_global_prio_offset(irq), &mut mask, true);
                    plic.mmio_callback(get_enable_offset(ctx_id, 0), &mut mask, true);

                    plic.trigger_level_irq(irq, true);
                    plic.mmio_callback(get_claim_offset(ctx_id), &mut read, false);
                    plic.trigger_level_irq(irq, false);
                    assert_eq!(read, irq);
            };
            local_claim_succeed(1, 0xdead, 0);
            local_claim_succeed(31, 0xdead, 0);
            
            let local_claim_failed = 
                |irq: u32, mut read: u32, ctx_id: u64| {
                    /* Init global priority & local enable */
                    let mut mask = 0xffffffff;
                    plic.mmio_callback(get_global_prio_offset(irq), &mut mask, true);
                    plic.mmio_callback(get_enable_offset(ctx_id, 0), &mut mask, true);

                    /* Set global priority to 0, so no IRQ will be selected */
                    mask = 0;
                    plic.mmio_callback(get_global_prio_offset(irq), &mut mask, true);
                    
                    plic.trigger_level_irq(irq, true);
                    plic.mmio_callback(get_claim_offset(ctx_id), &mut read, false);
                    plic.trigger_level_irq(irq, false);
                    assert_eq!(read, 0);
                    
                    /* Set local enable to 0, so no IRQ will be selected */
                    mask = 0;
                    plic.mmio_callback(get_enable_offset(ctx_id, 0), &mut mask, true);
                    
                    plic.trigger_level_irq(irq, true);
                    plic.mmio_callback(get_claim_offset(ctx_id), &mut read, false);
                    plic.trigger_level_irq(irq, false);
                    assert_eq!(read, 0);
                    
                    /* Out-of-range IRQ */
                    plic.trigger_level_irq(32, true);
                    plic.mmio_callback(get_claim_offset(ctx_id), &mut read, false);
                    plic.trigger_level_irq(32, false);
                    assert_eq!(read, 0);
            };
            local_claim_failed(1, 0xdead, 0);
            local_claim_failed(31, 0xdead, 0);
        }
        
        #[test]
        fn test_plic_multithlock() {
            let mut vm_config = test_vm_config_create();
            vm_config.vcpu_count = 2;
            let vm = virtualmachine::VirtualMachine::new(vm_config);
            let plic = Arc::new(Plic::new(&vm.vcpus));
            
            let thread_test = |plic: &Arc<Plic>, irq: u32| {
                let get_global_prio_offset = |irq: u32| -> u64 {
                    PLIC_BASE_ADDR + PRIORITY_BASE + (irq as u64) * PRIORITY_PER_ID
                };
                let get_enable_offset = |ctx_id: u64, offset: u64| -> u64 {
                    PLIC_BASE_ADDR + ENABLE_BASE + ctx_id * ENABLE_PER_HART + offset
                };
                let get_claim_offset = |ctx_id: u64| -> u64 {
                    PLIC_BASE_ADDR + CONTEXT_BASE + 
                        ctx_id * CONTEXT_PER_HART + CONTEXT_CLAIM
                };
                let local_claim_succeed = 
                    |irq: u32, mut read: u32, ctx_id: u64| {
                        /* Init global priority & local enable */
                        let mut mask = 0xffffffff;
                        plic.mmio_callback(get_global_prio_offset(irq), &mut mask, true);
                        plic.mmio_callback(get_enable_offset(ctx_id, 0), &mut mask, true);

                        plic.trigger_level_irq(irq, true);
                        plic.mmio_callback(get_claim_offset(ctx_id), &mut read, false);
                        plic.trigger_level_irq(irq, false);
                        assert_eq!(read, irq);
                    };
                local_claim_succeed(irq, 0xdead, 0);
            };

            let p1 = plic.clone();
            let handle1 = thread::spawn(move || {
                for irq in 1..16 {
                    thread_test(&p1, irq);
                }
            });
            
            let p2 = plic.clone();
            let handle2 = thread::spawn(move || {
                for irq in 16..32 {
                    thread_test(&p2, irq);
                }
            });

            handle1.join().ok();
            handle2.join().ok();
        }
    }
}
