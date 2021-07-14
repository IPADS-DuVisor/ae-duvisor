use std::sync::Arc;

use tty_uart_constants::*;
use crate::mm::utils::*;
use crate::irq::irqchip::IrqChip;

#[allow(unused)]
pub mod tty_uart_constants {
    /* UART_RX */
    pub const UART_RX: usize = 0;

    /* UART_TX */
    pub const UART_TX: usize = 0;

    /* UART_IER */
    pub const UART_IER: usize = 1;
    pub const UART_IER_RDI: u8 = 0x1;
    pub const UART_IER_THRI: u8 = 0x02;

    /* UART_IIR */
    pub const UART_IIR: usize = 2;
    pub const UART_IIR_NO_INT: u8 = 0x1;
    pub const UART_IIR_RDI: u8 = 0x4;
    pub const UART_IIR_THRI: u8 = 0x2;
    pub const UART_IIR_TYPE_BITS: u8 = 0xc0;

    /* UART_LCR */
    pub const UART_LCR: usize = 3;
    pub const UART_LCR_DLAB: u8 = 0x80;

    /* UART_MCR */
    pub const UART_MCR: usize = 4;
    pub const UART_MCR_OUT2: u8 = 0x08;
    pub const UART_MCR_LOOP: u8 = 0x10;

    /* UART_LSR */
    pub const UART_LSR: usize = 5;
    pub const UART_LSR_TEMT: u8 = 0x40;
    pub const UART_LSR_THRE: u8 = 0x20;
    pub const UART_LSR_DR: u8 = 0x1;

    /* UART_MSR */
    pub const UART_MSR: usize = 6;
    pub const UART_MSR_DCD: u8 = 0x80;
    pub const UART_MSR_DSR: u8 = 0x20;
    pub const UART_MSR_CTS: u8 = 0x10;

    /* UART_SCR */
    pub const UART_SCR: usize = 7;

    /* UART_DLL */
    pub const UART_DLL: usize = 8; /* Reuse offset 0 */

    /* UART_DLM */
    pub const UART_DLM: usize = 9; /* Reuse offset 1 */

    /* UART_FCR */
    pub const UART_FCR: usize = 10; /* Reuse offset 2 */
    pub const UART_FCR_CLEAR_RCVR: u8 = 0x02;
    pub const UART_FCR_CLEAR_XMIT: u8 = 0x04;
}

pub const FIFO_LEN: usize = 64;

pub struct Tty {
    pub value: [u8; 11],
    pub recv_buf: [char; FIFO_LEN],
    pub recv_buf_head: usize,
    pub recv_buf_tail: usize,
    pub avail_char: usize,
    pub irq_state: u8,
}

fn console_putchar(output: u64) {
    let ch = output as u8;
    let ch = ch as char;

    const ESCAPE_LEN: usize = 20;
    static mut ESCAPE: [char; ESCAPE_LEN] = ['\0'; ESCAPE_LEN];
    static mut ESCAPE_CNT: usize = 0;

    unsafe {
        /* The first letter must be ESCAPE */
        if ESCAPE_CNT == 0 && output == 27 {
            ESCAPE[0] = ch;
            ESCAPE_CNT += 1;
        } else if ESCAPE_CNT == 1 && ch == '[' {
            ESCAPE[1] = ch;
            ESCAPE_CNT += 1;
        } else if ESCAPE_CNT > 1 && ESCAPE_CNT < ESCAPE_LEN && output != 13 { 
            ESCAPE[ESCAPE_CNT] = ch;
            ESCAPE_CNT += 1;
        } else if ESCAPE_CNT > 1 && ESCAPE_CNT < ESCAPE_LEN && output == 13 {
            /* Match the pattern and throw out */
            ESCAPE_CNT = 0;
        } else {            
            for i in 0..ESCAPE_CNT {
                print!("{}", ESCAPE[i]);
            }
            print!("{}", ch);
            ESCAPE_CNT = 0;
        }
    }
}

