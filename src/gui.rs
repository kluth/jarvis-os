use crate::gui_3d::{HologramRenderer, Mesh3D};
use crate::notifications::CENTER;
use crate::serial_println;
use crate::telemetry;
use crate::vga_buffer::{Color, Rect, WRITER};

use spinning_top::Spinlock;

lazy_static::lazy_static! {
    static ref ACTIVITY_LOG: Spinlock<[Option<crate::notifications::Notification>; 5]> = Spinlock::new([None, None, None, None, None]);
}

pub fn init_ui() {
    if let Some(writer) = WRITER.lock().as_mut() {
        writer.clear();
        draw_static_elements(writer);
    }
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

    // Activity Log Box
    serial_println!("GUI: Drawing activity log box...");
    writer.draw_rect(
        Rect {
            x: 20,
            y: 230,
            width: 300,
            height: 150,
        },
        blue_border,
    );
    writer.write_string_at(30, 240, "[ RECENT ACTIVITY ]", cyan);

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
        "STATUS: SYSTEM CORE ONLINE | ENCRYPTION: ACTIVE | CONNECTION: SECURE",
        footer_text,
    );
}

pub async fn ui_task() {
    let mut angle: f32 = 0.0;

    loop {
        update_dynamic_elements(angle);
        angle += 0.05;
        if angle > core::f32::consts::TAU {
            angle = 0.0;
        }

        // Update every ~100ms approx
        for _ in 0..1 {
            core::future::ready(()).await;
            crate::task::yield_now().await;
        }
    }
}

fn update_dynamic_elements(angle: f32) {
    if let Some(mut writer_guard) = WRITER.try_lock() {
        if let Some(writer) = writer_guard.as_mut() {
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

            // 2. Clear & Update Activity Log Area
            writer.fill_rect(
                Rect {
                    x: 30,
                    y: 270,
                    width: 280,
                    height: 100,
                },
                black,
            );

            // Handle new notifications (rolling log)
            if let Some(new_notif) = CENTER.pop() {
                let mut log = ACTIVITY_LOG.lock();
                // Shift logs up
                for i in 0..4 {
                    log[i] = log[i + 1].clone();
                }
                log[4] = Some(new_notif);
            }

            let mut log_y = 270;
            {
                let log = ACTIVITY_LOG.lock();
                for notif in log.iter().flatten() {
                    let color = match notif.priority {
                        crate::notifications::Priority::Critical => Color { r: 255, g: 0, b: 0 },
                        crate::notifications::Priority::High => Color {
                            r: 255,
                            g: 255,
                            b: 0,
                        },
                        _ => Color {
                            r: 0,
                            g: 200,
                            b: 255,
                        },
                    };
                    writer.write_string_at(40, log_y, &notif.message, color);
                    log_y += 18;
                }
            }

            // 3. Draw 3D Resource Visualization
            let info = writer.get_info();
            let width = info.width;
            let height = info.height;

            // Clear 3D area
            writer.fill_rect(
                Rect {
                    x: 345,
                    y: 90,
                    width: width - 370,
                    height: height - 170,
                },
                black,
            );

            let renderer = HologramRenderer::new(width, height);

            // Render 4 bars representing "Cores" or "Tasks"
            for i in 0..4 {
                // Get simulated load for each "core"
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
