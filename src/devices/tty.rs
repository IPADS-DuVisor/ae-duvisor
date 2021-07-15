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
    pub const UART_IER_RDI: u8 = 0x01;
    pub const UART_IER_THRI: u8 = 0x02;
    pub const UART_IER_RLSI: u8 = 0x04;
    pub const UART_IRE_MSI: u8 = 0x08;
    pub const UART_IERX_SLEEP: u8 = 0x10;

    /* UART_IIR */
    pub const UART_IIR: usize = 2;
    pub const UART_IIR_NO_INT: u8 = 0x1;
    pub const UART_IIR_ID: u8 = 0x0e;
    pub const UART_IIR_MSI: u8 = 0x00;
    pub const UART_IIR_RDI: u8 = 0x4;
    pub const UART_IIR_THRI: u8 = 0x2;
    pub const UART_IIR_TYPE_BITS: u8 = 0xc0;
    pub const UART_IIR_RLSI: u8 = 0x06;
    pub const UART_IIR_BUSY: u8 = 0x07;
    pub const UART_IIR_RX_TIMEOUT: u8 = 0x0c;
    pub const UART_IIR_XOFF: u8 = 0x10;
    pub const UART_IIR_CTS_RTS_DSR: u8 = 0x20;

    /* UART_LCR */
    pub const UART_LCR: usize = 3;
    pub const UART_LCR_DLAB: u8 = 0x80;
    pub const UART_LCR_SBC: u8 = 0x40;
    pub const UART_LCR_SPAR: u8 = 0x20;
    pub const UART_LCR_EPAR: u8 = 0x10;
    pub const UART_LCR_PARITY: u8 = 0x08;
    pub const UART_LCR_STOP: u8 = 0x04;
    pub const UART_LCR_WLEN5: u8 = 0x00;
    pub const UART_LCR_WLEN6: u8 = 0x01;
    pub const UART_LCR_WLEN7: u8 = 0x02;
    pub const UART_LCR_WLEN8: u8 = 0x03;

    /* UART_MCR */
    pub const UART_MCR: usize = 4;
    pub const UART_MCR_OUT2: u8 = 0x08;
    pub const UART_MCR_LOOP: u8 = 0x10;
    pub const UART_MCR_CLKSEL: u8 = 0x80;
    pub const UART_MCR_TCRTLR: u8 = 0x40;
    pub const UART_MCR_XONANY: u8 = 0x20;
    pub const UART_MCR_AFE: u8 = 0x20;
    pub const UART_MCR_OUT1: u8 = 0x04;
    pub const UART_MCR_RTS: u8 = 0x02;
    pub const UART_MCR_DTR: u8 = 0x01;

    /* UART_LSR */
    pub const UART_LSR: usize = 5;
    pub const UART_LSR_TEMT: u8 = 0x40;
    pub const UART_LSR_THRE: u8 = 0x20;
    pub const UART_LSR_DR: u8 = 0x1;
    pub const UART_LSR_FIFOE: u8 = 0x80;
    pub const UART_LSR_BI: u8 = 0x10;
    pub const UART_LSR_FE: u8 = 0x08;
    pub const UART_LSR_PE: u8 = 0x04;
    pub const UART_LSR_OE: u8 = 0x02;
    pub const UART_LSR_BRK_ERROR_BITS: u8 = 0x1E;

    /* UART_MSR */
    pub const UART_MSR: usize = 6;
    pub const UART_MSR_DCD: u8 = 0x80;
    pub const UART_MSR_RI: u8 = 0x40;
    pub const UART_MSR_DSR: u8 = 0x20;
    pub const UART_MSR_CTS: u8 = 0x10;
    pub const UART_MSR_DDCD: u8 = 0x08;
    pub const UART_MSR_TERI: u8 = 0x04;
    pub const UART_MSR_DDSR: u8 = 0x02;
    pub const UART_MSR_DCTS: u8 = 0x01;
    pub const UART_MSR_ANY_DELTA: u8 = 0x0F;

    /* UART_SCR */
    pub const UART_SCR: usize = 7;

    /* UART_DLL */
    pub const UART_DLL: usize = 0; /* Reuse offset 0 */

    /* UART_DLM */
    pub const UART_DLM: usize = 1; /* Reuse offset 1 */

    /* UART_FCR */
    pub const UART_FCR: usize = 2; /* Reuse offset 2 */
    pub const UART_FCR_ENABLE_FIFO: u8 = 0x01;
    pub const UART_FCR_CLEAR_RCVR: u8 = 0x02;
    pub const UART_FCR_CLEAR_XMIT: u8 = 0x04;
    pub const UART_FCR_DMA_SELECT: u8 = 0x04;
    pub const UART_FCR_R_TRIG_00: u8 = 0x00;
    pub const UART_FCR_R_TRIG_01: u8 = 0x40;
    pub const UART_FCR_R_TRIG_10: u8 = 0x80;
    pub const UART_FCR_R_TRIG_11: u8 = 0xc0;
    pub const UART_FCR_T_TRIG_00: u8 = 0x00;
    pub const UART_FCR_T_TRIG_01: u8 = 0x10;
    pub const UART_FCR_T_TRIG_10: u8 = 0x20;
    pub const UART_FCR_T_TRIG_11: u8 = 0x30;
    pub const UART_FCR_TRIGGER_MASK: u8 = 0xc0;
    pub const UART_FCR_TRIGGER_1: u8 = 0x00;
    pub const UART_FCR_TRIGGER_4: u8 = 0x40;
    pub const UART_FCR_TRIGGER_8: u8 = 0x80;
    pub const UART_FCR_TRIGGER_14: u8 = 0xC0;
    pub const UART_FCR6_R_TRIGGER_8: u8 = 0x00;
    pub const UART_FCR6_R_TRIGGER_16: u8 = 0x40;
    pub const UART_FCR6_R_TRIGGER_24: u8 = 0x80;
    pub const UART_FCR6_R_TRIGGER_28: u8 = 0xC0;
    pub const UART_FCR6_T_TRIGGER_16: u8 = 0x00;
    pub const UART_FCR6_T_TRIGGER_8: u8 = 0x10;
    pub const UART_FCR6_T_TRIGGER_24: u8 = 0x20;
    pub const UART_FCR6_T_TRIGGER_30: u8 = 0x30;
    pub const UART_FCR7_64BYTE: u8 = 0x20;
}