impl Tty {
    pub fn new() -> Self {
        let mut value: [u8; 11] = [0; 11];

        /* TtyS0 init */
        value[UART_IIR] = UART_IIR_NO_INT;
        value[UART_MCR] = UART_MCR_OUT2;
        value[UART_LSR] = UART_LSR_TEMT | UART_LSR_THRE;
        value[UART_MSR] = UART_MSR_DCD | UART_MSR_DSR | UART_MSR_CTS;
        let recv_buf: [char; FIFO_LEN] = [0 as char; FIFO_LEN];
        let recv_buf_head: usize = FIFO_LEN;
        let recv_buf_tail: usize = 0;
        let avail_char: usize = 0;
        let irq_state: u8 = 0;

        Self {
            value,
            recv_buf,
            avail_char,
            recv_buf_head,
            recv_buf_tail,
            irq_state,
        }
    }

    pub fn trigger_irq(&mut self, irqchip: &Arc<dyn IrqChip>) {
        if self.avail_char > 0 {
            irqchip.trigger_irq(1, true);
        } else {
            irqchip.trigger_irq(1, false);
        }
    }

    pub fn update_irq(&mut self, irqchip: &Arc<dyn IrqChip>) {
        let mut iir: u8 = 0;

        /* Handle clear rx */
        if self.value[UART_LCR] & UART_FCR_CLEAR_RCVR != 0 {
            self.value[UART_LCR] &= !UART_FCR_CLEAR_RCVR;
            self.value[UART_LSR] &= !UART_LSR_DR;
        }

        /* Handle clear tx */
        if self.value[UART_LCR] & UART_FCR_CLEAR_XMIT != 0 {
            self.value[UART_LCR] &= !UART_FCR_CLEAR_XMIT;
            self.value[UART_LSR] |= UART_LSR_TEMT | UART_LSR_THRE;
        }

        /* Data ready and rcv interrupt enabled ? */
        if (self.value[UART_IER] & UART_IER_RDI != 0) && (self.value[UART_LSR] & UART_LSR_DR != 0) {
            iir |= UART_IIR_RDI;
        }

        /* Transmitter empty and interrupt enabled ? */
        if (self.value[UART_IER] & UART_IER_THRI != 0) && (self.value[UART_LSR] & UART_LSR_TEMT != 0) {
            iir |= UART_IIR_THRI;
        }

        /* Now update the irq line, if necessary */
        /* TODO: different from kvmtool, fix 8250 in the future */
        if iir != 0 {
            self.value[UART_IIR] = iir;

            if self.irq_state == 0 {
                irqchip.trigger_irq(1, false); 
            }
        } else {
            //if self.value[UART_IIR] != UART_IIR_NO_INT ||  self.value[UART_IIR] != iir {
            if self.value[UART_IIR] != iir {
                //println!("CATCH ! -- UART_IIR = 0x{:x}, iir = 0x{:x}", self.value[UART_IIR], iir);
            }

            self.value[UART_IIR] = iir;

            /* Debug */
            self.value[UART_IIR] = UART_IIR_NO_INT;

            if self.irq_state != 0 {
                irqchip.trigger_irq(1, true);
            }
        }

        //print!("-");

        self.irq_state = iir;

        /*
         * If the kernel disabled the tx interrupt, we know that there
         * is nothing more to transmit, so we can reset our tx logic
         * here.
         */
        if self.value[UART_IER] & UART_IER_THRI == 0 {
            self.flush_tx();
        }
    }

    pub fn load_emulation(&mut self, mmio_addr: u64, 
        irqchip: &Arc<dyn IrqChip>) -> u8 {
        let offset = mmio_addr - 0x3f8;
        let mut ret: u8 = 0 as u8;

        match offset as usize {
            UART_RX => { /* 0x3f8 */
                if self.value[UART_LCR] & UART_LCR_DLAB != 0 {
                    ret = self.value[UART_DLL];
                } else {
                    /* Get input */
                    let res = self.get_char();

                    if res.is_some() {
                        ret = res.unwrap() as u8;
                    } else {
                        dbgprintln!("mmio fault: get_char failed");
                    }
                }
            }
            UART_IER => { /* 0x3f9 */
                if self.value[UART_LCR] & UART_LCR_DLAB == 0 {
                    ret = self.value[UART_DLM];
                } else {
                    ret = self.value[UART_IER];
                }
            }
            UART_IIR => { /* 0x3fa */
                ret = self.value[UART_IIR] | UART_IIR_TYPE_BITS;
            }
            UART_LCR => { /* 0x3fb */
                ret = self.value[UART_LCR];
            }
            UART_MCR => { /* 0x3fc */
                ret = self.value[UART_MCR];
            }
            UART_LSR => { /* 0x3fd */
                ret = self.value[UART_LSR];
                /* if self.avail_char > 0 {
                    ret |= UART_LSR_DR;
                } */
            }
            UART_MSR => { /* 0x3fe */
                ret = self.value[UART_MSR];
            }
            UART_SCR => { /* 0x3ff */
                ret = self.value[UART_SCR];
            }
            _ => {
                println!("Unknown tty load offset {}", offset);
            }
        }

        self.update_irq(&irqchip);

        ret
    }

