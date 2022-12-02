use crate::mm::utils::*;
#[allow(unused)]
use crate::vcpu::utils::*;
use crate::vcpu::virtualcpu::VirtualCpu;
#[allow(unused)]
use crate::plat::uhe::csr::csr_constants::*;
use crate::irq::delegation::delegation_constants::*;
use crate::plat::uhe::ioctl::ioctl_constants::*;
use error_code::*;
use sbi_number::*;
use sbi_test::*;
use crate::init::cmdline::MAX_VCPU;
use std::sync::atomic::Ordering;
use std::{thread, time};
use crate::irq::vipi::VirtualIpi;
use std::io::{self, Write};

use irq_util::SharedStat;

#[cfg(test)]
use crate::irq::vipi::tests::TEST_SUCCESS_CNT;

#[cfg(test)]
use crate::irq::vipi::tests::INVALID_TARGET_VCPU;

/* Flag for SBI SHUTDOWN */
pub static mut SHUTDOWN_FLAG: i32 = 0;

pub mod sbi_number {
    pub const SBI_EXT_0_1_SET_TIMER: u64 = 0x0;
    pub const SBI_EXT_0_1_CONSOLE_PUTCHAR: u64 = 0x1;
    pub const SBI_EXT_0_1_CONSOLE_GETCHAR: u64 = 0x2;
    pub const SBI_EXT_0_1_CLEAR_IPI: u64 = 0x3;
    pub const SBI_EXT_0_1_SEND_IPI: u64 = 0x4;
    pub const SBI_EXT_0_1_REMOTE_FENCE_I: u64 = 0x5;
    pub const SBI_EXT_0_1_REMOTE_SFENCE_VMA: u64 = 0x6;
    pub const SBI_EXT_0_1_REMOTE_SFENCE_VMA_ASID: u64 = 0x7;
    pub const SBI_EXT_0_1_SHUTDOWN: u64 = 0x8;
    pub const SBI_EXT_0_1_DEBUG_START: u64 = 0x11;
    pub const SBI_EXT_0_1_DEBUG_END: u64 = 0x12;
    pub const SBI_EXT_0_1_DEBUG: u64 = 0x9;
    pub const ECALL_CALL_FOR_VIRQ: u64 = 0xF0;
    pub const ECALL_VM_TEST_END: u64 = 0xFF;
    pub const ECALL_WRONG_IRQ: u64 = 0xF8;
    pub const ECALL_ENTER_HANDLER: u64 = 0xF9;
    pub const ECALL_HANDLER_FINISH: u64 = 0xFA;
    pub const ECALL_CLAIM_FINISH: u64 = 0xFB;

    pub const ECALL_CALL_FOR_UTIMER: u64 = 0xE0;
    pub const ECALL_HANDLER_START: u64 = 0xE1;
    pub const ECALL_RIGHT_CAUSE: u64 = 0xE2;
    pub const ECALL_WRONG_CAUSE: u64 = 0xE3;
    pub const ECALL_STEP_LOG: u64 = 0xE4;
    pub const ECALL_STOP_UTIMER: u64 = 0xE5;
}

/*
 * SBI introduced for evaluation, test cases of this project.
 * Extension name: ULH Extension
 * The SBI extension space is 0xC000000-0xCFFFFFF
 */
pub mod sbi_test {
    pub const SBI_TEST_SPACE_START: u64 = 0xC000000;
    pub const SBI_TEST_SPACE_END: u64 = 0xCFFFFFF;
    
    pub const SBI_TEST_HU_VIRTUAL_IPI: u64 = 0xC000001;

    /* Test result */
    pub const SBI_TEST_SUCCESS: u64 = 0xC000007;
    pub const SBI_TEST_FAILED: u64 = 0xC000008;

    /* Loop in HU-mode */
    pub const SBI_TEST_HU_LOOP : u64 = 0xC100000;

