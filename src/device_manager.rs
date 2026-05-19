use crate::sync::Spinlock;
use alloc::vec::Vec;
use lazy_static::lazy_static;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceType {
    Storage,
    Network,
    Audio,
    Graphics,
    Input,
    System,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    PowerControl,
    VolumeControl,
    BrightnessControl,
    Diagnostic,
    StorageRead,
    StorageWrite,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: &'static str,
    pub dev_type: DeviceType,
    pub status: &'static str,
    pub capabilities: &'static [Capability],
}

#[derive(Default)]
pub struct DeviceManager {
    devices: Vec<DeviceInfo>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_device(&mut self, device: DeviceInfo) {
        crate::serial_println!(
            "Device Manager: Registering -> {:?} ({:?}) with capabilities: {:?}",
            device.name,
            device.dev_type,
            device.capabilities
        );
        self.devices.push(device);
    }

    pub fn get_devices(&self) -> &Vec<DeviceInfo> {
        &self.devices
    }

    pub fn find_by_capability(&self, capability: Capability) -> Vec<DeviceInfo> {
        self.devices
            .iter()
            .filter(|d| d.capabilities.contains(&capability))
            .cloned()
            .collect()
    }
}

lazy_static! {
    pub static ref MANAGER: Spinlock<DeviceManager> = Spinlock::new(DeviceManager::new());
}

pub fn register(device: DeviceInfo) {
    MANAGER.lock().register_device(device);
}
