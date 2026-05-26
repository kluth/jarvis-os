// Simple script to build bootable image
use bootloader::DiskImageBuilder;
use std::path::Path;

fn main() {
    // Paths
    let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let kernel = target_dir.join("target").join("x86_64-jarvis_os").join("release").join("jarvis-kernel");
    let out_dir = target_dir.join("target").join("image");
    std::fs::create_dir_all(&out_dir).unwrap();
    let image = out_dir.join("jarvis-os.img");

    eprintln!("Using kernel: {}", kernel.display());
    eprintln!("Output image: {}", image.display());

    if !kernel.exists() {
        eprintln!("ERROR: Kernel not found at {}", kernel.display());
        std::process::exit(1);
    }

    match DiskImageBuilder::new(&kernel).create_disk_image(&image) {
        Ok(_) => {
            eprintln!("SUCCESS: Bootable image created at {}", image.display());
        }
        Err(e) => {
            eprintln!("ERROR: Failed to create disk image: {}", e);
            std::process::exit(1);
        }
    }
}