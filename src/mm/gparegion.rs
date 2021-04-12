pub struct GpaRegion {
    pub gpa: u64,
    pub hpa: u64,
    pub length: u64,
}

impl GpaRegion {
    pub fn new(gpa: u64, hpa: u64, length: u64) -> GpaRegion {
        GpaRegion {
            gpa,
            hpa,
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
        let gpa = 0x4000;
        let hpa = 0x5000;
        let length = 0x1000;
        let gpa_region = GpaRegion::new(gpa, hpa, length);

        assert_eq!(gpa_region.gpa, gpa);
        assert_eq!(gpa_region.hpa, hpa);
        assert_eq!(gpa_region.length, length);
    }
}