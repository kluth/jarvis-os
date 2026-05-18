use crate::gui_3d::{HologramRenderer, Mesh3D, Point2D};
use crate::net::onion;
use crate::serial_println;
use crate::telemetry;
use crate::vga_buffer::{Color, Rect, WRITER};

pub fn init_ui() {}

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

    serial_println!("GUI: Screen size: {}x{}", width, height);

    // Main Border
    serial_println!("GUI: Drawing border...");
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
    serial_println!("GUI: Drawing header...");
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

    // Telemetry Box
    serial_println!("GUI: Drawing telemetry box...");
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

    // System Log Box
    serial_println!("GUI: Drawing system log box...");
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

    // Secure Network Box
    serial_println!("GUI: Drawing secure network box...");
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
    ); // Yellow

    // Brain Core Box
    serial_println!("GUI: Drawing brain core box...");
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
    ); // Purple

    // Hologram Box (Right side)
    serial_println!("GUI: Drawing AI shell box...");
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
    serial_println!("GUI: Drawing footer...");
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

    serial_println!("GUI: UI task started.");

    loop {
        if !initialized {
            if let Some(mut writer_guard) = WRITER.try_lock() {
                if let Some(writer) = writer_guard.as_mut() {
                    writer.clear();
                    draw_static_elements(writer);
                    initialized = true;
                    serial_println!("GUI: Static elements drawn.");
                }
            }
        }

        if initialized {
            update_dynamic_elements(angle);
        }

        angle += 0.05;
        if angle > core::f32::consts::TAU {
            angle = 0.0;
        }

        // Small yield to keep UI fluid
        crate::task::yield_now().await;
    }
}

