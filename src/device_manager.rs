//! ============================================================================
//! JARVIS OS Device Manager — Universal Device/Driver Model
//! ============================================================================
//! Provides:
//!   - DeviceClass enumeration for all hardware categories
//!   - Driver trait with probe/bind/init/shutdown lifecycle
//!   - Device struct with PCI info, MMIO BARs, IRQ routing
//!   - DeviceManager singleton for registration and lookup
//! ============================================================================

use alloc::vec::Vec;
use lazy_static::lazy_static;
use spinning_top::Spinlock;

// ============================================================================
// DEVICE CLASSIFICATION
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceClass {
    /// Display controllers / GPUs (PCI class 0x03)
    Display,
    /// Mass storage (PCI class 0x01: IDE, SATA, NVMe)
    Storage,
    /// Network controllers (PCI class 0x02)
    Network,
    /// Audio devices (PCI class 0x04: HDA, AC97)
    Audio,
    /// Bridge devices (PCI class 0x06: Host, ISA, PCI-PCI)
    Bridge,
    /// Input devices (keyboard, touchpad, touchscreen)
    Input,
    /// Sensors (accelerometer, gyro, ALS, temp)
    Sensor,
    /// Power management / EC
    Power,
    /// USB controllers (PCI class 0x0C: xHCI, EHCI, OHCI)
    Usb,
    /// System / uncategorized
    System,
}

impl DeviceClass {
    pub fn from_pci(class: u8, subclass: u8) -> Self {
        match class {
            0x01 => DeviceClass::Storage,
            0x02 => DeviceClass::Network,
            0x03 => DeviceClass::Display,
            0x04 => DeviceClass::Audio,
            0x06 => DeviceClass::Bridge,
            0x0C => {
                match subclass {
                    0x03 => DeviceClass::Usb,
                    _ => DeviceClass::System,
                }
            }
            _ => DeviceClass::System,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DeviceClass::Display => "Display",
            DeviceClass::Storage => "Storage",
            DeviceClass::Network => "Network",
            DeviceClass::Audio => "Audio",
            DeviceClass::Bridge => "Bridge",
            DeviceClass::Input => "Input",
            DeviceClass::Sensor => "Sensor",
            DeviceClass::Power => "Power",
            DeviceClass::Usb => "USB",
            DeviceClass::System => "System",
        }
    }
}

// ============================================================================
// DEVICE IDENTIFIER
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceId {
    pub vendor: u16,
    pub device: u16,
    pub subsystem_vendor: u16,
    pub subsystem_device: u16,
    pub revision: u8,
}

impl DeviceId {
    pub const fn unknown() -> Self {
        Self { vendor: 0, device: 0, subsystem_vendor: 0, subsystem_device: 0, revision: 0 }
    }
}

// ============================================================================
// MEMORY-MAPPED I/O REGION
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct MmioRegion {
    pub base: u64,
    pub len: usize,
    pub prefetchable: bool,
    pub is_mmio: bool,  // false = I/O port
}

impl MmioRegion {
    pub const fn empty() -> Self {
        Self { base: 0, len: 0, prefetchable: false, is_mmio: true }
    }
}

// ============================================================================
// DEVICE — represents a discovered hardware device
// ============================================================================

#[derive(Debug, Clone)]
pub struct Device {
    pub id: DeviceId,
    pub name: &'static str,
    pub class: DeviceClass,
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    pub bars: [MmioRegion; 6],
    pub irq: u8,
    pub irq_type: IrqType,
    pub enabled: bool,
    pub driver_name: Option<&'static str>,
    pub custom_data: Option<alloc::vec::Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IrqType {
    Legacy,
    Msi,
    Msix,
    None,
}

impl Device {
    pub fn new(
        vendor: u16, device: u16, class: DeviceClass,
        bus: u8, slot: u8, function: u8,
    ) -> Self {
        Self {
            id: DeviceId {
                vendor, device,
                subsystem_vendor: 0,
                subsystem_device: 0,
                revision: 0,
            },
            name: "Unknown Device",
            class,
            bus, slot, function,
            bars: [MmioRegion::empty(); 6],
            irq: 0,
            irq_type: IrqType::None,
            enabled: false,
            driver_name: None,
            custom_data: None,
        }
    }

    /// Returns true if this device has the given vendor:device ID
    pub fn matches_id(&self, vendor: u16, device: u16) -> bool {
        self.id.vendor == vendor && self.id.device == device
    }

    /// Returns the first non-zero MMIO BAR
    pub fn mmio_base(&self) -> Option<u64> {
        for bar in &self.bars {
            if bar.base != 0 && bar.is_mmio {
                return Some(bar.base);
            }
        }
        None
    }

    /// Returns the first non-zero framebuffer BAR (prefetchable MMIO)
    pub fn framebuffer_base(&self) -> Option<u64> {
        for bar in &self.bars {
            if bar.base != 0 && bar.is_mmio && bar.prefetchable {
                return Some(bar.base);
            }
        }
        None
    }
}

// ============================================================================
// DRIVER TRAIT — Every hardware driver implements this
// ============================================================================

pub trait Driver: Send {
    /// Human-readable driver name
    fn name(&self) -> &'static str;

    /// Check if this driver handles a given vendor:device pair
    fn probe(&self, vendor: u16, device: u16, class: DeviceClass) -> bool;

    /// Initialize the device hardware
    fn init(&mut self, device: &mut Device) -> Result<(), &'static str>;

    /// Shut down the device (power-off / disable)
    fn shutdown(&mut self, device: &mut Device) -> Result<(), &'static str>;

