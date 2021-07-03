use crate::mm::utils::*;

pub const DTB_GPA: u64 = 0x82200000;

#[allow(unused)]
pub struct BusRegion {
    offset: u64,
    size: u64,
}

pub const DTB_TARGET_PROP: [&str; 5] = ["memory", "soc", "chosen", "linux,initrd-start",
    "linux,initrd-end"];

impl BusRegion {
    pub fn new(offset: u64, size: u64) -> Self {
        Self {
            offset,
            size,
        }
    }
}

pub struct InitrdRegion {
    pub start: u64,
    pub end: u64,
}

impl InitrdRegion {
    pub fn new() -> Self {
        let start: u64 = 0;
        let end: u64 = 0;

        Self {
            start,
            end,
        }
    }
}

pub struct MachineMeta {
    pub address_cells: Vec<u32>,
    pub size_cells: Vec<u32>,
    pub memory_regions: Vec<BusRegion>,
    pub soc_regions: Vec<BusRegion>,
    pub initrd_region: InitrdRegion,
}

impl MachineMeta {
    const INITRD_START: i32 = 0;
    const INITRD_END: i32 = 1;

    pub fn new() -> Self {
        let address_cells: Vec<u32> = Vec::new();
        let size_cells: Vec<u32> = Vec::new();
        let memory_regions: Vec<BusRegion> = Vec::new();
        let soc_regions: Vec<BusRegion> = Vec::new();
        let initrd_region: InitrdRegion = InitrdRegion::new();

        Self {
            address_cells,
            size_cells,
            memory_regions,
            soc_regions,
            initrd_region,
        }
    }

    pub fn dtb_parse(&mut self, item: &dtb::StructItem, node_path: &Vec<&str>,
        file_path: &str) {
        let mut prop: &str = "";
        let mut match_flag = false;
        let res = std::fs::read(file_path);

        if res.is_err() {
            panic!("read file failed");
        }

        let mut file_data = res.unwrap();

        /* set address-cells */
        if item.name().unwrap().contains("address-cells") {
            self.address_cells.pop();
            self.address_cells.push(item.value_u32_list(&mut file_data).unwrap()[0]);
            dbgprintln!("match address-cells {:?}", self.address_cells);
            return;
        }

        /* set size-cells */
        if item.name().unwrap().contains("size-cells") {
            self.size_cells.pop();
            self.size_cells.push(item.value_u32_list(&mut file_data).unwrap()[0]);
            dbgprintln!("match size-cells {:?}", self.size_cells);
            return;
        }

        /* address_cells and size_cells shall be set first */
        let ac_len = self.address_cells.len();
        let sc_len = self.size_cells.len();
        assert_eq!(ac_len, sc_len);

        let mut address_cells = 2;
        let mut size_cells = 1;

        if ac_len > 1 {
            address_cells = self.address_cells[ac_len - 2];
            size_cells = self.size_cells[sc_len - 2];
        }

        dbgprintln!("cells {} {}", address_cells, size_cells);

        for i in node_path {
            for j in DTB_TARGET_PROP.iter() {
                if i.contains(j) {
                    match_flag = true;
                    dbgprintln!("find {} in {}", j, i);
                    prop = j;
                    break;
                }
            }
        }

        if !match_flag {
            return;
        }

        let prop_name = item.name().unwrap();
        match prop {
            "memory" => {
                dbgprintln!("match memory");
                /* add bus region for memory */
                if  prop_name == "reg" {
                    let values = item.value_u32_list(&mut file_data).unwrap();
                    dbgprintln!("MEMORY REG {:x?}", values);
                    self.memory_parse(values, address_cells, size_cells);
                }
            },
            "soc" => {
                dbgprintln!("match soc");

                if  prop_name == "reg" && node_path.len() == 3 {
                    let values = item.value_u32_list(&mut file_data).unwrap();
                    dbgprintln!("SOC REG {:x?}", values);
                    self.soc_parse(values, address_cells, size_cells);
                }
            },
            "chosen" => {
                dbgprintln!("match chosen");

                if  prop_name == "linux,initrd-start" && node_path.len() == 2 {
                    let values = item.value_u32_list(&mut file_data).unwrap();
                    dbgprintln!("INITRD REG {:x?}", values);
                    self.initrd_parse(values, address_cells, size_cells, MachineMeta::INITRD_START);
                }

                if  prop_name == "linux,initrd-end" && node_path.len() == 2 {
                    let values = item.value_u32_list(&mut file_data).unwrap();
                    dbgprintln!("INITRD REG {:x?}", values);
                    self.initrd_parse(values, address_cells, size_cells, MachineMeta::INITRD_END);
                }
            }
            _ => {
                dbgprintln!("match nothing");
            },
        }
    }

