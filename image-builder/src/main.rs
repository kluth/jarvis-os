use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use std::process::{Command, exit, Stdio};
use std::io::{BufRead, BufReader};

// Explicitly import from std to avoid prelude issues in no_std/std hybrid environments
use std::option::Option::{Some, None};
use std::result::Result::{Ok, Err};

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<String>>();
    let mut kernel_arg = None;
    let mut machine = "q35".to_string();
    let mut memory = "512".to_string();
    let mut no_run = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--machine" => {
                if i + 1 < args.len() {
                    machine = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Missing value for --machine");
                    exit(1);
                }
            }
            "--memory" => {
                if i + 1 < args.len() {
                    memory = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Missing value for --memory");
                    exit(1);
                }
            }
            "--no-run" => {
                no_run = true;
                i += 1;
            }
            arg if !arg.starts_with("--") && kernel_arg.is_none() => {
                kernel_arg = Some(arg.to_string());
                i += 1;
            }
            _ => i += 1,
        }
    }
    
    let kernel_path_buf = if let Some(arg) = kernel_arg {
        PathBuf::from(arg)
    } else {
        // Default kernel path for standard builds
        PathBuf::from("../target/x86_64-jarvis_os/debug/jarvis-kernel")
    };
    
    let is_test = !no_run && kernel_path_buf.to_str().map(|s| s.contains("debug")).unwrap_or(false);
    
    let kernel_path = kernel_path_buf.as_path();
    let out_dir = Path::new("../target/image");
    
    let image_name = if no_run { "jarvis-os.img" } else { "test-os.img" };
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

    if !no_run {
        println!("Running Test in QEMU (machine: {}, memory: {}M, 60s timeout)...", machine, memory);
        let mut qemu = Command::new("qemu-system-x86_64")
            .arg("-drive")
            .arg(format!("format=raw,file={}", image_path.display()))
            .arg("-device")
            .arg("isa-debug-exit,iobase=0xf4,iosize=0x04")
            .arg("-machine")
            .arg(machine)
            .arg("-m")
            .arg(memory)
            .arg("-nographic")
            .arg("-serial")
            .arg("mon:stdio")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start QEMU");

        let stdout = qemu.stdout.take().expect("Failed to open QEMU stdout");
        let stderr = qemu.stderr.take().expect("Failed to open QEMU stderr");

        // Real-time output handling
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    println!("[QEMU STDOUT] {}", l);
                }
            }
        });

        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(l) = line {
                    eprintln!("[QEMU STDERR] {}", l);
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
                            if code == 33 {
                                println!("Test Passed (raw code 33)!");
                                exit(0);
                            }
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