    /// Handle an IRQ from this device (returns true if handled)
    fn handle_irq(&mut self, _device: &mut Device) -> bool { false }
}

// ============================================================================
// DRIVER REGISTRY — holds all registered driver instances
// ============================================================================

pub struct DriverEntry {
    pub driver: &'static mut dyn Driver,
    pub bound_device: Option<usize>,  // device index
    pub initialized: bool,
}

pub struct DeviceManager {
    pub devices: Vec<Device>,
    pub drivers: Vec<DriverEntry>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            drivers: Vec::new(),
        }
    }

    /// Register a discovered device
    pub fn register(&mut self, device: Device) -> usize {
        let idx = self.devices.len();
        crate::serial_println!(
            "DEV: [{}:{}.{}] {} ({:#06x}:{:#06x}) class={}",
            device.bus, device.slot, device.function,
            device.name, device.id.vendor, device.id.device,
            device.class.name(),
        );
        self.devices.push(device);
        idx
    }

    /// Register a driver
    pub fn register_driver(&mut self, driver: &'static mut dyn Driver) -> usize {
        let idx = self.drivers.len();
        crate::serial_println!("DRV: Registered driver '{}'", driver.name());
        self.drivers.push(DriverEntry {
            driver,
            bound_device: None,
            initialized: false,
        });
        idx
    }

    /// Probe all drivers against all unbound devices
    pub fn probe_all(&mut self) {
        crate::serial_println!("DEV: Probing {} device(s) against {} driver(s)...",
            self.devices.len(), self.drivers.len());

        for dev_idx in 0..self.devices.len() {
            // Skip already-bound devices
            let already_bound = self.drivers.iter().any(|d| d.bound_device == Some(dev_idx));
            if already_bound { continue; }

            let (vendor, device, class) = {
                let d = &self.devices[dev_idx];
                (d.id.vendor, d.id.device, d.class)
            };

            for drv_idx in 0..self.drivers.len() {
                if self.drivers[drv_idx].bound_device.is_some() {
                    continue; // driver already bound to another device
                }

                if self.drivers[drv_idx].driver.probe(vendor, device, class) {
                    crate::serial_println!(
                        "DEV: Driver '{}' <-> [{}:{}.{}] {}",
                        self.drivers[drv_idx].driver.name(),
                        self.devices[dev_idx].bus,
                        self.devices[dev_idx].slot,
                        self.devices[dev_idx].function,
                        self.devices[dev_idx].name,
                    );
                    self.drivers[drv_idx].bound_device = Some(dev_idx);
                    break;
                }
            }
        }
    }

    /// Initialize all bound drivers
    pub fn init_all(&mut self) {
        for i in 0..self.drivers.len() {
            if self.drivers[i].initialized { continue; }
            if let Some(dev_idx) = self.drivers[i].bound_device {
                let result = {
                    let driver = &mut *self.drivers[i].driver;
                    driver.init(&mut self.devices[dev_idx])
                };
                match result {
                    Ok(()) => {
                        self.devices[dev_idx].enabled = true;
                        self.devices[dev_idx].driver_name = Some(self.drivers[i].driver.name());
                        self.drivers[i].initialized = true;
                        crate::serial_println!(
                            "DEV: '{}' initialized on [{}:{}.{}]",
                            self.drivers[i].driver.name(),
                            self.devices[dev_idx].bus,
                            self.devices[dev_idx].slot,
                            self.devices[dev_idx].function,
                        );
                    }
                    Err(e) => {
                        crate::serial_println!(
                            "DEV: '{}' failed init on [{}:{}.{}]: {}",
                            self.drivers[i].driver.name(),
                            self.devices[dev_idx].bus,
                            self.devices[dev_idx].slot,
                            self.devices[dev_idx].function,
                            e,
                        );
                    }
                }
            }
        }
    }

    /// Find all devices of a given class
    pub fn find_by_class(&self, class: DeviceClass) -> Vec<&Device> {
        self.devices.iter().filter(|d| d.class == class).collect()
    }

    /// Find all devices of a given class (mutable)
    pub fn find_by_class_mut(&mut self, class: DeviceClass) -> Vec<&mut Device> {
        self.devices.iter_mut().filter(|d| d.class == class).collect()
    }

    /// Find the first bound driver for a device class
    pub fn find_driver_for_class(&self, class: DeviceClass) -> Option<usize> {
        for i in 0..self.drivers.len() {
            if let Some(dev_idx) = self.drivers[i].bound_device {
                if self.devices[dev_idx].class == class {
                    return Some(i);
                }
            }
        }
        None
    }

    // Remove get_driver_mut - needs proper lifetime handling
    // Re-add when Phase 1 driver implementations are needed.
    // pub fn get_driver_mut(...) -> ...
}

// ============================================================================
// GLOBAL DEVICE MANAGER SINGLETON
// ============================================================================

lazy_static! {
    pub static ref MANAGER: Spinlock<DeviceManager> = Spinlock::new(DeviceManager::new());
}

/// Register a device (convenience wrapper)
pub fn register_device(device: Device) -> usize {
    MANAGER.lock().register(device)
}

/// Register a driver (convenience wrapper)
pub fn register_driver(driver: &'static mut dyn Driver) -> usize {
    MANAGER.lock().register_driver(driver)
}

/// Probe all devices against drivers
pub fn probe_all() {
    MANAGER.lock().probe_all();
}

/// Initialize all bound drivers
pub fn init_all() {
    MANAGER.lock().init_all();
}