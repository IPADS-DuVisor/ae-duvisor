// Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use libc::EAGAIN;
use net_util::{Error as TapError, Tap};
use net_sys;
use super::{Queue, VirtioDevice, INTERRUPT_STATUS_USED_RING, TYPE_NET};
use std::cmp;
use std::io::{Read, Write};
use std::mem;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use sys_util::{Error as SysError, EventFd, GuestMemory, Pollable, Poller, GuestAddress};
use virtio_sys::virtio_net;

use irq_util::IrqChip;
use irq_util::SharedStat;
extern crate core_affinity;

/// The maximum buffer size when segmentation offload is enabled. This
/// includes the 12-byte virtio net header.
/// http://docs.oasis-open.org/virtio/virtio/v1.0/virtio-v1.0.html#x1-1740003
const MAX_BUFFER_SIZE: usize = 65562;
const QUEUE_SIZE: u16 = 256;
const QUEUE_SIZES: &'static [u16] = &[QUEUE_SIZE, QUEUE_SIZE];

#[derive(Debug)]
pub enum NetError {
    /// Creating kill eventfd failed.
    CreateKillEventFd(SysError),
    /// Cloning kill eventfd failed.
    CloneKillEventFd(SysError),
    /// Open tap device failed.
    TapOpen(TapError),
    /// Setting tap IP failed.
    TapSetIp(TapError),
    /// Setting tap netmask failed.
    TapSetNetmask(TapError),
    /// Setting tap interface offload flags failed.
    TapSetOffload(TapError),
    /// Setting vnet header size failed.
    TapSetVnetHdrSize(TapError),
    /// Enabling tap interface failed.
    TapEnable(TapError),
    /// Error while polling for events.
    PollError(SysError),
}

struct Worker {
    mem: GuestMemory,
    rx_queue: Queue,
    tx_queue: Queue,
    tap: Tap,
    interrupt_status: Arc<AtomicUsize>,
    #[allow(unused)]
    interrupt_evt: EventFd,
    rx_buf: [u8; MAX_BUFFER_SIZE],
    rx_count: usize,
    deferred_rx: bool,
    // TODO(smbarber): http://crbug.com/753630
    // Remove once MRG_RXBUF is supported and this variable is actually used.
    #[allow(dead_code)] acked_features: u64,
    irqchip: Arc<dyn IrqChip>,
}

static mut RX_CYCLE_START: usize = 0;
static mut RX_CYCLE_TOTAL: usize = 0;
static mut MID_CYCLE_TOTAL: [usize; 16] = [0; 16];
static mut RX_TIME_START: usize = 0;
static mut RX_TIME_TOTAL: usize = 0;
static mut RX_LEN_TOTAL: usize = 0;
static mut MID_TIME_TOTAL: [usize; 16] = [0; 16];
static mut MID_CNT_TOTAL: [usize; 16] = [0; 16];
static mut TX_NOTIFY_CNT: usize = 0;

impl Worker {
    fn signal_used_queue(&self, nr_irq: u32) {
        self.interrupt_status
            .fetch_or(INTERRUPT_STATUS_USED_RING as usize, Ordering::SeqCst);
        //self.interrupt_evt.write(1).unwrap();
        self.irqchip.trigger_edge_irq(nr_irq);
    }
    
    fn net_fix_rx_hdr(&self, mem: &GuestMemory, index: u16, num_buffers: u16) {
        unsafe {
            let desc_head_hva = mem.get_host_address(
                self.rx_queue.desc_table.unchecked_add(
                    (index as usize) * 16)).unwrap();
            let addr = *(desc_head_hva as *const u64) as usize;
            let addr_hva = mem.get_host_address(GuestAddress(addr)).unwrap();
            *(addr_hva.add(10) as *mut u16) = num_buffers;
        }
        //let desc_head = mem.checked_offset(
        //    self.rx_queue.desc_table, (index as usize) * 16).unwrap();
        //let addr = mem.read_obj_from_addr::<u64>(desc_head).unwrap();
        //mem.write_obj_at_addr::<u16>(num_buffers, GuestAddress(addr as usize + 10)).unwrap();
    }

