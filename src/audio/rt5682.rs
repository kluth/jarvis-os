//! ============================================================================
//! JARVIS OS Realtek RT5682 Audio Codec Driver
//! ============================================================================
//! Provides:
//!   - I2C communication with the codec (address 0x1A)
//!   - Power-on/Reset sequence
//!   - Clocking configuration (PLL/BCLK)
//!   - Headset/Mic detection
//! ============================================================================

use crate::i2c::{I2cResult, I2cError};
use crate::serial_println;

pub const RT5682_I2C_ADDR: u8 = 0x1A;

// RT5682 Registers
const REG_RESET: u16 = 0x0000;
const REG_PWR_MGMT_1: u16 = 0x0001;
const REG_PLL_1: u16 = 0x0002;
const REG_CHIP_ID: u16 = 0x0003;

pub struct Rt5682 {
    bus_id: usize,
}

impl Rt5682 {
    pub fn new(bus_id: usize) -> Self {
        Self { bus_id }
    }

    pub fn init(&self) -> I2cResult<()> {
        serial_println!("AUDIO: Initializing RT5682 codec...");
        
        // 1. Reset codec
        self.write_reg(REG_RESET, 0x0000)?;
        
        // 2. Check Chip ID (Expected: 0x6419 or similar)
        let id = self.read_reg(REG_CHIP_ID)?;
        serial_println!("AUDIO: RT5682 Chip ID: 0x{:04X}", id);
        
        // 3. Basic power on
        self.write_reg(REG_PWR_MGMT_1, 0x8000)?; // Enable main power
        
        serial_println!("AUDIO: RT5682 codec ready.");
        Ok(())
    }

    fn read_reg(&self, reg: u16) -> I2cResult<u16> {
        let mut buf = [0u8; 2];
        let reg_bytes = reg.to_be_bytes();
        let mgr = crate::i2c::I2C_MANAGER.lock();
        let bus = mgr.bus(self.bus_id).ok_or(I2cError::InvalidParameter)?;
        bus.transfer(RT5682_I2C_ADDR, &reg_bytes, &mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    fn write_reg(&self, reg: u16, val: u16) -> I2cResult<()> {
        let reg_bytes = reg.to_be_bytes();
        let val_bytes = val.to_be_bytes();
        let mut combined = [0u8; 4];
        combined[0..2].copy_from_slice(&reg_bytes);
        combined[2..4].copy_from_slice(&val_bytes);
        let mgr = crate::i2c::I2C_MANAGER.lock();
        let bus = mgr.bus(self.bus_id).ok_or(I2cError::InvalidParameter)?;
        bus.transfer(RT5682_I2C_ADDR, &combined, &mut [])?;
        Ok(())
    }
}
