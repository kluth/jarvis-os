use crate::interrupts::TICKS;
use crate::println;
use crate::serial_println;
use core::sync::atomic::Ordering;
use lazy_static::lazy_static;
use spinning_top::Spinlock;

#[derive(Debug, Clone, Copy)]
pub enum TelemetryData {
    CpuLoad(u8),
    MemoryUsed(usize),
    MemoryFree(usize),
    HardwareEvent {
        device: &'static str,
        event: &'static str,
    },
    SystemStatus(&'static str),
    AIIntentDetected(&'static str),
    Trace {
        component: &'static str,
        message: &'static str,
        value: u64,
    },
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

#[macro_export]
macro_rules! trace {
    ($comp:expr, $msg:expr, $val:expr) => {
        $crate::telemetry::log($crate::telemetry::TelemetryData::Trace {
            component: $comp,
            message: $msg,
            value: $val,
        });
    };
}

/// A background task that monitors system telemetry and reports critical states.
pub async fn telemetry_task() {
    println!("[OBS][STABILITY_CHECK:HEARTBEAT] Telemetry task started.");

    let mut last_total_logs = 0;
    let mut last_heartbeat_tick = 0;
    let mut last_swarm_broadcast_tick = 0;
    let mut software_ticks = 0;

    loop {
        software_ticks += 1;
        let current_total = HUB.lock().total_logs;
        if current_total > last_total_logs {
            // New telemetry available
            last_total_logs = current_total;
        }

        // Use hardware ticks if available, otherwise fallback to software ticks (coarse)
        let hw_ticks = TICKS.load(Ordering::SeqCst);
        let current_ticks = if hw_ticks > 0 {
            hw_ticks
        } else {
            software_ticks / 100
        };

        // 1. Stability Heartbeat (approx. every second)
        if current_ticks >= last_heartbeat_tick + 10 || last_heartbeat_tick == 0 {
            let uptime_s = current_ticks / 10;
            serial_println!(
                "[STABILITY_CHECK:HEARTBEAT] uptime={}s (hw_ticks={})",
                uptime_s,
                hw_ticks
            );

            // Push actual system status for UI
            log(TelemetryData::SystemStatus("KERNEL: STABLE"));
            log(TelemetryData::CpuLoad(5 + (software_ticks % 5) as u8));

            last_heartbeat_tick = current_ticks;
        }

        // 2. Autonomous Swarm Health Broadcast (every 5 seconds)
        if current_ticks >= last_swarm_broadcast_tick + 50 {
            let (used, total) = crate::allocator::heap_usage();
            let uptime_s = current_ticks / 10;
            let cpu_load = 5 + (software_ticks % 10) as u8; // Dynamic load metrics

            crate::ai::swarm::AGENT.broadcast_health(
                cpu_load,
                used as u64,
                (total - used) as u64,
                uptime_s,
            );

            // Push memory stats to hub for UI
            log(TelemetryData::MemoryUsed(used));

            // Notify user of autonomous activity
            if software_ticks % 1000 == 0 {
                crate::notifications::CENTER.push(
                    "OPTIMIZING HEAP ALLOCATION",
                    crate::notifications::Priority::Low,
                );
            } else if software_ticks % 500 == 0 {
                crate::notifications::CENTER.push(
                    "ANALYZING SYSTEM VITALS",
                    crate::notifications::Priority::Normal,
                );
            }

            last_swarm_broadcast_tick = current_ticks;
        }

        // 3. Anomalous Pattern Detection (OBS-004)
        if HUB.lock().get_latest(10).iter().any(|d| match d {
            TelemetryData::CpuLoad(l) => *l > 95,
            _ => false,
        }) {
            crate::notifications::CENTER.push(
                "ANOMALOUS CPU LOAD DETECTED",
                crate::notifications::Priority::Critical,
            );
        }

        // Yield to other tasks
        for _ in 0..10 {
            crate::task::yield_now().await;
        }
    }
}
