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

    #[test]
    fn test_gpa_region_new() {
        let gpa = 0x8000;
        let length = 0x2000;
        let gpa_region = GpaRegion::new(gpa, length);

        assert_eq!(gpa_region.gpa, gpa);
        assert_eq!(gpa_region.length, length);
    }
}