pub const FIFO_LEN: usize = 64;

pub struct Tty {
    pub dll: u8,
    pub dlm: u8,
    pub iir: u8,
    pub ier: u8,
    pub fcr: u8,
    pub lcr: u8,
    pub mcr: u8,
    pub lsr: u8,
    pub msr: u8,
    pub scr: u8,
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
        let recv_buf: [char; FIFO_LEN] = [0 as char; FIFO_LEN];
        let recv_buf_head: usize = FIFO_LEN;
        let recv_buf_tail: usize = 0;
        let avail_char: usize = 0;
        let irq_state: u8 = 0;

        /* TtyS0 init */
        let iir: u8 = UART_IIR_NO_INT;
        let mcr: u8 = UART_MCR_OUT2;
        let lsr: u8 = UART_LSR_TEMT | UART_LSR_THRE;
        let msr: u8 = UART_MSR_DCD | UART_MSR_DSR | UART_MSR_CTS;
        let dll: u8 = 0;
        let dlm: u8 = 0;
        let ier: u8 = 0;
        let fcr: u8 = 0;
        let lcr: u8 = 0;
        let scr: u8 = 0;

        Self {
            dll,
            dlm,
            iir,
            ier,
            fcr,
            lcr,
            mcr,
            lsr,
            msr,
            scr,
            recv_buf,
            avail_char,
            recv_buf_head,
            recv_buf_tail,
            irq_state,
        }
    }

    /* 
     * Get char from recv_buf automitically.
     * Exploit the vtimer for now.
     * Set the DR of LSR for data ready.
     * Update irq with UART_LSR_DR.
     */
    pub fn update_recv(&mut self, irqchip: &Arc<dyn IrqChip>) {
        let avail_char = self.avail_char;

        if avail_char > 0 {
            dbgprintln!("avail_char {}", avail_char);
            self.lsr |= UART_LSR_DR;
            self.update_irq(&irqchip);
        }
    }

    pub fn update_irq(&mut self, irqchip: &Arc<dyn IrqChip>) {
        let mut iir: u8 = 0;

        /* Handle clear rx */
        if self.lcr & UART_FCR_CLEAR_RCVR != 0 {
            self.lcr &= !UART_FCR_CLEAR_RCVR;
            self.lsr &= !UART_LSR_DR;
        }

        /* Handle clear tx */
        if self.lcr & UART_FCR_CLEAR_XMIT != 0 {
            self.lcr &= !UART_FCR_CLEAR_XMIT;
            self.lsr |= UART_LSR_TEMT | UART_LSR_THRE;
        }

        /* Data ready and rcv interrupt enabled ? */
        if (self.ier & UART_IER_RDI != 0) && (self.lsr & UART_LSR_DR != 0) {
            iir |= UART_IIR_RDI;
        }

        /* Transmitter empty and interrupt enabled ? */
        if (self.ier & UART_IER_THRI != 0) && (self.lsr & UART_LSR_TEMT != 0) {
            iir |= UART_IIR_THRI;
        }

        /* Now update the irq line, if necessary */
        if iir != 0 {
            self.iir = iir;

            if self.irq_state == 0 {
                dbgprintln!("[2] tty set");
                irqchip.trigger_irq(1, true);
            }
        } else {
            self.iir = UART_IIR_NO_INT;

            if self.irq_state != 0 {
                dbgprintln!("[1] tty clear");
                irqchip.trigger_irq(1, false);
            }
        }

        self.irq_state = iir;

        /*
         * If the kernel disabled the tx interrupt, we know that there
         * is nothing more to transmit, so we can reset our tx logic
         * here.
         */
        if self.ier & UART_IER_THRI == 0 {
            self.flush_tx();
        }
    }

    pub fn load_emulation(&mut self, mmio_addr: u64, 
        irqchip: &Arc<dyn IrqChip>) -> u8 {
        let offset = mmio_addr - 0x3f8;
        let mut ret: u8 = 0 as u8;

        match offset as usize {
            UART_RX => {
                if self.lcr & UART_LCR_DLAB != 0 {
                    ret = self.dll;
                } else {
                    /* Get input */
                    let res = self.get_char();

                    if res.is_some() {
                        ret = res.unwrap() as u8;
                    } else {
                        dbgprintln!("mmio fault: get_char failed");
                    }

                    /* If there is no chars, clear the DR bit */
                    if self.avail_char == 0 {
                        self.lsr &= !UART_LSR_DR;
                    }
                }
            }
            UART_IER => {
                if self.lcr & UART_LCR_DLAB != 0 {
                    ret = self.dlm;
                } else {
                    ret = self.ier;
                }
            }
            UART_IIR => {
                ret = self.iir | UART_IIR_TYPE_BITS;
            }
            UART_LCR => {
                ret = self.lcr;
            }
            UART_MCR => {
                ret = self.mcr;
            }
            UART_LSR => {
                ret = self.lsr;
            }
            UART_MSR => {
                ret = self.msr;
            }
            UART_SCR => {
                ret = self.scr;
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
            UART_TX => {
                if self.lcr & UART_LCR_DLAB != 0 {
                    self.dll = data;
                } else if (self.mcr & UART_MCR_LOOP) != 0 {
                    /* Loop mode */
                    /* TODO: loop mode for 8250 */
                    panic!("Loop mode not implemented");
                } else {
                    /* If DLAB=0, just output the char. */
                    console_putchar(data as u64);

                    /* Also output the recv_buf in kvmtool */
                    self.flush_tx();
                }
            }
            UART_IER => {
                if self.lcr & UART_LCR_DLAB == 0 {
                    self.ier = data & 0x0f;
                } else {
                    self.dlm = data;
                }
            }
            UART_FCR => {
                self.fcr = data;
                dbgprintln!("fcr {:x}", data);
            }
            UART_LCR => {
                self.lcr = data;
                dbgprintln!("lcr {:x}", data);
            }
            UART_MCR => {
                self.mcr = data;
                dbgprintln!("mcr {:x}", data);
            }
            UART_SCR => {
                self.scr = data;
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
        self.lsr |= UART_LSR_TEMT;

        /* Trasmitter hold empty */
        self.lsr |= UART_LSR_THRE;
    }

    pub fn recv_char(&mut self, input: char) -> i32 {
        if self.recv_buf_head == FIFO_LEN {
            /* First char */
            self.recv_buf_head = 0;
            self.recv_buf_tail = 1;
            self.recv_buf[0] = input;
            self.avail_char += 1;
            self.lcr |= UART_LSR_DR;
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
                self.lcr |= UART_LSR_DR;
                return 0;
            }  
        } else {
            self.recv_buf[self.recv_buf_tail] = input;
            self.recv_buf_tail = (self.recv_buf_tail + 1) % FIFO_LEN;
            self.avail_char += 1;
            self.lcr |= UART_LSR_DR;
            return 0;
        }
    }
}