    // Copies a single frame from `self.rx_buf` into the guest. Returns true
    // if a buffer was used, and false if the frame must be deferred until a buffer
    // is made available by the driver.
    fn rx_single_frame(&mut self) -> bool {
        let mut next_desc = self.rx_queue.iter(&self.mem).next();

        if next_desc.is_none() {
            return false;
        }

        unsafe {
            asm!("csrr {}, 0xC00", out(reg) RX_CYCLE_START);
            asm!("csrr {}, 0xC01", out(reg) RX_TIME_START);
        }
        // We just checked that the head descriptor exists.
        let head_index = next_desc.as_ref().unwrap().index;
        let mut first_index = head_index;
        let mut write_count = 0;
        let mut io_size = 0;
        let mut num_buffers: u16 = 0;
        
        let mut cycle_start: usize = 0;
        let mut memcpy_start: usize = 0;

        // Copy from frame into buffer, which may span multiple descriptors.
        loop {
            match next_desc {
                Some(ref mut desc) => {
                    if !desc.is_write_only() {
                        break;
                    }
                    unsafe {
                        asm!("csrr {}, 0xC00", out(reg) cycle_start);
                        asm!("csrr {}, 0xC01", out(reg) memcpy_start);
                    }
                    loop {
                        let limit = cmp::min(write_count + desc.len as usize, self.rx_count);
                        let source_slice = &self.rx_buf[write_count..limit];
                        let write_result = self.mem.write_slice_at_addr(source_slice, desc.addr);

                        match write_result {
                            Ok(sz) => {
                                let old_wr_cnt = write_count;
                                write_count += sz;
                                io_size += sz;
                                desc.addr = desc.addr.unchecked_add(sz);
                                desc.len -= sz as u32;
                                if (write_count < self.rx_count) &&
                                    (desc.len > 0) {
                                        warn!("net.rs:{} cur addr {:x}, rest len {}, sz {}",
                                            line!(), desc.addr.offset(), desc.len, sz);
                                        continue;
                                }
                                break;
                            }
                            Err(e) => {
                                warn!("net: rx: failed to write slice: {:?}", e);
                                break;
                            }
                        };
                    }
                    unsafe {
                        let cur_memcpy_cycle: usize;
                        asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
                        MID_CYCLE_TOTAL[0] += cur_memcpy_cycle - cycle_start;
                        cycle_start = cur_memcpy_cycle;
                    }
                    unsafe {
                        let cur_memcpy_time: usize;
                        asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
                        MID_TIME_TOTAL[0] += cur_memcpy_time - memcpy_start;
                        MID_CNT_TOTAL[0] += 1;
                        memcpy_start = cur_memcpy_time;
                    }

                    if write_count >= self.rx_count {
                        self.rx_queue
                            .set_used_elem(&self.mem,
                                first_index, io_size as u32,
                                num_buffers);
                        num_buffers += 1;
                        unsafe {
                            let cur_memcpy_cycle: usize;
                            asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
                            MID_CYCLE_TOTAL[1] += cur_memcpy_cycle - cycle_start;
                            cycle_start = cur_memcpy_cycle;
                        }
                        unsafe {
                            let cur_memcpy_time: usize;
                            asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
                            MID_TIME_TOTAL[1] += cur_memcpy_time - memcpy_start;
                            MID_CNT_TOTAL[1] += 1;
                            memcpy_start = cur_memcpy_time;
                        }
                        break;
                    }
                    unsafe {
                        let cur_memcpy_cycle: usize;
                        asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
                        MID_CYCLE_TOTAL[2] += cur_memcpy_cycle - cycle_start;
                        cycle_start = cur_memcpy_cycle;
                    }
                    unsafe {
                        let cur_memcpy_time: usize;
                        asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
                        MID_TIME_TOTAL[2] += cur_memcpy_time - memcpy_start;
                        MID_CNT_TOTAL[2] += 1;
                        memcpy_start = cur_memcpy_time;
                    }
                    
                    if !desc.has_next() {
                        unsafe {
                            let cur_memcpy_cycle: usize;
                            asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
                            MID_CYCLE_TOTAL[3] += cur_memcpy_cycle - cycle_start;
                            cycle_start = cur_memcpy_cycle;
                        }
                        unsafe {
                            let cur_memcpy_time: usize;
                            asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
                            MID_TIME_TOTAL[3] += cur_memcpy_time - memcpy_start;
                            MID_CNT_TOTAL[3] += 1;
                            memcpy_start = cur_memcpy_time;
                        }
                        self.rx_queue
                            .set_used_elem(&self.mem,
                                desc.index, io_size as u32,
                                num_buffers);
                        num_buffers += 1;
                        io_size = 0;
                        unsafe {
                            let cur_memcpy_cycle: usize;
                            asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
                            MID_CYCLE_TOTAL[4] += cur_memcpy_cycle - cycle_start;
                            cycle_start = cur_memcpy_cycle;
                        }
                        unsafe {
                            let cur_memcpy_time: usize;
                            asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
                            MID_TIME_TOTAL[4] += cur_memcpy_time - memcpy_start;
                            MID_CNT_TOTAL[4] += 1;
                            memcpy_start = cur_memcpy_time;
                        }
                        //next_desc = self.rx_queue.iter(&self.mem).next();
                        unsafe {
                            asm!("nop");
                            asm!("nop");
                        }
                        let mut iter = self.rx_queue.iter(&self.mem);
                        unsafe {
                            asm!("nop");
                            asm!("nop");
                        }
                        unsafe {
                            let cur_memcpy_cycle: usize;
                            asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
                            MID_CYCLE_TOTAL[5] += cur_memcpy_cycle - cycle_start;
                            cycle_start = cur_memcpy_cycle;
                        }
                        unsafe {
                            let cur_memcpy_time: usize;
                            asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
                            MID_TIME_TOTAL[5] += cur_memcpy_time - memcpy_start;
                            MID_CNT_TOTAL[5] += 1;
                            memcpy_start = cur_memcpy_time;
                        }
                        unsafe {
                            asm!("nop");
                            asm!("nop");
                            asm!("nop");
                        }
                        next_desc = iter.next();
                        unsafe {
                            asm!("nop");
                            asm!("nop");
                            asm!("nop");
                        }
                        if next_desc.is_none() {
                            unsafe {
                                let cur_memcpy_cycle: usize;
                                asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
                                MID_CYCLE_TOTAL[6] += cur_memcpy_cycle - cycle_start;
                                cycle_start = cur_memcpy_cycle;
                            }
                            unsafe {
                                let cur_memcpy_time: usize;
                                asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
                                MID_TIME_TOTAL[6] += cur_memcpy_time - memcpy_start;
                                MID_CNT_TOTAL[6] += 1;
                                memcpy_start = cur_memcpy_time;
                            }
                            break;
                        }
                        unsafe {
                            let cur_memcpy_cycle: usize;
                            asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
                            MID_CYCLE_TOTAL[7] += cur_memcpy_cycle - cycle_start;
                            cycle_start = cur_memcpy_cycle;
                        }
                        unsafe {
                            let cur_memcpy_time: usize;
                            asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
                            MID_TIME_TOTAL[7] += cur_memcpy_time - memcpy_start;
                            MID_CNT_TOTAL[7] += 1;
                            memcpy_start = cur_memcpy_time;
                        }
                        first_index = next_desc.as_ref().unwrap().index;
                    } else {
                        next_desc = desc.next_descriptor();
                        unsafe {
                            let cur_memcpy_cycle: usize;
                            asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
                            MID_CYCLE_TOTAL[8] += cur_memcpy_cycle - cycle_start;
                            cycle_start = cur_memcpy_cycle;
                        }
                        unsafe {
                            let cur_memcpy_time: usize;
                            asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
                            MID_TIME_TOTAL[8] += cur_memcpy_time - memcpy_start;
                            MID_CNT_TOTAL[8] += 1;
                            memcpy_start = cur_memcpy_time;
                        }
                    }
                }
                None => {
                    warn!(
                        "net: rx: buffer is too small to hold frame of size {}, write_count {}, num_buffers {}",
                        self.rx_count, write_count, num_buffers
                    );
                    break;
                }
            }
        }
        unsafe {
            let cur_memcpy_cycle: usize;
            asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
            MID_CYCLE_TOTAL[9] += cur_memcpy_cycle - cycle_start;
            cycle_start = cur_memcpy_cycle;
        }
        unsafe {
            let cur_memcpy_time: usize;
            asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
            MID_TIME_TOTAL[9] += cur_memcpy_time - memcpy_start;
            MID_CNT_TOTAL[9] += 1;
            memcpy_start = cur_memcpy_time;
        }

        self.net_fix_rx_hdr(&self.mem, head_index, num_buffers);
        unsafe {
            let cur_memcpy_cycle: usize;
            asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
            MID_CYCLE_TOTAL[10] += cur_memcpy_cycle - cycle_start;
            cycle_start = cur_memcpy_cycle;
        }
        unsafe {
            let cur_memcpy_time: usize;
            asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
            MID_TIME_TOTAL[10] += cur_memcpy_time - memcpy_start;
            MID_CNT_TOTAL[10] += 1;
            memcpy_start = cur_memcpy_time;
        }

        self.rx_queue
            .update_used_idx(&self.mem, num_buffers);
        unsafe {
            let cur_memcpy_cycle: usize;
            asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
            MID_CYCLE_TOTAL[11] += cur_memcpy_cycle - cycle_start;
            cycle_start = cur_memcpy_cycle;
        }
        unsafe {
            let cur_memcpy_time: usize;
            asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
            MID_TIME_TOTAL[11] += cur_memcpy_time - memcpy_start;
            MID_CNT_TOTAL[11] += 1;
            memcpy_start = cur_memcpy_time;
        }

        // Interrupt the guest immediately for received frames to
        // reduce latency.
        if self.rx_queue.should_signal(&self.mem) {
            self.signal_used_queue(3);
            unsafe {
                let cur_memcpy_cycle: usize;
                asm!("csrr {}, 0xC00", out(reg) cur_memcpy_cycle);
                MID_CYCLE_TOTAL[12] += cur_memcpy_cycle - cycle_start;
                cycle_start = cur_memcpy_cycle;
            }
            unsafe {
                let cur_memcpy_time: usize;
                asm!("csrr {}, 0xC01", out(reg) cur_memcpy_time);
                MID_TIME_TOTAL[12] += cur_memcpy_time - memcpy_start;
                MID_CNT_TOTAL[12] += 1;
                memcpy_start = cur_memcpy_time;
            }
        }

        unsafe {
            let cycle: usize;
            let time: usize;
            asm!("csrr {}, 0xC00", out(reg) cycle);
            asm!("csrr {}, 0xC01", out(reg) time);
            RX_CYCLE_TOTAL += (time - RX_CYCLE_START);
            RX_TIME_TOTAL += (time - RX_TIME_START);
            RX_LEN_TOTAL += write_count;
            if RX_LEN_TOTAL > (200 << 20) {
                warn!("--- RX_LEN_TOTAL {}, RX_TIME_TOTAL {}, avg {}\n \
                    RX_CYCLE_TOTAL {}, avg {}, freq {}\n \
                    mid_time {}, {}, {}, {}\n \
                    \t\t {}, {}, {}, {}\n \
                    \t\t {}, {}, {}, {}\n \
                    \t\t {}\n \
                    mid_cycle {}, {}, {}, {}\n \
                    \t\t {}, {}, {}, {}\n \
                    \t\t {}, {}, {}, {}\n \
                    \t\t {}\n \
                    mid_cnt {}, {}, {}, {}\n \
                    \t\t {}, {}, {}, {}\n \
                    \t\t {}, {}, {}, {}\n \
                    \t\t {}\n \
                    \t TX_NOTIFY_CNT {}",
                    RX_LEN_TOTAL, RX_TIME_TOTAL, RX_LEN_TOTAL / RX_TIME_TOTAL,
                    RX_CYCLE_TOTAL, RX_LEN_TOTAL / RX_CYCLE_TOTAL, RX_CYCLE_TOTAL/ RX_TIME_TOTAL,
                    MID_TIME_TOTAL[0], MID_TIME_TOTAL[1], MID_TIME_TOTAL[2], MID_TIME_TOTAL[3],
                    MID_TIME_TOTAL[4], MID_TIME_TOTAL[5], MID_TIME_TOTAL[6], MID_TIME_TOTAL[7],
                    MID_TIME_TOTAL[8], MID_TIME_TOTAL[9], MID_TIME_TOTAL[10], MID_TIME_TOTAL[11],
                    MID_TIME_TOTAL[12],
                    MID_CYCLE_TOTAL[0], MID_CYCLE_TOTAL[1], MID_CYCLE_TOTAL[2], MID_CYCLE_TOTAL[3],
                    MID_CYCLE_TOTAL[4], MID_CYCLE_TOTAL[5], MID_CYCLE_TOTAL[6], MID_CYCLE_TOTAL[7],
                    MID_CYCLE_TOTAL[8], MID_CYCLE_TOTAL[9], MID_CYCLE_TOTAL[10], MID_CYCLE_TOTAL[11],
                    MID_CYCLE_TOTAL[12],

                    MID_CNT_TOTAL[0], MID_CNT_TOTAL[1], MID_CNT_TOTAL[2], MID_CNT_TOTAL[3],
                    MID_CNT_TOTAL[4], MID_CNT_TOTAL[5], MID_CNT_TOTAL[6], MID_CNT_TOTAL[7],
                    MID_CNT_TOTAL[8], MID_CNT_TOTAL[9], MID_CNT_TOTAL[10], MID_CNT_TOTAL[11],
                    MID_CNT_TOTAL[12], TX_NOTIFY_CNT);
                RX_LEN_TOTAL = 0;
                RX_TIME_TOTAL = 0;
                RX_CYCLE_TOTAL = 0;
                TX_NOTIFY_CNT = 0;
                for i in 0..13 {
                    MID_TIME_TOTAL[i] = 0;
                    MID_CYCLE_TOTAL[i] = 0;
                    MID_CNT_TOTAL[i] = 0;
                }
                SharedStat::print_all();
                SharedStat::reset_all();
            }
        }
        if next_desc.is_none() {
            return false;
        } else {
            return true;
        }
    }

