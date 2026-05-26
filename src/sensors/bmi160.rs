//! ============================================================================
//! JARVIS OS BMI160 Accelerometer/Gyroscope Driver
//! ============================================================================
//! Provides:
//!   - 6-axis motion sensing
//!   - Real-time orientation tracking for the GUI
//!   - Collision/Impact detection
//! ============================================================================

use crate::i2c::{I2cResult, I2cError};
use crate::serial_println;

pub const BMI160_I2C_ADDR: u8 = 0x68;

// BMI160 Registers
const REG_CHIP_ID: u8 = 0x00;
const REG_DATA_ACCEL_X: u8 = 0x12;
const REG_DATA_GYRO_X: u8 = 0x0C;
const REG_CMD: u8 = 0x7E;

pub struct Bmi160 {
    bus_id: usize,
}

impl Bmi160 {
    pub fn new(bus_id: usize) -> Self {
        Self { bus_id }
    }

    pub fn init(&self) -> I2cResult<()> {
        let chip_id = crate::i2c::smbus_read_byte(self.bus_id, BMI160_I2C_ADDR, REG_CHIP_ID)?;
        if chip_id != 0xD1 {
            return Err(I2cError::InvalidParameter);
        }
        
        // Power up: ACCEL_NORMAL = 0x11, GYRO_NORMAL = 0x15
        crate::i2c::smbus_write_byte(self.bus_id, BMI160_I2C_ADDR, REG_CMD, 0x11)?;
        crate::i2c::smbus_write_byte(self.bus_id, BMI160_I2C_ADDR, REG_CMD, 0x15)?;
        
        serial_println!("SENSOR: BMI160 initialized successfully.");
        Ok(())
    }

    pub fn read_accel(&self) -> I2cResult<(i16, i16, i16)> {
        let mut buf = [0u8; 6];
        crate::i2c::smbus_read_block(self.bus_id, BMI160_I2C_ADDR, REG_DATA_ACCEL_X, &mut buf)?;
        
        let x = i16::from_le_bytes([buf[0], buf[1]]);
        let y = i16::from_le_bytes([buf[2], buf[3]]);
        let z = i16::from_le_bytes([buf[4], buf[5]]);
        
        Ok((x, y, z))
    }
}

pub async fn sensor_task() {
    serial_println!("SENSOR: Starting BMI160 monitoring task...");
    let sensor = Bmi160::new(0);
    
    if sensor.init().is_ok() {
        loop {
            if let Ok((x, y, _z)) = sensor.read_accel() {
                // For now, just log if there's significant movement
                if x.abs() > 1000 || y.abs() > 1000 {
                    // serial_println!("SENSOR: Accel -> X: {}, Y: {}, Z: {}", x, y, z);
                }
            }
            crate::task::yield_now().await;
        }
    }
}
