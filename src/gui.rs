use crate::serial_println;
use crate::telemetry;
use crate::vga_buffer::{Color, Rect, WRITER};

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
            height: 200,
        },
        blue_border,
    );
    writer.write_string_at(30, 80, "[ SYSTEM TELEMETRY ]", cyan);

    // AI Shell Box
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
    writer.write_string_at(350, 80, "[ VOICE SHELL TRANSCRIPT ]", cyan);

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
    loop {
        update_dynamic_elements();
        // Update every 200ms approx
        for _ in 0..2 {
            core::future::ready(()).await;
        }
    }
}

fn update_dynamic_elements() {
    if let Some(writer) = WRITER.lock().as_mut() {
        let black = Color { r: 0, g: 0, b: 0 };
        let green_text = Color {
            r: 0,
            g: 255,
            b: 150,
        };

        // Clear telemetry area (background only)
        writer.fill_rect(
            Rect {
                x: 30,
                y: 110,
                width: 280,
                height: 150,
            },
            black,
        );

        let latest = telemetry::HUB.lock().get_latest(5);
        let mut y = 110;
        for data in latest {
            let text = match data {
                telemetry::TelemetryData::CpuLoad(_) => "CPU Load: ACTIVE",
                telemetry::TelemetryData::MemoryUsed(_) => "Memory: ALLOCATED",
                telemetry::TelemetryData::MemoryFree(_) => "Memory: AVAILABLE",
                telemetry::TelemetryData::HardwareEvent { .. } => "HW Event: DETECTED",
                telemetry::TelemetryData::SystemStatus(s) => s,
                telemetry::TelemetryData::AIIntentDetected(_) => "AI: INTENT ANALYZED",
            };

            writer.write_string_at(40, y, text, green_text);
            y += 20;
        }
    }
}
