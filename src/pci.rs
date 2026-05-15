use x86_64::instructions::port::Port;

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
}

pub fn scan_bus() -> alloc::vec::Vec<PciDevice> {
    let mut devices = alloc::vec::Vec::new();
    for bus in 0..255 {
        for slot in 0..32 {
            for function in 0..8 {
                let vendor_id = pci_read_word(bus, slot, function, 0);
                if vendor_id == 0xFFFF {
                    continue;
                }
                
                let device_id = pci_read_word(bus, slot, function, 2);
                let class_rev = pci_read_word(bus, slot, function, 8);
                let class = (class_rev >> 8) as u8;
                let subclass = (class_rev & 0xFF) as u8;

                let dev = PciDevice {
                    bus,
                    slot,
                    function,
                    vendor_id,
                    device_id,
                    class,
                    subclass,
                };

                // Register with Device Manager
                let name = match (class, subclass) {
                    (0x01, 0x01) => "IDE Controller",
                    (0x01, 0x06) => "SATA Controller",
                    (0x02, 0x00) => "Ethernet Controller",
                    (0x03, 0x00) => "VGA Display Controller",
                    (0x04, 0x03) => "High Definition Audio",
                    (0x06, 0x00) => "Host Bridge",
                    (0x06, 0x01) => "ISA Bridge",
                    _ => "Unknown PCI Device",
                };

                let dev_type = match class {
                    0x01 => crate::device_manager::DeviceType::Storage,
                    0x02 => crate::device_manager::DeviceType::Network,
                    0x03 => crate::device_manager::DeviceType::Graphics,
                    0x04 => crate::device_manager::DeviceType::Audio,
                    _ => crate::device_manager::DeviceType::System,
                };

                crate::device_manager::register(crate::device_manager::DeviceInfo {
                    name: alloc::string::String::from(name),
                    dev_type,
                    status: "Discovered",
                });

                // Intel HDA: Class 04, Subclass 03
                if class == 0x04 && subclass == 0x03 {
                    devices.push(dev);
                }
                
                // If it's not a multi-function device, don't check other functions
                if function == 0 {
                    let header_type = pci_read_word(bus, slot, 0, 0x0E) & 0xFF;
                    if header_type & 0x80 == 0 {
                        break;
                    }
                }
            }
        }
    }
    devices
}

impl PciDevice {
    pub fn read_bar(&self, bar_index: u8) -> u32 {
        let offset = 0x10 + (bar_index * 4);
        let low = pci_read_dword(self.bus, self.slot, self.function, offset);
        low
    }

    pub fn enable_bus_mastering(&self) {
        let mut command = pci_read_word(self.bus, self.slot, self.function, 0x04);
        command |= 1 << 2; // Bus Master
        command |= 1 << 1; // Memory Space
        pci_write_word(self.bus, self.slot, self.function, 0x04, command);
    }
}

fn pci_read_dword(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | (offset as u32 & 0xFC)
        | 0x80000000;

    let mut config_addr = Port::<u32>::new(0xCF8);
    let mut config_data = Port::<u32>::new(0xCFC);

    unsafe {
        config_addr.write(address);
        config_data.read()
    }
}

fn pci_write_word(bus: u8, slot: u8, func: u8, offset: u8, value: u16) {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | (offset as u32 & 0xFC)
        | 0x80000000;

    let mut config_addr = Port::<u32>::new(0xCF8);
    let mut config_data = Port::<u32>::new(0xCFC);

    unsafe {
        config_addr.write(address);
        let mut tmp = config_data.read();
        tmp &= !(0xFFFF << ((offset & 2) * 8));
        tmp |= (value as u32) << ((offset & 2) * 8);
        config_data.write(tmp);
    }
}

fn pci_read_word(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | (offset as u32 & 0xFC)
        | 0x80000000;

    let mut config_addr = Port::<u32>::new(0xCF8);
    let mut config_data = Port::<u32>::new(0xCFC);

    unsafe {
        config_addr.write(address);
        ((config_data.read() >> ((offset & 2) * 8)) & 0xFFFF) as u16
    }
}
