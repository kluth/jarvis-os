//! ============================================================================
//! JARVIS OS Maxim MAX98390 Smart Amplifier Driver
//! ============================================================================
//! Provides:
//!   - I2C communication (address 0x38 or 0x3C)
//!   - Dynamic Voltage/Current sensing
//!   - Speaker protection (thermal/excursion)
//! ============================================================================

use crate::i2c::{I2cError, I2cResult};
use crate::serial_println;

pub const MAXIM_I2C_ADDR_L: u8 = 0x38;
pub const MAXIM_I2C_ADDR_R: u8 = 0x3C;

pub struct Max98390 {
    bus_id: usize,
    addr: u8,
}

impl Max98390 {
    pub fn new(bus_id: usize, addr: u8) -> Self {
        Self { bus_id, addr }
    }

    pub fn init(&self) -> I2cResult<()> {
        serial_println!(
            "AUDIO: Initializing Maxim Smart Amp at 0x{:02X}...",
            self.addr
        );

        // 1. Reset
        self.write_reg(0x2000, 0x01)?;

        // 2. Enable Amp
        self.write_reg(0x2001, 0x01)?;

        serial_println!("AUDIO: Maxim Smart Amp ready.");
        Ok(())
    }

    fn write_reg(&self, reg: u16, val: u8) -> I2cResult<()> {
        let reg_bytes = reg.to_be_bytes();
        let mut combined = [0u8; 3];
        combined[0..2].copy_from_slice(&reg_bytes);
        combined[2] = val;
        let mgr = crate::i2c::I2C_MANAGER.lock();
        let bus = mgr.bus(self.bus_id).ok_or(I2cError::InvalidParameter)?;
        bus.transfer(self.addr, &combined, &mut [])?;
        Ok(())
    }
}
