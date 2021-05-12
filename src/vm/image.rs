use std::path::PathBuf;

// Read and parse the vm img file
pub struct VmImage {
    pub elf_file: elf::File,
    pub file_data: Vec<u8>,
}

impl VmImage {
    pub fn new(file_path: &str) -> Self {
        // parse ELF file
        let elf_file: elf::File = VmImage::elf_parse(file_path);
        let res = std::fs::read(file_path);

        if res.is_err() {
            panic!("read file failed");
        }

        let file_data = res.unwrap();

        Self {
            elf_file,
            file_data,
        }
    }

    pub fn elf_parse(elf_path: &str) -> elf::File {
        let path = PathBuf::from(elf_path);
        let file = match elf::File::open_path(&path) {
            Ok(f) => f,
            Err(e) => panic!("Error: {:?}", e),
        };

        file
    }
}