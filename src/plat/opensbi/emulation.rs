use crate::mm::utils::*;

pub const SBI_EXT_0_1_SET_TIMER: u64 = 0x0;
pub const SBI_EXT_0_1_CONSOLE_PUTCHAR: u64 = 0x1;
pub const SBI_EXT_0_1_CONSOLE_GETCHAR: u64 = 0x2;
pub const SBI_EXT_0_1_CLEAR_IPI: u64 = 0x3;
pub const SBI_EXT_0_1_SEND_IPI: u64 = 0x4;
pub const SBI_EXT_0_1_REMOTE_FENCE_I: u64 = 0x5;
pub const SBI_EXT_0_1_REMOTE_SFENCE_VMA: u64 = 0x6;
pub const SBI_EXT_0_1_REMOTE_SFENCE_VMA_ASID: u64 = 0x7;
pub const SBI_EXT_0_1_SHUTDOWN: u64 = 0x8;

pub struct SbiArg {
    pub ext_id: u64, // EID - a7
    pub func_id: u64, // FID - a6
    pub arg: [u64; 6], // args - a0~a5
    pub ret: [u64; 2], // return - a0, a1
}

impl SbiArg {
    pub fn new() -> Self {
        let ext_id: u64 = 0;
        let func_id: u64 = 0;
        let arg: [u64; 6] = [0; 6];
        let ret: [u64; 2] = [0; 2];

        Self {
            ext_id,
            func_id,
            arg,
            ret,
        }
    }

    pub fn ecall_handler(&mut self) {
        let ext_id = self.ext_id;

        match ext_id {
            SBI_EXT_0_1_SET_TIMER => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
            },
            SBI_EXT_0_1_CONSOLE_PUTCHAR => {
                self.console_putchar();
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
            },
            SBI_EXT_0_1_CONSOLE_GETCHAR => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
            },
            SBI_EXT_0_1_CLEAR_IPI => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
            },
            SBI_EXT_0_1_SEND_IPI => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
            },
            SBI_EXT_0_1_SHUTDOWN => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
            },
            SBI_EXT_0_1_REMOTE_FENCE_I => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
            },
            SBI_EXT_0_1_REMOTE_SFENCE_VMA => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
            },
            SBI_EXT_0_1_REMOTE_SFENCE_VMA_ASID => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
            },
            _ => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
            },
        }
    }

    fn console_putchar(&mut self) {
        dbgprintln!("console_putchar start");
        let ch = self.arg[0] as u8;
        let ch = ch as char;
        print!("1{}2", ch);
        dbgprintln!("console_putchar end");
/*         let ch_try = u8::try_from(self.arg[0]);
        if ch_try.is_ok() {
            let ch = ch_try.unwrap();
            print!("{}", ch);
        } */
        
    }
}