    fn process_rx(&mut self) {
        // Read as many frames as possible.
        loop {
            let res = self.tap.read(&mut self.rx_buf);
            match res {
                Ok(count) => {
                    self.rx_count = count;
                    if !self.rx_single_frame() {
                        self.deferred_rx = true;
                        break;
                    }
                }
                Err(e) => {
                    // The tap device is nonblocking, so any error aside from EAGAIN is
                    // unexpected.
                    if e.raw_os_error().unwrap() != EAGAIN {
                        warn!("net: rx: failed to read tap: {:?}", e);
                    }
                    break;
                }
            }
        }
    }

    fn process_tx(&mut self) {
        let mut frame = [0u8; MAX_BUFFER_SIZE];
        let mut used_desc_heads = [0u16; QUEUE_SIZE as usize];
        let mut used_count = 0;

        for avail_desc in self.tx_queue.iter(&self.mem) {
            let head_index = avail_desc.index;
            let mut next_desc = Some(avail_desc);
            let mut read_count = 0;

            // Copy buffer from across multiple descriptors.
            loop {
                match next_desc {
                    Some(desc) => {
                        if desc.is_write_only() {
                            break;
                        }
                        let limit = cmp::min(read_count + desc.len as usize, frame.len());
                        //let read_result = self.mem
                        //    .read_slice_at_addr(&mut frame[read_count..limit as usize], desc.addr);
                        //if limit - read_count > 4096 {
                        //    println!("--- {}:{} limit - read_count: {}, res: {:?}",
                        //        limit, read_count, limit - read_count, 
                        //        read_result);
                        //}
                        //match read_result {
                        //    Ok(sz) => {
                        //        read_count += sz;
                        //    }
                        //    Err(e) => {
                        //        warn!("net: tx: failed to read slice: {:?}", e);
                        //        break;
                        //    }
                        //}
                        let mut cur_len: usize = 0;
                        while cur_len < desc.len as usize {
                            let read_result = self.mem
                                .read_slice_at_addr(&mut frame[read_count..limit as usize], desc.addr.unchecked_add(cur_len));
                            //if limit - read_count > 4096 {
                            //    println!("--- {}:{} limit - read_count: {}, res: {:?}",
                            //        limit, read_count, limit - read_count, 
                            //        read_result);
                            //}
                            match read_result {
                                Ok(sz) => {
                                    read_count += sz;
                                    cur_len += sz;
                                }
                                Err(e) => {
                                    warn!("net: tx: failed to read slice: {:?}", e);
                                    break;
                                }
                            }
                        }
                        next_desc = desc.next_descriptor();
                    }
                    None => {
                        break;
                    }
                }
            }

            let write_result = self.tap.write(&frame[..read_count as usize]);
            match write_result {
                Ok(_) => {}
                Err(e) => {
                    warn!("net: tx: error failed to write to tap: {:?}", e);
                }
            };

            used_desc_heads[used_count] = head_index;
            used_count += 1;
        }

        for &desc_index in &used_desc_heads[..used_count] {
            self.tx_queue.add_used(&self.mem, desc_index, 0);
        }

        if self.tx_queue.should_signal(&self.mem) {
            self.signal_used_queue(3);
            unsafe {
                TX_NOTIFY_CNT += 1;
            }
        }
    }
    
