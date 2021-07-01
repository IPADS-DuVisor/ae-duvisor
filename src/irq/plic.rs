use std::vec::Vec;
use std::sync::{Arc, Mutex, RwLock};

const MAX_DEVICES: usize = 32;

const PRIORITY_BASE: u64 = 0;
const PRIORITY_PER_ID: u64 = 4;

const ENABLE_BASE: u64 = 0x2000;
const ENABLE_PER_HART: u64 = 0x80;

const CONTEXT_BASE: u64 = 0x200000;
const CONTEXT_PER_HART: u64 = 0x1000;
const CONTEXT_THRESHOLD: u64 = 0;
const CONTEXT_CLAIM: u64 = 4;

const REG_SIZE: u64 = 0x1000000;

const PLIC_BASE_ADDR: u64 = 0xc000000;

const PRIORITY_END: u64 = ENABLE_BASE - 1;
const ENABLE_END: u64 = CONTEXT_BASE - 1;
const CONTEXT_END: u64 = REG_SIZE - 1;

struct PlicState {
    // Static configuration
    num_irq: u32,
    num_irq_word: u32,
    max_prio: u32,
    // Global IRQ state
    irq_priority: [u8; MAX_DEVICES],
    irq_level: [u32; MAX_DEVICES / 32],
}

struct PlicContext {
    // Static configuration
    ctx_id: u32,
    vcpu_id: u32,
    // Local IRQ state
    irq_priority_threshold: u8,
    irq_enable: [u32; MAX_DEVICES / 32],
    irq_pending: [u32; MAX_DEVICES / 32],
    irq_pending_priority: [u32; MAX_DEVICES],
    irq_claimed: [u32; MAX_DEVICES / 32],
    irq_autoclear: [u32; MAX_DEVICES / 32],
}

pub struct Plic {
    plic_state: RwLock<PlicState>,
    plic_contexts: RwLock<Vec<Mutex<PlicContext>>>,
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
    pub fn new(ctx_id: u32, vcpu_id: u32) -> Self {
        let irq_priority_threshold: u8 = 0;
        let irq_enable = [0; MAX_DEVICES / 32];
        let irq_pending = [0; MAX_DEVICES / 32];
        let irq_pending_priority = [0; MAX_DEVICES];
        let irq_claimed = [0; MAX_DEVICES / 32];
        let irq_autoclear = [0; MAX_DEVICES / 32];
        PlicContext {
            ctx_id,
            vcpu_id,
            irq_priority_threshold,
            irq_enable,
            irq_pending,
            irq_pending_priority,
            irq_claimed,
            irq_autoclear,
        }
    }
}

impl Plic {
    pub fn new(nr_vcpu: u32) -> Self {
        let plic_state = RwLock::new(PlicState::new());
        let nr_ctx = nr_vcpu * 2;
        let mut contexts: Vec<Mutex<PlicContext>> = 
            Vec::with_capacity(nr_ctx as usize);
        for i in 0..nr_ctx {
            let ctx_id = i;
            let vcpu_id = i / 2;
            let ctx = PlicContext::new(ctx_id, vcpu_id);
            contexts.push(Mutex::new(ctx));
        }
        let plic_contexts = RwLock::new(contexts);

        Plic {
            plic_state,
            plic_contexts,
        }
    }
    
    fn select_local_pending_irq(&self, ctx: &mut PlicContext) -> u32 {
        let mut best_irq_prio: u8 = 0;
        let (mut i, mut j, mut irq): (u32, u32, u32);
        let mut best_irq: u32 = 0;
        
        let state = self.plic_state.read().unwrap();

        for i in 0..state.num_irq_word {
            if ctx.irq_pending[i as usize] != 0 { continue; }

            for j in 0..32 {
                irq = i * 32 + j;
                if (state.num_irq <= irq) ||
                    (ctx.irq_pending[i as usize] & (1 << j)) == 0 ||
                        (ctx.irq_claimed[i as usize] & (1 << j)) != 0 {
                            continue;
                }

                if best_irq != 0 ||
                    (best_irq_prio < ctx.irq_pending_priority[irq as usize] as u8) {
                        best_irq = irq;
                        best_irq_prio = ctx.irq_pending_priority[irq as usize] as u8;
                }
            }
        }

        best_irq
    }

