use crate::vga_buffer::WRITER;
use crate::telemetry;

pub fn init_ui() {
    if let Some(writer) = WRITER.lock().as_mut() {
        writer.clear();
        draw_static_elements();
    }
}

fn draw_static_elements() {
    if let Some(writer) = WRITER.lock().as_mut() {
        let info = writer.get_info();
        let width = info.width;
        let height = info.height;

        // Main Border
        writer.draw_rect(5, 5, width - 10, height - 10, 0, 180, 255);
        writer.draw_rect(10, 10, width - 20, height - 20, 0, 100, 200);

        // Header
        writer.fill_rect(20, 20, 200, 30, 0, 80, 150);
        writer.write_string_at(30, 30, "J.A.R.V.I.S. OS v0.1.0", 255, 255, 255);

        // Telemetry Box
        writer.draw_rect(20, 70, 300, 200, 0, 180, 255);
        writer.write_string_at(30, 80, "[ SYSTEM TELEMETRY ]", 0, 255, 255);

        // AI Shell Box
        writer.draw_rect(340, 70, width - 360, height - 150, 0, 180, 255);
        writer.write_string_at(350, 80, "[ VOICE SHELL TRANSCRIPT ]", 0, 255, 255);

        // Footer
        writer.write_string_at(20, height - 40, "STATUS: SYSTEM CORE ONLINE | ENCRYPTION: ACTIVE | CONNECTION: SECURE", 0, 200, 200);
    }
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
        // Clear telemetry area (background only)
        writer.fill_rect(30, 110, 280, 150, 0, 0, 0);

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
            
            writer.write_string_at(40, y, text, 0, 255, 150);
            y += 20;
        }
    }
}