    fn run_rx(
        &mut self,
        rx_queue_evt: EventFd,
        tx_queue_evt: EventFd,
        kill_evt: EventFd,
    ) -> Result<(), NetError> {
        let mut poller = Poller::new(4);
        // A frame is available for reading from the tap device to receive in the guest.
        const RX_TAP: u32 = 1;
        // The guest has made a buffer available to receive a frame into.
        const RX_QUEUE: u32 = 2;
        // crosvm has requested the device to shut down.
        const KILL: u32 = 4;

        'poll: loop {
            let tokens = match poller.poll(&[
                (RX_TAP, &self.tap as &dyn Pollable),
                (RX_QUEUE, &rx_queue_evt as &dyn Pollable),
                (KILL, &kill_evt as &dyn Pollable),
            ]) {
                Ok(v) => v,
                Err(e) => return Err(NetError::PollError(e)),
            };

            for &token in tokens {
                match token {
                    RX_TAP => {
                        // Process a deferred frame first if available. Don't read from tap again
                        // until we manage to receive this deferred frame.
                        if self.deferred_rx {
                            if self.rx_single_frame() {
                                self.deferred_rx = false;
                            } else {
                                continue;
                            }
                        }
                        self.process_rx();
                    }
                    RX_QUEUE => {
                        if let Err(e) = rx_queue_evt.read() {
                            error!("net: error reading rx queue EventFd: {:?}", e);
                            break 'poll;
                        }
                        // There should be a buffer available now to receive the frame into.
                        //warn!("net.rs:{} deferred_rx {}", line!(), self.deferred_rx);
                        if self.deferred_rx && self.rx_single_frame() {
                            self.deferred_rx = false;
                        }
                    }
                    KILL => break 'poll,
                    _ => unreachable!(),
                }
            }
        }
        Ok(())
    }

    fn run_tx(
        &mut self,
        rx_queue_evt: EventFd,
        tx_queue_evt: EventFd,
        kill_evt: EventFd,
    ) -> Result<(), NetError> {
        let mut poller = Poller::new(4);
        // The transmit queue has a frame that is ready to send from the guest.
        const TX_QUEUE: u32 = 3;
        // crosvm has requested the device to shut down.
        const KILL: u32 = 4;

        'poll: loop {
            let tokens = match poller.poll(&[
                (TX_QUEUE, &tx_queue_evt as &dyn Pollable),
                (KILL, &kill_evt as &dyn Pollable),
            ]) {
                Ok(v) => v,
                Err(e) => return Err(NetError::PollError(e)),
            };

            for &token in tokens {
                match token {
                    TX_QUEUE => {
                        if let Err(e) = tx_queue_evt.read() {
                            error!("net: error reading tx queue EventFd: {:?}", e);
                            break 'poll;
                        }
                        self.process_tx();
                    }
                    KILL => break 'poll,
                    _ => unreachable!(),
                }
            }
        }
        Ok(())
    }
}