    pub fn store_emulation(&mut self, mmio_addr: u64, data: u8, 
        irqchip: &Arc<dyn IrqChip>) -> i32 {
        let mut ret: i32 = 0;
        let offset = mmio_addr - 0x3f8;

        match offset as usize {
            UART_TX => { /* 0x3f8 */
                if self.value[UART_LCR] & UART_LCR_DLAB != 0 {
                    self.value[UART_DLL] = data;
                } else if (self.value[UART_MCR] & UART_MCR_LOOP) != 0 {
                    /* loop mode */
                    panic!("Loop mode not implemented");
                } else {
                    /* If DLAB=0, just output the char. */
                    console_putchar(data as u64);
                    self.flush_tx();

                    /* Since the output is finished, notice the guest */
                    //irqchip.trigger_irq(1, true);
                }
            }
            UART_IER => { /* 0x3f9 */
                if self.value[UART_LCR] & UART_LCR_DLAB == 0 {
                    self.value[UART_IER] = data & 0x0f;
                } else {
                    self.value[UART_DLM] = data;
                }
            }
            UART_IIR => { /* 0x3fa UART_FCR */
                self.value[UART_FCR] = data;
                dbgprintln!("fcr {:x}", data);
            }
            UART_LCR => { /* 0x3fb */
                self.value[UART_LCR] = data;
                dbgprintln!("lcr {:x}", data);
            }
            UART_MCR => { /* 0x3fc */
                self.value[UART_MCR] = data;
                dbgprintln!("mcr {:x}", data);
            }
            UART_SCR => { /* 0x3ff */
                self.value[UART_SCR] = data;
                dbgprintln!("scr {:x}", data);
            }
            _ => {
                println!("Unknown tty store offset {}", offset);
                ret = 1;
            }
        }

        self.update_irq(&irqchip);

        ret
    }

    pub fn get_char(&mut self) -> Option<char> {
        let res: char;

        if self.recv_buf_head == FIFO_LEN {
            /* Not start yet */
            return None;
        } else if self.recv_buf_head == self.recv_buf_tail {
            if self.avail_char != FIFO_LEN {
                /* Empty */
                return None;
            } else {
                /* Full */
                res = self.recv_buf[self.recv_buf_head] as char;
                self.recv_buf_head += 1;
                self.avail_char -= 1;
            }
        } else {
            res = self.recv_buf[self.recv_buf_head] as char;
            self.recv_buf_head  = (self.recv_buf_head + 1) % FIFO_LEN;
            self.avail_char -= 1;
        }

        return Some(res);
    }

    pub fn flush_tx(&mut self) {
        /* Transmitter Empty */
        self.value[UART_LSR] |= UART_LSR_TEMT;

        /* Trasmitter hold empty */
        self.value[UART_LSR] |= UART_LSR_THRE;
    }

    pub fn recv_char(&mut self, input: char) -> i32 {
        if self.recv_buf_head == FIFO_LEN {
            /* First char */
            self.recv_buf_head = 0;
            self.recv_buf_tail = 1;
            self.recv_buf[0] = input;
            self.avail_char += 1;
            self.value[UART_LCR] |= UART_LSR_DR;
            return 0;
        } else if self.recv_buf_head == self.recv_buf_tail {
            if self.avail_char == FIFO_LEN {
                /* Full */
                return 1;
            } else {
                /* Empty */
                self.recv_buf[self.recv_buf_head] = input;
                self.recv_buf_tail = (self.recv_buf_tail + 1) % FIFO_LEN;
                self.avail_char += 1;
                self.value[UART_LCR] |= UART_LSR_DR;
                return 0;
            }  
        } else {
            self.recv_buf[self.recv_buf_tail] = input;
            self.recv_buf_tail = (self.recv_buf_tail + 1) % FIFO_LEN;
            self.avail_char += 1;
            self.value[UART_LCR] |= UART_LSR_DR;
            return 0;
        }
    }
}
