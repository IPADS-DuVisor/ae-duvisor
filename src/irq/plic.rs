use std::vec::Vec;
use std::sync::{Arc, Mutex};

const MAX_DEVICES: usize = 32;

const PRIORITY_BASE: u64 = 0;
const PRIORITY_PER_ID: u64 = 4;

const CONTEXT_BASE: u64 = 0x200000;
const CONTEXT_PER_HART: u64 = 0x1000;
const CONTEXT_THRESHOLD: u64 = 0;
const CONTEXT_CLAIM: u64 = 4;

const REG_SIZE: u64 = 0x1000000;

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
    plic_state: Arc<Mutex<PlicState>>,
    plic_contexts: Vec<Arc<Mutex<PlicContext>>>,
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
        let plic_state = Arc::new(Mutex::new(PlicState::new()));
        let nr_ctx = nr_vcpu * 2;
        let mut plic_contexts: Vec<Arc<Mutex<PlicContext>>> = 
            Vec::with_capacity(nr_ctx as usize);
        for i in 0..nr_ctx {
            let ctx_id = i;
            let vcpu_id = i / 2;
            let ctx = PlicContext::new(ctx_id, vcpu_id);
            plic_contexts.push(Arc::new(Mutex::new(ctx)));
        }

        Plic {
            plic_state,
            plic_contexts,
        }
    }
}