    fn update_local_irq(&self, ctx: &mut PlicContext) {
        let best_irq: u32 = self.select_local_pending_irq(&mut *ctx);

        if best_irq == 0 {
            // unset irq
        } else {
            // set irq
        }
    }

    fn write_global_priority(&self, offset: u64, data: u32) {
        let irq: u32 = (offset >> 2) as u32;
        if irq == 0 || irq >= self.plic_state.read().unwrap().num_irq { return; }
        
        let mut state = self.plic_state.write().unwrap();
        let val = data & ((1 << PRIORITY_PER_ID) - 1);
        state.irq_priority[irq as usize] = val as u8;
    }

    fn read_global_priority(&self, offset: u64, data: &mut u32) {
        let irq: u32 = (offset >> 2) as u32;
        if irq == 0 || irq >= self.plic_state.read().unwrap().num_irq { return; }
        
        let mut state = self.plic_state.write().unwrap();
        *data = state.irq_priority[irq as usize] as u32;
    }
    
    fn write_local_enable(&self, ctx_id: usize, offset: u64, data: u32) {
        let mut irq_prio: u8;
        let (mut i, mut irq, mut irq_mask): (u32, u32, u32);
        let irq_word: u32 = (offset >> 2) as u32;
        let (mut old_val, mut new_val, mut xor_val): (u32, u32, u32);
        
        let state = self.plic_state.read().unwrap();
        if state.num_irq_word < irq_word { return; }

        let vec = self.plic_contexts.read().unwrap();
        let mut ctx = vec[ctx_id].lock().unwrap();

        old_val = ctx.irq_enable[irq_word as usize];
        new_val = data;

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
                ctx.irq_pending[irq_word as usize] = 
                    ctx.irq_pending[irq_word as usize] | irq_mask;
                ctx.irq_pending_priority[irq as usize] = irq_prio as u32;
            } else if (new_val & irq_mask) == 0 {
                ctx.irq_pending[irq_word as usize] = 
                    ctx.irq_pending[irq_word as usize] & !irq_mask;
                ctx.irq_pending_priority[irq as usize] = 0;
                ctx.irq_claimed[irq_word as usize] = 
                    ctx.irq_claimed[irq_word as usize] & !irq_mask;
            }
        }

        self.update_local_irq(&mut *ctx);
    }
    
    fn read_local_enable(&self, ctx_id: usize, offset: u64, data: &mut u32) {
        let irq_word: u32 = (offset >> 2) as u32;
        
        let state = self.plic_state.read().unwrap();
        if state.num_irq_word < irq_word { return; }
        
        let vec = self.plic_contexts.read().unwrap();
        let mut ctx = vec[ctx_id].lock().unwrap();
        *data = ctx.irq_enable[irq_word as usize]
    }
    
    fn write_local_context(&self, ctx_id: usize, offset: u64, data: u32) {
        let mut irq_update = false;
        let vec = self.plic_contexts.read().unwrap();
        let mut ctx = vec[ctx_id].lock().unwrap();

        match offset {
            CONTEXT_THRESHOLD => {
                let val = data & ((1 << PRIORITY_PER_ID) - 1);
                let state = self.plic_state.read().unwrap();
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
            self.update_local_irq(&mut *ctx);
        }
    }
    
    fn read_local_context(&self, ctx_id: usize, offset: u64, data: &mut u32) {
        let vec = self.plic_contexts.read().unwrap();
        let mut ctx = vec[ctx_id].lock().unwrap();
        
        match offset {
            CONTEXT_THRESHOLD => {
                *data = ctx.irq_priority_threshold as u32;
            }
            CONTEXT_CLAIM => {
                let best_irq: u32 = self.select_local_pending_irq(&mut *ctx);
                let best_irq_word: u32 = best_irq / 32;
                let best_irq_mask: u32 = (1 << (best_irq % 32));

                // unset irq

                if best_irq != 0 {
                    if (ctx.irq_autoclear[best_irq_word as usize] & 
                        best_irq_mask) != 0 {
                        ctx.irq_pending[best_irq_word as usize] = 
                            ctx.irq_pending[best_irq_word as usize] & !best_irq_mask;
                        ctx.irq_pending_priority[best_irq as usize] = 0;
                        ctx.irq_claimed[best_irq_word as usize] = 
                            ctx.irq_claimed[best_irq_word as usize] & !best_irq_mask;
                        ctx.irq_autoclear[best_irq_word as usize] = 
                            ctx.irq_autoclear[best_irq_word as usize] & !best_irq_mask;
                    } else {
                        ctx.irq_claimed[best_irq_word as usize] = 
                            ctx.irq_claimed[best_irq_word as usize] | best_irq_mask;
                    }
                }
                self.update_local_irq(&mut *ctx);
                
                *data = best_irq;
            }
            _ => {}
        }
    }

    pub fn mmio_callback(&self, vcpu_id: u32, 
        addr: u64, data: &mut u32, is_write: bool) {
        let ctx_id: u64;

        let mut offset = addr & !0x3;
        offset = offset - PLIC_BASE_ADDR;

        if is_write {
            match offset {
                PRIORITY_BASE..=PRIORITY_END => {
                    self.write_global_priority(offset, *data);
                }
                ENABLE_BASE..=ENABLE_END => {
                    ctx_id = (offset - ENABLE_BASE) / ENABLE_PER_HART;
                    offset = offset - (ctx_id * ENABLE_PER_HART + ENABLE_BASE);
                    if (ctx_id as usize) < self.plic_contexts.read().unwrap().len() {
                        self.write_local_enable(ctx_id as usize, offset, *data);
                    }
                } 
                CONTEXT_BASE..=CONTEXT_END => {
                    ctx_id = (offset - CONTEXT_BASE) / CONTEXT_PER_HART;
                    offset = offset - (ctx_id * CONTEXT_PER_HART + CONTEXT_BASE);
                    if (ctx_id as usize) < self.plic_contexts.read().unwrap().len() {
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
                    if (ctx_id as usize) < self.plic_contexts.read().unwrap().len() {
                        self.read_local_enable(ctx_id as usize, offset, data);
                    }
                } 
                CONTEXT_BASE..=CONTEXT_END => {
                    ctx_id = (offset - CONTEXT_BASE) / CONTEXT_PER_HART;
                    offset = offset - (ctx_id * CONTEXT_PER_HART + CONTEXT_BASE);
                    if (ctx_id as usize) < self.plic_contexts.read().unwrap().len() {
                        self.read_local_context(ctx_id as usize, offset, data);
                    }
                }
                _ => {
                    panic!("Invalid offset: {:?}", offset)
                }
            }
        }
    }

    // Only support level-triggered IRQs
    pub fn trigger_irq(&self, irq: u32, level: u32) {
        if (irq < 0) || (self.plic_state.read().unwrap().num_irq <= irq) { return; }

        let mut state = self.plic_state.write().unwrap();

        let irq_prio: u8 = state.irq_priority[irq as usize];
        let irq_word: u8 = (irq / 32) as u8;
        let irq_mask: u32 = 1 << (irq % 32);

        if level != 0 {
            state.irq_level[irq_word as usize] = 
                state.irq_level[irq_word as usize] | irq_mask;
        } else {
            state.irq_level[irq_word as usize] = 
                state.irq_level[irq_word as usize] & !irq_mask;
        }

        let vec = self.plic_contexts.read().unwrap();
        for ctx_id in 0..vec.len() {
            let mut irq_marked: bool = false;
            let mut ctx = vec[ctx_id].lock().unwrap();
            
            if (ctx.irq_enable[irq_word as usize] & irq_mask) != 0 {
                if level != 0 {
                    ctx.irq_pending[irq_word as usize] = 
                        ctx.irq_pending[irq_word as usize] | irq_mask;
                    ctx.irq_pending_priority[irq as usize] = irq_prio as u32;
                } else {
                    ctx.irq_pending[irq_word as usize] = 
                        ctx.irq_pending[irq_word as usize] & !irq_mask;
                    ctx.irq_pending_priority[irq as usize] = 0;
                    ctx.irq_claimed[irq_word as usize] = 
                        ctx.irq_claimed[irq_word as usize] & !irq_mask;
                    ctx.irq_autoclear[irq_word as usize] = 
                        ctx.irq_autoclear[irq_word as usize] & !irq_mask;
                }
                self.update_local_irq(&mut *ctx);
                irq_marked = true;
            }

            if irq_marked { break; }
        }
    }
}