pub struct Net {
    workers_kill_evt: Option<EventFd>,
    kill_evt: EventFd,
    tap: Option<Tap>,
    avail_features: u64,
    acked_features: u64,
}

impl Net {
    /// Create a new virtio network device with the given IP address and
    /// netmask.
    pub fn new(ip_addr: Ipv4Addr, netmask: Ipv4Addr, vmtap_name: String) -> Result<Net, NetError> {
        let kill_evt = EventFd::new().map_err(NetError::CreateKillEventFd)?;

        let tap = Tap::new(vmtap_name).map_err(NetError::TapOpen)?;
        tap.set_ip_addr(ip_addr).map_err(NetError::TapSetIp)?;
        tap.set_netmask(netmask).map_err(NetError::TapSetNetmask)?;

        // Set offload flags to match the virtio features below.
        tap.set_offload(
            net_sys::TUN_F_CSUM | net_sys::TUN_F_UFO | net_sys::TUN_F_TSO4 | net_sys::TUN_F_TSO6,
        ).map_err(NetError::TapSetOffload)?;

        let vnet_hdr_size = mem::size_of::<virtio_net::virtio_net_hdr_v1>() as i32;
        tap.set_vnet_hdr_size(vnet_hdr_size)
            .map_err(NetError::TapSetVnetHdrSize)?;

        tap.enable().map_err(NetError::TapEnable)?;

        let avail_features = 1 << virtio_net::VIRTIO_NET_F_GUEST_CSUM
            | 1 << virtio_net::VIRTIO_NET_F_CSUM
            | 1 << virtio_net::VIRTIO_NET_F_GUEST_TSO4
            | 1 << virtio_net::VIRTIO_NET_F_GUEST_UFO
            | 1 << virtio_net::VIRTIO_NET_F_HOST_TSO4
            | 1 << virtio_net::VIRTIO_NET_F_HOST_UFO
            | 1 << virtio_net::VIRTIO_NET_F_MRG_RXBUF
            | 1 << virtio_net::VIRTIO_F_VERSION_1;

        Ok(Net {
            workers_kill_evt: Some(kill_evt.try_clone().map_err(NetError::CloneKillEventFd)?),
            kill_evt: kill_evt,
            tap: Some(tap),
            avail_features: avail_features,
            acked_features: 0u64,
        })
    }
}

