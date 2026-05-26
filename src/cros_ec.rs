//! ============================================================================
//! JARVIS OS Chromebook EC (cros_ec) Driver
//! ============================================================================
//! Provides:
//!   - Communication with the ChromeOS Embedded Controller
//!   - Keyboard/Touchpad event tunneling
//!   - Battery/Power status monitoring
//!   - Thermal/Fan control
//! ============================================================================

use crate::i2c::{I2cResult, I2cError};
use crate::serial_println;

pub const EC_I2C_ADDR: u8 = 0x1E;

pub struct CrosEc {
    bus_id: usize,
}

impl CrosEc {
    pub fn new(bus_id: usize) -> Self {
        Self { bus_id }
    }

    /// Read the EC version string
    pub fn get_version(&self) -> I2cResult<alloc::string::String> {
        let mut buf = [0u8; 32];
        // EC_CMD_GET_VERSION = 0x02
        crate::i2c::smbus_read_block(self.bus_id, EC_I2C_ADDR, 0x02, &mut buf)?;
        
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        Ok(alloc::string::String::from_utf8_lossy(&buf[..len]).into_owned())
    }

    /// Read battery percentage
    pub fn get_battery_level(&self) -> I2cResult<u8> {
        // EC_CMD_BATTERY_GET_REMAINING_CAPACITY = 0x24 (pseudo)
        let val = crate::i2c::smbus_read_byte(self.bus_id, EC_I2C_ADDR, 0x24)?;
        Ok(val)
    }
}

pub fn init() {
    serial_println!("EC: Initializing Chromebook EC driver...");
    // The EC is usually on the primary SMBus (bus 0)
    let ec = CrosEc::new(0);
    
    match ec.get_version() {
        Ok(ver) => serial_println!("EC: Found ChromeOS EC, Version: {}", ver),
        Err(_) => serial_println!("EC: ChromeOS EC not found or not responsive on I2C 0x1E"),
    }
}
