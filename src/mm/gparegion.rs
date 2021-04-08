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

#[cfg(test)]
mod tests {
    use super::*;

    // Check new() of GStageMmu
    #[test]
    fn test_gpa_region_new() { 
        let base_address = 0x4000;
        let length = 0x1000;
        let gpa_region = GpaRegion::new(base_address, length);

        assert_eq!(gpa_region.base_address, base_address);
        assert_eq!(gpa_region.length, length);
    }
}