impl Drop for Net {
    fn drop(&mut self) {
        // Only kill the child if it claimed its eventfd.
        if self.workers_kill_evt.is_none() {
            // Ignore the result because there is nothing we can do about it.
            let _ = self.kill_evt.write(1);
        }
    }
}

impl VirtioDevice for Net {
    fn device_type(&self) -> u32 {
        TYPE_NET
    }

    fn queue_max_sizes(&self) -> &[u16] {
        QUEUE_SIZES
    }

    fn features(&self, page: u32) -> u32 {
        match page {
            0 => self.avail_features as u32,
            1 => (self.avail_features >> 32) as u32,
            _ => {
                warn!("net: virtio net got request for features page: {}", page);
                0u32
            }
        }
    }

    fn ack_features(&mut self, page: u32, value: u32) {
        let mut v = match page {
            0 => value as u64,
            1 => (value as u64) << 32,
            _ => {
                warn!(
                    "net: virtio net device cannot ack unknown feature page: {}",
                    page
                );
                0u64
            }
        };

        // Check if the guest is ACK'ing a feature that we didn't claim to have.
        let unrequested_features = v & !self.avail_features;
        if unrequested_features != 0 {
            warn!("net: virtio net got unknown feature ack: {:x}", v);

            // Don't count these features as acked.
            v &= !unrequested_features;
        }
        self.acked_features |= v;
        warn!("--- acked_features 0x{:x}", self.acked_features);
    }

