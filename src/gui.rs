use crate::gui_3d::{HologramRenderer, Mesh3D};
use crate::net::onion;
use crate::telemetry;
use crate::vga_buffer::{Color, Rect, WRITER};
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::Ordering;

pub fn init_ui() {}

#[derive(Default, PartialEq)]
struct UiState {
    cpu_load: u32,
    memory_used: usize,
    synaptic_density: usize,
    onion_hops: usize,
    onion_status: String,
    last_logs: Vec<String>,
}

fn draw_static_elements(writer: &mut crate::vga_buffer::FramebufferWriter) {
    let info = writer.get_info();
    let width = info.width;
    let height = info.height;

    let cyan = Color {
        r: 0,
        g: 255,
        b: 255,
    };
    let white = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    let blue_border = Color {
        r: 0,
        g: 180,
        b: 255,
    };
    let header_bg = Color {
        r: 0,
        g: 80,
        b: 150,
    };
    let footer_text = Color {
        r: 0,
        g: 200,
        b: 200,
    };

    crate::serial_println!("GUI: Drawing static interface...");

    // Main Border
    writer.draw_rect(
        Rect {
            x: 5,
            y: 5,
            width: width - 10,
            height: height - 10,
        },
        blue_border,
    );

    // Header
    writer.fill_rect(
        Rect {
            x: 20,
            y: 20,
            width: 200,
            height: 30,
        },
        header_bg,
    );
    writer.write_string_at(30, 30, "J.A.R.V.I.S. OS v0.1.0", white);

    // Box Borders
    writer.draw_rect(
        Rect {
            x: 20,
            y: 70,
            width: 300,
            height: 150,
        },
        blue_border,
    );
    writer.write_string_at(30, 80, "[ SYSTEM TELEMETRY ]", cyan);

    writer.draw_rect(
        Rect {
            x: 20,
            y: 230,
            width: 300,
            height: 120,
        },
        blue_border,
    );
    writer.write_string_at(30, 240, "[ SYSTEM LOG ]", cyan);

    writer.draw_rect(
        Rect {
            x: 20,
            y: 360,
            width: 300,
            height: 100,
        },
        blue_border,
    );
    writer.write_string_at(
        30,
        370,
        "[ SECURE NETWORK ]",
        Color {
            r: 255,
            g: 255,
            b: 0,
        },
    );

    writer.draw_rect(
        Rect {
            x: 20,
            y: 470,
            width: 300,
            height: height - 490,
        },
        blue_border,
    );
    writer.write_string_at(
        30,
        480,
        "[ BRAIN CORE ]",
        Color {
            r: 180,
            g: 0,
            b: 255,
        },
    );

    writer.draw_rect(
        Rect {
            x: 340,
            y: 70,
            width: width - 360,
            height: height - 150,
        },
        blue_border,
    );
    writer.write_string_at(350, 80, "[ 3D RESOURCE LOAD ]", cyan);

    // Footer
    writer.write_string_at(
        20,
        height - 40,
        "STATUS: SYSTEM CORE ONLINE | ENCRYPTION: ACTIVE | HIVE: SYNCHRONIZED",
        footer_text,
    );
}

pub async fn ui_task() {
    let mut angle: f32 = 0.0;
    let mut initialized = false;
    let mut last_state = UiState::default();

    crate::serial_println!("GUI: UI task started.");

    loop {
        if !initialized {
            if let Some(mut writer_guard) = WRITER.try_lock() {
                if let Some(writer) = writer_guard.as_mut() {
                    writer.clear();
                    draw_static_elements(writer);
                    initialized = true;
                    crate::serial_println!("GUI: Static elements drawn.");
                }
            }
        }

        if initialized {
            let current_state = capture_state();
            update_dynamic_elements(&current_state, &last_state, angle);
            last_state = current_state;
        }

        angle += 0.05;
        if angle > core::f32::consts::TAU {
            angle = 0.0;
        }

        crate::task::yield_now().await;
    }
}

fn capture_state() -> UiState {
    let (used, _) = crate::allocator::heap_usage();
    let onion_status = match *onion::SUBSYSTEM.status.lock() {
        onion::OnionStatus::Disconnected => "OFFLINE",
        onion::OnionStatus::SocksGreeting | onion::OnionStatus::SocksConnect => "HANDSHAKING",
        onion::OnionStatus::CircuitEstablished => "ACTIVE",
        onion::OnionStatus::Error => "ERROR",
    };

    UiState {
        cpu_load: 5, // Simulated base load
        memory_used: used,
        synaptic_density: crate::storage::brain::CORE
            .synaptic_density
            .load(Ordering::SeqCst),
        onion_hops: onion::ACTIVE_HOPS.load(Ordering::SeqCst),
        onion_status: String::from(onion_status),
        last_logs: telemetry::SYSTEM_LOG.lock().clone(),
    }
}

