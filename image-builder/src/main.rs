use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use std::process::{Command, exit, Stdio};
use std::io::{BufRead, BufReader};

// Explicitly import from std to avoid prelude issues in no_std/std hybrid environments
use std::option::Option::{Some, None};
use std::result::Result::{Ok, Err};

fn main() {
    let mut args = env::args().skip(1);
    let first_arg = args.next();
    let is_test = first_arg.is_some();
    
    let kernel_path_buf = if let Some(arg) = first_arg {
        PathBuf::from(arg)
    } else {
        // Default kernel path for standard builds
        PathBuf::from("target/x86_64-jarvis_os/debug/jarvis-kernel")
    };
    
    let kernel_path = kernel_path_buf.as_path();
    let out_dir = Path::new("target/image");
    
    // Use a unique name for test images to avoid conflicts
    let image_name = if is_test {
        format!("test-{}.img", kernel_path.file_name().unwrap().to_str().unwrap())
    } else {
        "jarvis-os.img".to_string()
    };
    let image_path = out_dir.join(image_name);

    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!("Failed to create output directory: {}", e);
        exit(1);
    }

    if !kernel_path.exists() {
        eprintln!("Kernel binary not found at {}.", kernel_path.display());
        exit(1);
    }

    println!("Creating disk image at {}...", image_path.display());

    // Use the bootloader crate to create a bootable disk image
    let boot = bootloader::BiosBoot::new(kernel_path);
    
    if let Err(e) = boot.create_disk_image(&image_path) {
        eprintln!("Failed to create disk image: {}", e);
        exit(1);
    }

    println!("Success: Disk image created at {}", image_path.display());

    if is_test {
        println!("Running Test in QEMU (60s timeout)...");
        let mut qemu = Command::new("qemu-system-x86_64")
            .arg("-drive")
            .arg(format!("format=raw,file={}", image_path.display()))
            .arg("-device")
            .arg("isa-debug-exit,iobase=0xf4,iosize=0x04")
            .arg("-display")
            .arg("none")
            .arg("-serial")
            .arg("stdio")
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to start QEMU");

        let stdout = qemu.stdout.take().expect("Failed to open QEMU stdout");
        let reader = BufReader::new(stdout);

        // Spawn a thread to read and print QEMU output in real-time
        std::thread::spawn(move || {
            for line in reader.lines() {
                if let Ok(l) = line {
                    println!("[QEMU] {}", l);
                }
            }
        });

        // Use a simple polling loop with a timeout for the child process
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(60);
        
        loop {
            match qemu.try_wait() {
                Ok(Some(status)) => {
                    // isa-debug-exit returns (exit_code << 1) | 1.
                    // QemuExitCode::Success (0x10) -> 33
                    // QemuExitCode::Failed (0x11) -> 35
                    match status.code() {
                        Some(33) => {
                            println!("Test Passed!");
                            exit(0);
                        }
                        Some(35) => {
                            eprintln!("Test Failed!");
                            exit(1);
                        }
                        Some(code) => {
                            eprintln!("QEMU exited with unexpected code: {}", code);
                            exit(1);
                        }
                        None => {
                            eprintln!("QEMU was killed by a signal");
                            exit(1);
                        }
                    }
                }
                Ok(None) => {
                    if start_time.elapsed() > timeout {
                        println!("Test Timed Out! Killing QEMU...");
                        let _ = qemu.kill();
                        exit(1);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    eprintln!("Error waiting for QEMU: {}", e);
                    exit(1);
                }
            }
        }
    }
}