    /* Timing */
    pub const SBI_TEST_TIMING_START: u64 = 0xC200000;
    pub const SBI_TEST_TIMING_END: u64 = 0xC200001;

    /* Call local sbi for evaluation */
    pub const SBI_TEST_LOCAL_SBI: u64 = 0xC200002;
    pub const SBI_DEBUG_NEW_VIPI: u64 = 0xC200003;

}

#[allow(unused)]
pub mod error_code {
    pub const SBI_SUCCESS: i64 = 0;
    pub const SBI_ERR_FAILURE: i64 = -1;
    pub const SBI_ERR_NOT_SUPPORTED: i64 = -2;
    pub const SBI_ERR_INVALID_PARAM: i64 = -3;
    pub const SBI_ERR_DENIED: i64 = -4;
    pub const SBI_ERR_INVALID_ADDRESS: i64 = -5;
}

#[allow(unused)]
extern "C"
{
    fn getchar_emulation() -> i32;
    fn wrvtimectl(val: u64);
    fn wrvtimecmp(val: u64);
}

pub struct Ecall {
    /* EID - a7 */
    pub ext_id: u64,

    /* FID - a6 */
    pub func_id: u64,

    /* Args - a0~a5 */
    pub arg: [u64; 6],

    /* Return - a0, a1 */
    pub ret: [u64; 2],
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