fn update_dynamic_elements(state: &UiState, last_state: &UiState, angle: f32) {
    let mut writer_guard = WRITER.lock();
    if let Some(writer) = writer_guard.as_mut() {
        let black = Color { r: 0, g: 0, b: 0 };
        let white = Color {
            r: 255,
            g: 255,
            b: 255,
        };
        let green_text = Color {
            r: 0,
            g: 255,
            b: 150,
        };

        // 1. System Telemetry (Only redraw if changed)
        if state.cpu_load != last_state.cpu_load || state.memory_used != last_state.memory_used {
            writer.fill_rect(
                Rect {
                    x: 30,
                    y: 110,
                    width: 280,
                    height: 40,
                },
                black,
            );
            writer.write_string_at(40, 110, "CPU LOAD:", green_text);
            writer.write_string_at(
                150,
                110,
                if state.cpu_load > 0 { "BUSY" } else { "IDLE" },
                white,
            );
            writer.write_string_at(40, 130, "MEM USED:", green_text);
            writer.write_string_at(150, 130, "ALLOCATED", white);
        }

        // 2. System Log (Only redraw if logs changed)
        if state.last_logs != last_state.last_logs {
            writer.fill_rect(
                Rect {
                    x: 30,
                    y: 260,
                    width: 280,
                    height: 80,
                },
                black,
            );
            let mut log_y = 260;
            for msg in state.last_logs.iter() {
                let display_msg = if msg.len() > 32 { &msg[0..32] } else { msg };
                writer.write_string_at(40, log_y, display_msg, white);
                log_y += 10;
                if log_y > 340 {
                    break;
                }
            }
        }

        // 3. Secure Network (Only redraw if status or hops changed)
        if state.onion_status != last_state.onion_status
            || state.onion_hops != last_state.onion_hops
        {
            writer.fill_rect(
                Rect {
                    x: 30,
                    y: 400,
                    width: 280,
                    height: 40,
                },
                black,
            );
            writer.write_string_at(
                40,
                400,
                &alloc::format!("STATUS: {}", state.onion_status),
                Color {
                    r: 200,
                    g: 200,
                    b: 0,
                },
            );
            writer.write_string_at(
                40,
                420,
                &alloc::format!("ACTIVE HOPS: {}", state.onion_hops),
                green_text,
            );
        }

        // 4. Brain Core (Only redraw if density changed)
        if state.synaptic_density != last_state.synaptic_density {
            writer.fill_rect(
                Rect {
                    x: 30,
                    y: 510,
                    width: 280,
                    height: 40,
                },
                black,
            );
            writer.write_string_at(
                40,
                510,
                &alloc::format!("DENSITY: {} SYNAPSES", state.synaptic_density),
                green_text,
            );
            writer.write_string_at(
                40,
                530,
                "HIVE: STABLE",
                Color {
                    r: 180,
                    g: 0,
                    b: 255,
                },
            );
        }

        // 5. 3D Resource Load (Always redraw because of rotation, but tightly cleared)
        let renderer = HologramRenderer::new(writer.get_info().width, writer.get_info().height);
        writer.fill_rect(
            Rect {
                x: 345,
                y: 90,
                width: writer.get_info().width - 370,
                height: writer.get_info().height - 150,
            },
            black,
        );

        // Bind bars to real metrics
        let metrics = [
            (state.cpu_load as f32 / 10.0).max(0.2),
            (state.memory_used as f32 / (32.0 * 1024.0 * 1024.0)).clamp(0.1, 2.0),
            (state.synaptic_density as f32 / 100.0).clamp(0.1, 2.0),
            (state.onion_hops as f32 / 5.0).clamp(0.1, 2.0),
        ];

        for (i, &val) in metrics.iter().enumerate() {
            let mesh = Mesh3D::new_bar(0.4, val, 0.4, green_text);
            renderer.render_mesh(
                writer,
                &mesh,
                0.2,
                angle * 0.2,
                (writer.get_info().width as isize / 4) + 50 + (i as isize * 60),
                30,
            );
        }
    }
}
