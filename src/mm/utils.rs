
pub const PAGE_SIZE_SHIFT: u64 = 12;
pub const PAGE_TABLE_REGION_SIZE: u64 = 1u64 << 25; // 32MB for now
pub const PAGE_SIZE: u64 = 1u64 << PAGE_SIZE_SHIFT;
pub const PAGE_SIZE_MASK: u64 = PAGE_SIZE - 1;
pub const PAGE_SHIFT: u64 = 12;
pub const PAGE_ORDER: u64 = 9;

pub fn page_size_round_up(length: u64) -> u64 {
    println!("length 0x{:x}", length);
    if length & PAGE_SIZE_MASK == 0 {
        return length;
    }

    let result: u64 = (length & !PAGE_SIZE_MASK) + PAGE_SIZE;
    println!("result 0x{:x}", result);

    result
}