    /* 
     * Emulation for the ecall from VS-mode, however part of the ecall cannot 
     * be finished in U-mode for now. So pass the ioctl_fd to call kernel
     * module. 
     */
    pub fn ecall_handler(&mut self, ioctl_fd: i32, vcpu: &VirtualCpu) -> i32 {
        let ext_id = self.ext_id;
        let ret: i32;

        match ext_id {
            SBI_TEST_TIMING_START => {
                SharedStat::start_breakdown();
                ret = 0;
            },
            SBI_TEST_TIMING_END => {
                SharedStat::end_breakdown();
                ret = 0;
            },
            SBI_TEST_LOCAL_SBI => {
                println!("ALL TEST DONE\n");
                ret = 0;
            },
            SBI_DEBUG_NEW_VIPI => {
                //println!("I am vcpu {}, args {} {} {}", vcpu.vcpu_id, self.arg[0], self.arg[1], self.arg[3]);
                ret = 0;
            },
            SBI_EXT_0_1_SET_TIMER => {
                /* 
                 * TODO: add rust feature to tell between rv64 and rv32
                 * TODO: next_cycle = ((u64)cp->a1 << 32) | (u64)cp->a0; if
                 * rv32
                 */
                let next_cycle = self.arg[0];

                //println!("Set timer");
                
                /*
                 * Linux thinks that the IRQ_S_TIMER will be cleared when ecall
                 * SBI_EXT_0_1_SET_TIMER
                 * For record, opensbi thinks that IRQ_M_TIMER should be 
                 * cleared by software.
                 * Qemu and xv6 think that IRQ_M_TIMER should be clear when 
                 * writing timecmp. 
                 * I think that IRQ_U_TIMER should be cleared by software.
                 * That's a drawback of riscv, unlike GIC which can provide the
                 * same interface for eoi. 
                 */
                vcpu.virq.unset_pending_irq(IRQ_VS_TIMER);
                unsafe {
                    #[cfg(feature = "xilinx")]
                    {
                        wrvtimectl(1);
                        wrvtimecmp(next_cycle);
                    } 

                    #[cfg(feature = "qemu")]
                    {
                        csrw!(VTIMECTL, (IRQ_U_TIMER << 1) | (1 << VTIMECTL_ENABLE));
                        csrw!(VTIMECMP, next_cycle);
                    }
                }
                dbgprintln!("set vtimer for ulh");
                ret = 0;
            },
            SBI_EXT_0_1_CONSOLE_PUTCHAR => {
                ret = self.console_putchar();
            },
            SBI_EXT_0_1_CONSOLE_GETCHAR => {
                ret = self.console_getchar();
            },
            SBI_EXT_0_1_CLEAR_IPI => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
                ret = self.unsupported_sbi();
            },
            SBI_EXT_0_1_SEND_IPI => {
                extern crate irq_util;
                use irq_util::SharedStat;
                dbgprintln!("ready to hart mask");
                let hart_mask = self.get_hart_mask(self.arg[0]);
                dbgprintln!("finish hart mask");

                let mut vipi_id: u64;
                for i in 0..MAX_VCPU {
                    if ((1 << i) & hart_mask) != 0 {
                        /* Check whether the target vcpu is valid */
                        if i >= vcpu.vipi.vcpu_num {
                            /* Invalid target */
                            #[cfg(test)]
                            unsafe {
                                *INVALID_TARGET_VCPU.lock().unwrap() += 1;
                            }

                            continue;
                        }
                        vipi_id = vcpu.vipi.vcpu_id_map[i as usize]
                            .load(Ordering::SeqCst);
                        SharedStat::add_shared_mem(110020 + i as usize, 1);
                        let vcpu_state =  vcpu.irqchip.get().unwrap()
                            .trigger_virtual_irq(i);
                        if vcpu_state == 1 {
                            VirtualIpi::set_vipi(vipi_id);
                        }
                    }
                }
                //println!("hart mask 0x{:x}", hart_mask);
                //println!("{} send ipi ... vipi-id-0 is {}", vcpu.vcpu_id, vcpu.vipi.vcpu_id_map[0 as usize].load(Ordering::SeqCst));
                //println!("{} send ipi ... vipi-id-1 is {}", vcpu.vcpu_id, vcpu.vipi.vcpu_id_map[1 as usize].load(Ordering::SeqCst));

                ret = 0;
            },
            SBI_EXT_0_1_SHUTDOWN => {
                println!("Poweroff the virtual machine by vcpu {}",
                    vcpu.vcpu_id);
                ret = -100;
                unsafe {
                    SHUTDOWN_FLAG = 1;
                }
            },
            SBI_EXT_0_1_DEBUG => {
                println!("SBI DEBUG 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
            //    unsafe {
            //        libc::ioctl(ioctl_fd, 0x6b10); // Output the VS* csrs.
            //    }
                ret = 0;
            },
            ECALL_CALL_FOR_VIRQ => {
                static mut virq_cnt: u64 = 0;
                unsafe {
                virq_cnt += 1;
                
                    if (virq_cnt < 50000) {
                        println!("ECALL_CALL_FOR_VIRQ {} CNT, 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", virq_cnt, self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
                        libc::ioctl(ioctl_fd, 0x80086b0f, 0);
                    }

                    if (virq_cnt < 3) {
                        #[cfg(feature = "qemu")]
                        {
                            csrw!(VTIMECTL, (IRQ_U_TIMER << 1) | (1 << VTIMECTL_ENABLE));
                            csrw!(VTIMECMP, self.arg[5] + virq_cnt * 0x10000);
                        }

                        #[cfg(feature = "xilinx")]
                        {
                            wrvtimectl(1);
                            wrvtimecmp(self.arg[5] + virq_cnt * 0x10000);
                        } 
                    }
                }
                ret = 0;
            },
            ECALL_VM_TEST_END => {
                static mut test_cnt: u64 = 0;
                unsafe {
                test_cnt += 1;
                println!("**********ECALL_VM_TEST_END {} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", test_cnt, self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
                }
                ret = 0;
            },
            ECALL_WRONG_IRQ => {
                println!("**********ECALL_WRONG_IRQ 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
                ret = 0;
            },
            ECALL_ENTER_HANDLER => {
                println!("**********ECALL_ENTER_HANDLER***");
                ret = 0;
            },
            ECALL_CALL_FOR_UTIMER => {
                static mut timer_cnt: u64 = 0;
                unsafe {
                    timer_cnt += 1;

                    println!("**********ECALL_CALL_FOR_UTIMER {} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", 
                        timer_cnt, self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
                    
                    #[cfg(feature = "xilinx")]
                    {
                        wrvtimectl(1);
                        wrvtimecmp(self.arg[5] + timer_cnt * 0x1000);
                    }

                    #[cfg(feature = "qemu")]
                    {
                        csrw!(VTIMECTL, (IRQ_U_TIMER << 1) | (1 << VTIMECTL_ENABLE));
                        csrw!(VTIMECMP, self.arg[5] + timer_cnt * 0x1000);
                    }
                }
                ret = 0;
            },
            ECALL_HANDLER_START => {
                println!("**********ECALL_HANDLER_START 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", 
                    self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
                ret = 0;
            },
            ECALL_RIGHT_CAUSE => {
                println!("**********ECALL_RIGHT_CAUSE 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", 
                    self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
                ret = 0;
            },
            ECALL_WRONG_CAUSE => {
                println!("**********ECALL_WRONG_CAUSE 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", 
                    self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
                ret = 0;
            },
            ECALL_STEP_LOG => {
                static mut log_cnt: u64 = 0;

                unsafe {
                log_cnt += 1;
                println!("**********ECALL_STEP_LOG {} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", log_cnt,
                    self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
                }
                ret = 0;
            },
            ECALL_STOP_UTIMER => {
                println!("**********ECALL_STOP_UTIMER 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", 
                    self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
                println!("****Unset utimer!!!!");
                
                vcpu.virq.unset_pending_irq(IRQ_VS_TIMER);
                unsafe {
                    #[cfg(feature = "xilinx")]
                    {
                        wrvtimectl(0);
                        csrc!(HUIP, 1 << IRQ_U_TIMER);
                    }

                    #[cfg(feature = "qemu")]
                    {
                        csrc!(VTIMECTL, 1 << VTIMECTL_ENABLE);
                        csrc!(HUIP, 1 << IRQ_U_TIMER);
                    }
                }
                ret = 0;
            },
            ECALL_HANDLER_FINISH => {
                println!("**********ECALL_HANDLER_FINISH 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", 
                    self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
                ret = 0;
            },
            ECALL_CLAIM_FINISH => {
                println!("**********ECALL_CLAIM_FINISH 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x} 0x{:x}", 
                    self.arg[0], self.arg[1], self.arg[2], self.arg[3], self.arg[4], self.arg[5]);
                ret = 0;
            },
            SBI_EXT_0_1_REMOTE_FENCE_I | SBI_EXT_0_1_REMOTE_SFENCE_VMA
                    | SBI_EXT_0_1_REMOTE_SFENCE_VMA_ASID=> {
                /* 
                 * All of these three SBIs will be directly emulated as
                 * SBI_EXT_0_1_REMOTE_FENCE_I for now.
                 */
                unsafe {
                    let ecall_ret: [u64;2] = [0, 0];
                    let ret_ptr = (&ecall_ret) as *const u64;

                    /* Call ioctl IOCTL_REMOTE_FENCE to kernel module */
                    let _res = libc::ioctl(ioctl_fd, IOCTL_REMOTE_FENCE,
                            ret_ptr);
                                        
                    self.ret[0] = ecall_ret[0];
                    self.ret[1] = ecall_ret[1];
                }
                ret = 0;
            },
            SBI_EXT_0_1_DEBUG_START => {
                extern crate irq_util;
                use irq_util::SharedStat;
                SharedStat::reset_all();
                SharedStat::set_debug_flag(true);
                ret = 0;
            },
            SBI_EXT_0_1_DEBUG_END => {
                extern crate irq_util;
                use irq_util::SharedStat;
                SharedStat::set_debug_flag(false);
                SharedStat::print_all();
                ret = 0;
            },
            SBI_TEST_SPACE_START..=SBI_TEST_SPACE_END => { /* ULH Extension */
                ret = self.ulh_extension_emulation(vcpu);
            },
            _ => {
                println!("EXT ID {} has not been implemented yet.", ext_id);
                ret = self.unsupported_sbi();
            },
        }

        ret
    }

    fn ulh_extension_emulation(&mut self, vcpu: &VirtualCpu) -> i32{
        let ext_id = self.ext_id;

        match ext_id {
            SBI_TEST_HU_VIRTUAL_IPI => {
                /* Set vipi for the vcpu itself */
                vcpu.irqchip.get().unwrap().trigger_virtual_irq(vcpu.vcpu_id);
            },
            SBI_TEST_HU_LOOP => {
                /* Keep the vcpu thread in HU-mode */

                /* Get hva of the sync data and the end signal */
                let target_hva: u64 = self.arg[1];
                let start_signal = self.arg[2];
                let end_signal = self.arg[3];
                println!("target a1: 0x{:x}", target_hva);
                println!("start signal a2: {}", start_signal);
                println!("end signal a3: {}", end_signal);

                unsafe {
                    /* Set up the start signal */
                    *(target_hva as *mut u64) = start_signal;

                    /* Wait for the end signal */
                    while *(target_hva as *mut u64) != end_signal {
                        let ten_millis = time::Duration::from_millis(10);

                        thread::sleep(ten_millis);
                    }
                }

                println!("SBI_TEST_HU_LOOP end!");
            },
            SBI_TEST_SUCCESS => {
                #[cfg(test)]
                unsafe {
                    *TEST_SUCCESS_CNT.lock().unwrap() += 1;
                }

                dbgprintln!("***SBI_TEST_SUCCESS vcpu: {}", vcpu.vcpu_id);
            },
            SBI_TEST_FAILED => {
                dbgprintln!("SBI_TEST_FAILED {}", vcpu.vcpu_id);
            },
            _ => {
                dbgprintln!("EXT ID {} has not been implemented yet.", ext_id);
                self.unsupported_sbi();
            },
        }

        0
    }

    /* Get hart_mask from guest memory by the address in a0 */
    fn get_hart_mask(&self, target_address: u64) -> u64 {
        let a0 = target_address;
        let hart_mask: u64;
        dbgprintln!("get_hart_mask a0 = 0x{:x}", a0);
        unsafe {
            asm!(
                ".option push",
                ".option norvc",

                /* HULVX.HU t0, (t2) */
                ".word 0x6c03c2f3",

                /* HULVX.HU t1, (t2) */
                out("t0") hart_mask,
                in("t2") a0,
            );
        }

        return hart_mask;
    }

    fn console_putchar(&mut self) -> i32{
        let ch = self.arg[0] as u8;
        let ch = ch as char;
        print_flush!("{}", ch);

        /* Success and return with a0 = 0 */
        self.ret[0] = 0;

        0
    }

    fn unsupported_sbi(&mut self) -> i32{
        /* SBI error and return with a0 = SBI_ERR_NOT_SUPPORTED */
        self.ret[0] = SBI_ERR_NOT_SUPPORTED as u64;

        0
    }

    fn console_getchar(&mut self) -> i32{
        let ret: i32;

        /* Cannot switch the backend process to the front. */
        /* So test_ecall_getchar() have to get chars from here.  */
        #[cfg(test)]
        {
            let virtual_input: [i32; 16];

            /* Input "getchar succeed\n" */
            virtual_input = [103, 101, 116, 99, 104, 97, 114, 32, 115, 117, 99,
                99, 101, 101, 100, 10];
    
            static mut INDEX: usize = 0;

            unsafe {
                ret = virtual_input[INDEX];
                INDEX += 1;
            }

            /* Success and return with a0 = 0 */
            self.ret[0] = ret as u64;

            return 0;
        }

        #[allow(unreachable_code)]
        {
            unsafe {
                ret = getchar_emulation();
            }
    
            /* Success and return with a0 = 0 */
            self.ret[0] = ret as u64;
    
            0
        }
    }
}

