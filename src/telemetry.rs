use spinning_top::Spinlock;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy)]
pub enum TelemetryData {
    CpuLoad(u8),
    MemoryUsed(usize),
    MemoryFree(usize),
    HardwareEvent { device: &'static str, event: &'static str },
    SystemStatus(&'static str),
    AIIntentDetected(&'static str),
}

const BUFFER_SIZE: usize = 256;

pub struct TelemetryHub {
    buffer: [Option<TelemetryData>; BUFFER_SIZE],
    index: usize,
    total_logs: u64,
}

impl TelemetryHub {
    pub fn log(&mut self, data: TelemetryData) {
        self.buffer[self.index] = Some(data);
        self.index = (self.index + 1) % BUFFER_SIZE;
        self.total_logs += 1;
    }

    pub fn get_latest(&self, count: usize) -> alloc::vec::Vec<TelemetryData> {
        let mut result = alloc::vec::Vec::new();
        let actual_count = core::cmp::min(count, BUFFER_SIZE);
        
        for i in 0..actual_count {
            let idx = (self.index + BUFFER_SIZE - 1 - i) % BUFFER_SIZE;
            if let Some(data) = self.buffer[idx] {
                result.push(data);
            } else {
                break;
            }
        }
        result
    }
}

lazy_static! {
    pub static ref HUB: Spinlock<TelemetryHub> = Spinlock::new(TelemetryHub {
        buffer: [None; BUFFER_SIZE],
        index: 0,
        total_logs: 0,
    });
}

pub fn log(data: TelemetryData) {
    HUB.lock().log(data);
}

/// A background task that monitors system telemetry and reports critical states.
pub async fn telemetry_task() {
    let mut last_total_logs = 0;
    loop {
        let current_total = HUB.lock().total_logs;
        if current_total > last_total_logs {
            // New telemetry available
            // In a real JARVIS OS, this would be analyzed by the AI layer
            last_total_logs = current_total;
        }
        
        // Yield to other tasks
        core::future::ready(()).await;
    }
}
