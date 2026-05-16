use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use spinning_top::Spinlock;

use crate::println;

#[derive(Debug, Clone)]
pub struct BiometricReading {
    pub heart_rate: u8,
    pub oxygen_saturation: u8,
    pub stress_level: u8, // 0-100
    pub timestamp: u64,
}

pub struct BiometricSensor {
    history: Spinlock<Vec<BiometricReading>>,
    counter: AtomicUsize,
}

impl Default for BiometricSensor {
    fn default() -> Self {
        Self {
            history: Spinlock::new(Vec::new()),
            counter: AtomicUsize::new(0),
        }
    }
}

impl BiometricSensor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, reading: BiometricReading) {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        if count.is_multiple_of(10) {
            println!(
                "Biometrics: HR: {} bpm, SpO2: {}%, Stress: {}/100",
                reading.heart_rate, reading.oxygen_saturation, reading.stress_level
            );
        }

        let mut hist = self.history.lock();
        if hist.len() > 100 {
            hist.remove(0); // keep rolling window
        }
        hist.push(reading);
    }

    pub fn get_latest(&self) -> Option<BiometricReading> {
        self.history.lock().last().cloned()
    }
}

lazy_static! {
    pub static ref BIOMETRICS: Arc<BiometricSensor> = Arc::new(BiometricSensor::new());
}

pub async fn biometrics_task() {
    println!("Health: Biometric Telemetry task initialized.");

    // Simulate heart rate variability
    let mut base_hr = 70;

    loop {
        // Simulate reading from I2C/SPI sensors
        base_hr += 1;
        if base_hr > 85 {
            base_hr = 70;
        }

        BIOMETRICS.record(BiometricReading {
            heart_rate: base_hr,
            oxygen_saturation: 98,
            stress_level: (base_hr - 60) * 2,
            timestamp: 0,
        });

        for _ in 0..10 {
            crate::task::yield_now().await;
        }
    }
}

#[cfg(feature = "test")]
pub fn test_biometrics() {
    crate::serial_print!("test_biometrics... ");

    let sensor = BiometricSensor::new();
    sensor.record(BiometricReading {
        heart_rate: 75,
        oxygen_saturation: 99,
        stress_level: 20,
        timestamp: 100,
    });

    let latest = sensor.get_latest().unwrap();
    assert_eq!(latest.heart_rate, 75);

    crate::serial_println!("[ok]");
}
