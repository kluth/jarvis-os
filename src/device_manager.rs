use alloc::vec::Vec;
use alloc::string::String;
use spinning_top::Spinlock;
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub enum DeviceType {
    Storage,
    Network,
    Audio,
    Graphics,
    Input,
    System,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub dev_type: DeviceType,
    pub status: &'static str,
}

pub struct DeviceManager {
    devices: Vec<DeviceInfo>,
}

impl DeviceManager {
    pub fn new() -> Self {
        DeviceManager {
            devices: Vec::new(),
        }
    }

    pub fn register_device(&mut self, device: DeviceInfo) {
        crate::println!("Device Manager: Registering -> {:?} ({:?})", device.name, device.dev_type);
        self.devices.push(device);
    }

    pub fn get_devices(&self) -> &Vec<DeviceInfo> {
        &self.devices
    }
}

lazy_static! {
    pub static ref MANAGER: Spinlock<DeviceManager> = Spinlock::new(DeviceManager::new());
}

pub fn register(device: DeviceInfo) {
    MANAGER.lock().register_device(device);
}