fn update_dynamic_elements(angle: f32) {
    if let Some(mut writer_guard) = WRITER.try_lock() {
        if let Some(writer) = writer_guard.as_mut() {
            let info = writer.get_info();
            let width = info.width;
            let height = info.height;

            let black = Color { r: 0, g: 0, b: 0 };
            let green_text = Color {
                r: 0,
                g: 255,
                b: 150,
            };
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

            // 1. Clear & Update Telemetry Area
            writer.fill_rect(
                Rect {
                    x: 30,
                    y: 110,
                    width: 280,
                    height: 100,
                },
                black,
            );

            let latest = telemetry::HUB.lock().get_latest(4);
            let mut y = 110;
            for data in latest {
                let (text, color) = match data {
                    telemetry::TelemetryData::CpuLoad(load) => {
                        writer.write_string_at(40, y, "CPU LOAD:", green_text);
                        writer.write_string_at(
                            150,
                            y,
                            if load > 0 { "BUSY" } else { "IDLE" },
                            white,
                        );
                        y += 20;
                        continue;
                    }
                    telemetry::TelemetryData::MemoryUsed(_used) => {
                        writer.write_string_at(40, y, "MEM USED:", green_text);
                        // Simple hex display for memory since we don't have full formatting
                        writer.write_string_at(150, y, "ALLOCATED", white);
                        y += 20;
                        continue;
                    }
                    telemetry::TelemetryData::MemoryFree(_) => ("MEMORY: STABLE", green_text),
                    telemetry::TelemetryData::HardwareEvent { device, .. } => {
                        writer.write_string_at(
                            40,
                            y,
                            "HW EVENT:",
                            Color {
                                r: 255,
                                g: 165,
                                b: 0,
                            },
                        ); // Orange
                        writer.write_string_at(150, y, device, white);
                        y += 20;
                        continue;
                    }
                    telemetry::TelemetryData::SystemStatus(s) => (s, green_text),
                    telemetry::TelemetryData::AIIntentDetected(intent) => {
                        writer.write_string_at(40, y, "AI INTENT:", cyan);
                        writer.write_string_at(150, y, intent, white);
                        y += 20;
                        continue;
                    }
                };

                writer.write_string_at(40, y, text, color);
                y += 20;
            }

            // 2. Clear & Update System Log Area
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
            {
                let log = telemetry::SYSTEM_LOG.lock();
                for msg in log.iter() {
                    // Truncate or wrap if needed (simple truncate for now)
                    let display_msg = if msg.len() > 32 { &msg[0..32] } else { msg };
                    writer.write_string_at(40, log_y, display_msg, white);
                    log_y += 10;
                    if log_y > 340 {
                        break;
                    }
                }
            }

            // 3. Clear & Update Secure Network Area
            writer.fill_rect(
                Rect {
                    x: 30,
                    y: 400,
                    width: 280,
                    height: 60,
                },
                black,
            );

            let onion_status = *onion::SUBSYSTEM.status.lock();
            let status_text = match onion_status {
                onion::OnionStatus::Disconnected => "STATUS: OFFLINE",
                onion::OnionStatus::SocksGreeting => "STATUS: HANDSHAKING...",
                onion::OnionStatus::SocksConnect => "STATUS: CONNECTING...",
                onion::OnionStatus::CircuitEstablished => "STATUS: CIRCUIT ACTIVE",
                onion::OnionStatus::Error => "STATUS: ERROR",
            };
            writer.write_string_at(
                40,
                400,
                status_text,
                Color {
                    r: 200,
                    g: 200,
                    b: 0,
                },
            );

            if onion_status == onion::OnionStatus::CircuitEstablished
                || onion_status == onion::OnionStatus::SocksGreeting
                || onion_status == onion::OnionStatus::SocksConnect
            {
                let active_hops = onion::ACTIVE_HOPS.load(core::sync::atomic::Ordering::SeqCst);
                writer.write_string_at(
                    40,
                    420,
                    &alloc::format!("ACTIVE HOPS: {}", active_hops),
                    green_text,
                );

                // Render 3D circuit visualization in the secure box
                let secure_renderer = HologramRenderer::new(width, height);
                let mut prev_pos: Option<Point2D> = None;

                for i in 0..active_hops {
                    let sphere = Mesh3D::new_node_sphere(0.15, 6, cyan);
                    // Calculate a snake-like path for the circuit in UI space
                    let x_off = -110 + (i as isize * 35);
                    let y_off = 150 + ((i % 2) as isize * 15);

                    secure_renderer.render_mesh(writer, &sphere, angle, angle * 0.5, x_off, y_off);

                    let curr_p2d = Point2D {
                        x: (x_off + (width as isize / 2)),
                        y: (y_off + (height as isize / 2)),
                    };

                    if let Some(prev) = prev_pos {
                        writer.draw_line(
                            prev.x,
                            prev.y,
                            curr_p2d.x,
                            curr_p2d.y,
                            Color {
                                r: 0,
                                g: 100,
                                b: 255,
                            },
                        );
                    }
                    prev_pos = Some(curr_p2d);
                }
            }

            // 4. Clear & Update Brain Core Area
            writer.fill_rect(
                Rect {
                    x: 30,
                    y: 500,
                    width: 280,
                    height: 60,
                },
                black,
            );

            let density = crate::storage::brain::CORE
                .synaptic_density
                .load(core::sync::atomic::Ordering::SeqCst);
            let hive_integrity = if crate::storage::brain::CORE
                .hive_root
                .load(core::sync::atomic::Ordering::SeqCst)
                != 0
            {
                "STABLE"
            } else {
                "INITIALIZING"
            };

            writer.write_string_at(
                40,
                510,
                &alloc::format!("DENSITY: {} SYNAPSES", density),
                green_text,
            );
            writer.write_string_at(
                40,
                530,
                &alloc::format!("HIVE: {}", hive_integrity),
                Color {
                    r: 180,
                    g: 0,
                    b: 255,
                },
            );

            // Render 3D Synaptic Hub
            let brain_renderer = HologramRenderer::new(width, height);
            // Nucleus (Center)
            let nucleus = Mesh3D::new_node_sphere(
                0.25,
                8,
                Color {
                    r: 150,
                    g: 0,
                    b: 255,
                },
            );
            brain_renderer.render_mesh(writer, &nucleus, angle * 0.5, angle, -110, 250);

            // Surrounding Synapses (representing fragments)
            for i in 0..6 {
                let s_angle = (i as f32 * 1.0) + angle;
                let s_dist = 0.5 + libm::sinf(angle * 2.0 + i as f32) * 0.1;
                let x = libm::cosf(s_angle) * s_dist;
                let z = libm::sinf(s_angle) * s_dist;

                let synapse = Mesh3D::new_node_sphere(0.08, 4, cyan);

                // Manual offset for orbiting synapses
                let x_off = -110 + (x * 60.0) as isize;
                let y_off = 250 + (z * 30.0) as isize;

                brain_renderer.render_mesh(writer, &synapse, angle, angle * 2.0, x_off, y_off);

                // Visual "Firing" - lines to nucleus
                let nucleus_p2d = Point2D {
                    x: -110 + (width as isize / 2),
                    y: 250 + (height as isize / 2),
                };
                let synapse_p2d = Point2D {
                    x: x_off + (width as isize / 2),
                    y: y_off + (height as isize / 2),
                };

                if libm::sinf(angle * 5.0 + i as f32) > 0.8 {
                    writer.draw_line(
                        nucleus_p2d.x,
                        nucleus_p2d.y,
                        synapse_p2d.x,
                        synapse_p2d.y,
                        white,
                    );
                }
            }

            // 5. Draw 3D Resource Visualization
            // Clear 3D area
            writer.fill_rect(
                Rect {
                    x: 345,
                    y: 90,
                    width: width - 370,
                    height: height - 150,
                },
                black,
            );

            let renderer = HologramRenderer::new(width, height);

            // Render 4 bars representing "Cores" or "Tasks"
            for i in 0..4 {
                // Get load metrics for each "core"
                let load = 0.5 + (libm::sinf(angle + (i as f32 * 0.5)) * 0.5 + 0.5) * 1.5;
                let mesh = Mesh3D::new_bar(0.4, load, 0.4, green_text);

                renderer.render_mesh(
                    writer,
                    &mesh,
                    0.2,         // Tilted slightly forward
                    angle * 0.2, // Slow spin
                    (width as isize / 4) + 50 + (i as isize * 60),
                    30,
                );
            }
        }
    }
}
