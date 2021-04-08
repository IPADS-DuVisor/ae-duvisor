pub struct GpaRegion {
    pub base_address: u64,
    pub length: u64,
}

impl GpaRegion {
    pub fn new(base_address: u64, length: u64) -> GpaRegion {
        GpaRegion {
            base_address,
            length,
        }
    }
}