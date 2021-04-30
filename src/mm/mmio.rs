use crate::mm::gparegion;

pub struct MmioManager {
    pub gpa_regions: Vec<gparegion::GpaRegion>,
}

impl MmioManager {
    pub fn new() -> Self {
        let gpa_regions: Vec<gparegion::GpaRegion> = Vec::new();

        Self {
            gpa_regions,
        }
    }

    pub fn mmio_add(&mut self, gpa: u64, length: u64) {
        let gpa_region = gparegion::GpaRegion::new(gpa, length);

        self.gpa_regions.push(gpa_region);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Check new() of GpaBlock
    #[test]
    fn test_mmio_add() {
        let mut gpa: u64 = 0;
        let mut length: u64 = 0;
        let gpa_ans = 0x4000;
        let length_ans = 0x1000;
        let mut mmio_manager = MmioManager::new();

        mmio_manager.mmio_add(gpa_ans, length_ans);

        let len = mmio_manager.gpa_regions.len();
        assert_eq!(len, 1);

        for i in mmio_manager.gpa_regions {
            gpa = i.gpa;
            length = i.length;
        }

        assert_eq!(gpa_ans, gpa);
        assert_eq!(length_ans, length);
    }
}