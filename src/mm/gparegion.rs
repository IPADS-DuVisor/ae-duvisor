pub struct GpaBlock {
    pub gpa: u64,
    pub hva: u64,
    pub hpa: u64,
    pub length: u64,
}

impl GpaBlock {
    pub fn new(gpa: u64, hva: u64, hpa: u64, length: u64) -> Self {
        Self {
            gpa,
            hva,
            hpa,
            length,
        }
    }
}

pub struct GpaRegion {
    pub gpa: u64,
    pub length: u64,
}

impl GpaRegion {
    pub fn new(gpa: u64, length: u64) -> Self {
        Self {
            gpa,
            length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /* Check new() of GpaBlock */
    #[test]
    fn test_gpa_block_new() {
        let gpa = 0x4000;
        let hva = 0x5000;
        let hpa = 0x5000;
        let length = 0x1000;
        let gpa_block = GpaBlock::new(gpa, hva, hpa, length);

        assert_eq!(gpa_block.gpa, gpa);
        assert_eq!(gpa_block.hva, hva);
        assert_eq!(gpa_block.hpa, hpa);
        assert_eq!(gpa_block.length, length);
    }

    #[test]
    fn test_gpa_region_new() {
        let gpa = 0x8000;
        let length = 0x2000;
        let gpa_region = GpaRegion::new(gpa, length);

        assert_eq!(gpa_region.gpa, gpa);
        assert_eq!(gpa_region.length, length);
    }
}
