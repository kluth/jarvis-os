use std::path::PathBuf;

fn main() {
    let kernel_path = std::env::args().nth(1).expect("Usage: build_image <kernel-elf-path>");
    let out_path = std::env::args().nth(2).unwrap_or_else(|| "boot-jarvis-os.img".into());
    
    let kernel_path = PathBuf::from(kernel_path);
    let out_path = PathBuf::from(&out_path);
    
    let builder = bootloader::DiskImageBuilder::new(kernel_path);
    builder.create_bios_image(&out_path).expect("Failed to create boot image");
    
    let len = std::fs::metadata(&out_path).unwrap().len();
    eprintln!("Boot image written: {} ({} bytes)", out_path.display(), len);
}