use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{exit, Command, Stdio};
use std::thread;
use std::time::Duration;

// Explicitly import from std to avoid prelude issues in no_std/std hybrid environments
use std::option::Option::{None, Some};
use std::result::Result::{Err, Ok};

fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();
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
        PathBuf::from("../target/x86_64-jarvis_os/debug/jarvis-kernel")
    };

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
    let boot = bootloader::BiosBoot::new(kernel_path);
    if let Err(e) = boot.create_disk_image(&image_path) {
        eprintln!("Failed to create disk image: {}", e);
        exit(1);
    }
    println!("Success: Disk image created at {}", image_path.display());

    if !no_run {
        let qmp_socket = "/tmp/qmp-sock";
        let screenshot_path = "../target/screenshot.ppm";

        // Remove old socket if exists
        let _ = fs::remove_file(qmp_socket);

        println!(
            "Running Test in QEMU (machine: {}, memory: {}M, 120s timeout)...",
            machine, memory
        );
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
            .arg("-qmp")
            .arg(format!("unix:{},server,nowait", qmp_socket))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start QEMU");

        let stdout = qemu.stdout.take().expect("Failed to open QEMU stdout");
        let stderr = qemu.stderr.take().expect("Failed to open QEMU stderr");

        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    println!("[QEMU STDOUT] {}", l);
                }
            }
        });

        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(l) = line {
                    eprintln!("[QEMU STDERR] {}", l);
                }
            }
        });

        // Trigger screenshot via QMP after some time
        thread::spawn(move || {
            println!("QMP Thread: Waiting for boot window (15s)...");
            thread::sleep(Duration::from_secs(15));
            
            let mut retry_count = 0;
            let mut connected = false;
            let mut stream = None;
            
            while retry_count < 10 {
                println!("QMP Thread: Connecting to QMP (attempt {})...", retry_count + 1);
                match UnixStream::connect(qmp_socket) {
                    Ok(s) => {
                        println!("QMP Thread: Connected!");
                        stream = Some(s);
                        connected = true;
                        break;
                    }
                    Err(e) => {
                        eprintln!("QMP Thread: Connection failed: {}. Retrying in 1s...", e);
                        thread::sleep(Duration::from_secs(1));
                        retry_count += 1;
                    }
                }
            }

            if let (true, Some(mut s)) = (connected, stream) {
                let _ = s.write_all(b"{\"execute\": \"qmp_capabilities\"}\n");
                thread::sleep(Duration::from_millis(500));
                let cmd = format!(
                    "{{\"execute\": \"screendump\", \"arguments\": {{\"filename\": \"{}\"}}}} \n",
                    screenshot_path
                );
                let _ = s.write_all(cmd.as_bytes());
                println!("QMP Thread: Screenshot command sent.");
                // Give it a moment to write the file
                thread::sleep(Duration::from_secs(2));
            } else {
                eprintln!("QMP Thread: Failed to establish QMP connection after retries.");
            }
        });

        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(120);
        let mut test_result = None;

        loop {
            match qemu.try_wait() {
                Ok(Some(status)) => {
                    if test_result.is_none() {
                        test_result = Some(status.code());
                        println!("OS finished execution. Waiting for screenshot window...");
                    }
                }
                Ok(None) => {
                    if start_time.elapsed() > timeout {
                        println!("Test Timed Out! Killing QEMU...");
                        let _ = qemu.kill();
                        exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error waiting for QEMU: {}", e);
                    exit(1);
                }
            }

            // Ensure we run for at least 30 seconds to allow the QMP thread to do its work
            if start_time.elapsed() > Duration::from_secs(30) {
                if let Some(code_opt) = test_result {
                    match code_opt {
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
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}