    pub fn soc_parse(&mut self, value_u32_list: &[u32], address_cells: u32, size_cells: u32) {
        dbgprintln!("soc_parse {} {} {:x?}", address_cells, size_cells, value_u32_list);
        let len = value_u32_list.len() as u32;
        let t = address_cells + size_cells;
        let cycle: u32 = len / t;

        assert_eq!(len % t, 0);
        if len == 0 {
            return;
        }

        for i in 0..cycle {
            let mut offset: u64 = 0;
            let mut size: u64 = 0;

            for j in 0..address_cells {
                offset = (offset << 32) + (value_u32_list[(i * t + j) as usize] as u64);
            }

            for k in 0..size_cells {
                size = (size << 32) + (value_u32_list[(i * t + address_cells + k) as usize] as u64);
            }

            /* add memory region */
            self.soc_regions.push(BusRegion::new(offset, size));
            dbgprintln!("add soc region {:x} {:x}", offset, size);
        }
    }

    pub fn initrd_parse(&mut self, value_u32_list: &[u32], address_cells: u32, size_cells: u32, value_type: i32) {
        let mut prop_value: u64 = 0;
        let cells: u32;
        
        if value_type == MachineMeta::INITRD_START {
            cells = address_cells;
        } else {
            cells = size_cells;
        }

        for i in 0..cells {
            prop_value = (prop_value << 32) + (value_u32_list[i as usize] as u64);
        }

        println!("initrd_parse - prop_value 0x{:x}", prop_value);

        if value_type == MachineMeta::INITRD_START {
            self.initrd_region.start = prop_value;
        } else {
            self.initrd_region.end = prop_value;
        }
    }

    pub fn memory_parse(&mut self, value_u32_list: &[u32], address_cells: u32, size_cells: u32) {
        dbgprintln!("memory_parse {:x?}", value_u32_list);
        let len = value_u32_list.len() as u32;
        let t = address_cells + size_cells;
        let cycle: u32 = len / t;

        assert_eq!(len % t, 0);
        if len == 0 {
            return;
        }

        for i in 0..cycle {
            let mut offset: u64 = 0;
            let mut size: u64 = 0;

            for j in 0..address_cells {
                offset = (offset << 32) + (value_u32_list[(i * t + j) as usize] as u64);
            }

            for k in 0..size_cells {
                size = (size << 32) + (value_u32_list[(i * t + address_cells + k) as usize] as u64);
            }

            /* add memory region */
            self.memory_regions.push(BusRegion::new(offset, size));
            dbgprintln!("add memory region {:x} {:x}", offset, size);
        }
    }
}

/* Read and parse the vm img file */
pub struct DeviceTree {
    pub file_data: Vec<u8>,
    pub meta_data: MachineMeta,
}

