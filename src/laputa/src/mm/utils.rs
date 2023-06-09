pub const PAGE_SIZE_SHIFT: u64 = 12;
pub const PAGE_TABLE_REGION_SIZE: u64 = 32 << MB_SHIFT; /* 32MB for now */
pub const PAGE_SIZE: u64 = 1u64 << PAGE_SIZE_SHIFT;
pub const PAGE_SIZE_MASK: u64 = PAGE_SIZE - 1;
pub const PAGE_SHIFT: u64 = 12;
pub const PAGE_ORDER: u64 = 9;
pub const KB_SHIFT: u64 = 10;
pub const MB_SHIFT: u64 = 2 * KB_SHIFT;
#[allow(unused)]
pub const GB_SHIFT: u64 = 3 * KB_SHIFT;
#[allow(unused)]
pub const TB_SHIFT: u64 = 4 * KB_SHIFT;

pub macro_rules! dbgprintln {
    () => {
        #[cfg(test)]
        print!("\n");
    };
    ($fmt: expr) => {
        #[cfg(test)]
        print!(concat!($fmt, "\n"));
    };
    ($fmt: expr, $($arg:tt)*) => {
        #[cfg(test)]
        print!(concat!($fmt, "\n"), $($arg)*);
    };
}

pub macro_rules! print_flush {
    ( $($t:tt)* ) => {
        {
            let mut h = io::stdout();
            write!(h, $($t)* ).unwrap();
            h.flush().unwrap();
        }
    }
}

pub fn page_size_round_up(length: u64) -> u64 {
    if length & PAGE_SIZE_MASK == 0 {
        return length;
    }

    let result: u64 = (length & !PAGE_SIZE_MASK) + PAGE_SIZE;

    result
}
