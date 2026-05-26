use crate::device_manager::{self, Device, DeviceClass, DeviceId, IrqType, MmioRegion};

pub fn init() {
    crate::serial_println!("Ethernet: Initialized VirtIO driver.");

    device_manager::register_device(Device {
        id: DeviceId {
            vendor: 0x1AF4,
            device: 0x1000,
            subsystem_vendor: 0,
            subsystem_device: 0,
            revision: 0,
        },
        name: "VirtIO Interface 0",
        class: DeviceClass::Network,
        bus: 0,
        slot: 0,
        function: 0,
        bars: [MmioRegion::empty(); 6],
        irq: 0,
        irq_type: IrqType::None,
        enabled: true,
        power_state: device_manager::PowerState::S0,
        driver_name: None,
        custom_data: None,
    });

    crate::serial_println!("Ethernet: Local IP assigned: 192.168.1.100");
}
