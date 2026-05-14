use std::path::Path;
use std::fs;

fn main() {
    let kernel_path = Path::new("target/x86_64-jarvis_os/debug/jarvis-kernel");
    let out_dir = Path::new("target/image");
    let image_path = out_dir.join("jarvis-os.img");

    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!("Failed to create output directory: {}", e);
        std::process::exit(1);
    }

    if !kernel_path.exists() {
        eprintln!("Kernel binary not found at {}. Run 'cargo build' first.", kernel_path.display());
        std::process::exit(1);
    }

    println!("Creating disk image at {}...", image_path.display());

    let mut bootloader_config = bootloader::BootConfig::default();
    // In CI we might want to ensure certain offsets or configurations
    
    let mut builder = bootloader::DiskImageBuilder::new(kernel_path.to_path_buf());
    builder.set_output_ext("img");
    
    if let Err(e) = builder.create(&image_path) {
        eprintln!("Failed to create disk image: {}", e);
        std::process::exit(1);
    }

    println!("Success: Disk image created at {}", image_path.display());
}