impl DeviceTree {
    pub fn new(file_path: &str) -> Self {
        let res = std::fs::read(file_path);

        if res.is_err() {
            panic!("read file failed");
        }

        let file_data = res.unwrap();
        let dtb_reader: dtb::Reader = dtb::Reader::read(file_data.as_slice()).unwrap();

        dbgprintln!("dtb_reader success");

        let root = dtb_reader.struct_items();

        /* parse dtb info */
        let mut node_path: Vec<&str> = Vec::new();
        let mut name;
        let mut meta_data = MachineMeta::new();

        for i in root {
            if i.name().is_err() {
                /* endnode */
                dbgprintln!("Endnode or error");
                node_path.pop();
                meta_data.address_cells.pop();
                meta_data.size_cells.pop();
            } else {
                /* node or property */
                name = i.name().unwrap();
                if i.node_name().is_err() {
                    /* property */
                    meta_data.dtb_parse(&i, &node_path, file_path);
                } else {
                    /* node */
                    node_path.push(name);
                    dbgprintln!("Node name: {:?}, name: {}", node_path, name);

                    /* A client should assume a default value of 2 for  */
                    /* #address-cells and a value of 1 for #size-cells */
                    meta_data.address_cells.push(2);
                    meta_data.size_cells.push(1);
                }
            }
        }

        Self {
            file_data,
            meta_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn test_dtb_parse_sifive() {
            /* Compiled from linux/arch/riscv/boot/dts/sifive/ */
            let dtb = DeviceTree::new(
                    "./test-files-laputa/hifive-unleashed-a00.dtb");
            let mut len;
            let mut offset: u64;
            let mut size: u64;

            /* memory info check */
            len = dtb.meta_data.memory_regions.len();
            assert_eq!(len, 1);

            offset = dtb.meta_data.memory_regions[0].offset;
            size = dtb.meta_data.memory_regions[0].size;
            assert_eq!(offset, 0x80000000);
            assert_eq!(size, 0x200000000);

            /* soc info check */
            len = dtb.meta_data.soc_regions.len();
            assert_eq!(len, 17);

            let offset_ans: [u64; 17] = [0xc000000, 0x10000000, 0x10010000,
                0x3000000, 0x10011000, 0x10030000, 0x10040000, 0x20000000,
                0x10041000, 0x30000000, 0x10050000, 0x10090000, 0x100a0000,
                0x10020000, 0x10021000, 0x2010000, 0x10060000];
            let size_ans: [u64; 17] = [0x4000000, 0x1000, 0x1000,
                0x8000, 0x1000, 0x1000, 0x1000, 0x10000000, 0x1000, 0x10000000,
                0x1000, 0x2000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000];

            for i in 0..17 {
                offset = dtb.meta_data.soc_regions[i].offset;
                size = dtb.meta_data.soc_regions[i].size;
                assert_eq!(offset, offset_ans[i]);
                assert_eq!(size, size_ans[i]);
            }
        }

        #[test]
        fn test_dtb_parse_kendryte() {
            /* Compiled from linux/arch/riscv/boot/dts/kendryte/ */
            let dtb = DeviceTree::new("./test-files-laputa/k210.dtb");
            let mut len;
            let mut offset: u64;
            let mut size: u64;

            /* memory info check */
            len = dtb.meta_data.memory_regions.len();
            assert_eq!(len, 3);

            offset = dtb.meta_data.memory_regions[0].offset;
            size = dtb.meta_data.memory_regions[0].size;
            assert_eq!(offset, 0x80000000);
            assert_eq!(size, 0x400000);

            offset = dtb.meta_data.memory_regions[1].offset;
            size = dtb.meta_data.memory_regions[1].size;
            assert_eq!(offset, 0x80400000);
            assert_eq!(size, 0x200000);

            offset = dtb.meta_data.memory_regions[2].offset;
            size = dtb.meta_data.memory_regions[2].size;
            assert_eq!(offset, 0x80600000);
            assert_eq!(size, 0x200000);

            /* soc info check */
            len = dtb.meta_data.soc_regions.len();
            assert_eq!(len, 4);

            offset = dtb.meta_data.soc_regions[0].offset;
            size = dtb.meta_data.soc_regions[0].size;
            assert_eq!(offset, 0x50440000);
            assert_eq!(size, 0x1000);

            offset = dtb.meta_data.soc_regions[1].offset;
            size = dtb.meta_data.soc_regions[1].size;
            assert_eq!(offset, 0x2000000);
            assert_eq!(size, 0xc000);

            offset = dtb.meta_data.soc_regions[2].offset;
            size = dtb.meta_data.soc_regions[2].size;
            assert_eq!(offset, 0xc000000);
            assert_eq!(size, 0x4000000);

            offset = dtb.meta_data.soc_regions[3].offset;
            size = dtb.meta_data.soc_regions[3].size;
            assert_eq!(offset, 0x38000000);
            assert_eq!(size, 0x1000);
        }
    }
}