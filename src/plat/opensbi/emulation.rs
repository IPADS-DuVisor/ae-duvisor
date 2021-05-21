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

#[allow(unused)]
extern "C"
{
    fn getchar_emulation() -> i32;
}

pub struct Ecall {
    pub ext_id: u64, // EID - a7
    pub func_id: u64, // FID - a6
    pub arg: [u64; 6], // args - a0~a5
    pub ret: [u64; 2], // return - a0, a1
}

impl Ecall {
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

    pub fn ecall_handler(&mut self) -> i32 {
        let ext_id = self.ext_id;
        let mut ret: i32 = 1;

        match ext_id {
            SBI_EXT_0_1_SET_TIMER => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
            },
            SBI_EXT_0_1_CONSOLE_PUTCHAR => {
                ret = self.console_putchar();
            },
            SBI_EXT_0_1_CONSOLE_GETCHAR => {
                ret = self.console_getchar();
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

        ret
    }

    fn console_putchar(&mut self) -> i32{
        let ch = self.arg[0] as u8;
        let ch = ch as char;
        print!("{}", ch);

        // success and return with a0 = 0
        self.ret[0] = 0;

        0
    }

    fn console_getchar(&mut self) -> i32{
        let ret: i32;

        // Cannot switch the backend process to the front.
        // So test_ecall_getchar() have to get chars from here. 
        #[cfg(test)]
        {
            let virtual_input: [i32; 16];

            // input "getchar succeed\n"
            virtual_input = [103, 101, 116, 99, 104, 97, 114, 32, 115, 117, 99,
                99, 101, 101, 100, 10];
    
            static mut INDEX: usize = 0;

            unsafe {
                ret = virtual_input[INDEX];
                INDEX += 1;
            }

            // success and return with a0 = 0
            self.ret[0] = ret as u64;

            return 0;
        }

        unsafe {
            ret = getchar_emulation();
        }

        // success and return with a0 = 0
        self.ret[0] = ret as u64;

        0
    }
}