    fn activate(
        &mut self,
        mem: GuestMemory,
        interrupt_evt: EventFd,
        status: Arc<AtomicUsize>,
        mut queues: Vec<Queue>,
        mut queue_evts: Vec<EventFd>,
        irqchip: Arc<dyn IrqChip>,
    ) {
        if queues.len() != 2 || queue_evts.len() != 2 {
            error!("net: expected 2 queues, got {}", queues.len());
            return;
        }

        if let Some(tap) = self.tap.take() {
            if let Some(kill_evt) = self.workers_kill_evt.take() {
                let acked_features = self.acked_features;
                let mem_clone = mem.clone();
                let tap_clone = tap.clone();
                let status_clone = status.clone();
                let interrupt_evt_clone = interrupt_evt.try_clone().unwrap();
                let irqchip_clone = irqchip.clone();
                // First queue is rx, second is tx.
                let rx_queue = queues.remove(0);
                let tx_queue = Queue::new(0);
                let rx_queue_evt = queue_evts.remove(0);
                let tx_queue_evt = EventFd::new().unwrap();
                let rx_kill_evt = kill_evt.try_clone().unwrap();
                let rx_worker_result = thread::Builder::new().name("virtio_net_rx".to_string()).spawn(
                    move || {
                        core_affinity::set_for_current(core_affinity::CoreId {id: 2});
                        let mut worker = Worker {
                            mem: mem_clone,
                            rx_queue: rx_queue,
                            tx_queue: tx_queue,
                            tap: tap_clone,
                            interrupt_status: status_clone,
                            interrupt_evt: interrupt_evt_clone,
                            rx_buf: [0u8; MAX_BUFFER_SIZE],
                            rx_count: 0,
                            deferred_rx: false,
                            acked_features: acked_features,
                            irqchip: irqchip_clone,
                        };
                        let result = worker.run_rx(rx_queue_evt, tx_queue_evt, rx_kill_evt);
                        if let Err(e) = result {
                            error!("net worker thread exited with error: {:?}", e);
                        }
                    },
                );

                if let Err(e) = rx_worker_result {
                    error!("failed to spawn virtio_net worker: {}", e);
                    return;
                }
                
                // First queue is rx, second is tx.
                let rx_queue = Queue::new(0);
                let tx_queue = queues.remove(0);
                let rx_queue_evt = EventFd::new().unwrap();
                let tx_queue_evt = queue_evts.remove(0);
                let tx_worker_result = thread::Builder::new().name("virtio_net_tx".to_string()).spawn(
                    move || {
                        core_affinity::set_for_current(core_affinity::CoreId {id: 3});
                        let mut worker = Worker {
                            mem: mem,
                            rx_queue: rx_queue,
                            tx_queue: tx_queue,
                            tap: tap,
                            interrupt_status: status,
                            interrupt_evt: interrupt_evt,
                            rx_buf: [0u8; MAX_BUFFER_SIZE],
                            rx_count: 0,
                            deferred_rx: false,
                            acked_features: acked_features,
                            irqchip: irqchip,
                        };
                        let result = worker.run_tx(rx_queue_evt, tx_queue_evt, kill_evt);
                        if let Err(e) = result {
                            error!("net worker thread exited with error: {:?}", e);
                        }
                    },
                );

                if let Err(e) = tx_worker_result {
                    error!("failed to spawn virtio_net worker: {}", e);
                    return;
                }
            }
        }
    }
}
