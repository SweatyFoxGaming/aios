//! Aether: PCI bus discovery for Phoenix OS.

use crate::drivers::manager::{Device, DeviceType};
use crate::println;
use alloc::format;
use x86_64::instructions::port::Port;

/// PCI configuration port.
const CONFIG_ADDRESS: u16 = 0xCF8;
/// PCI data port.
const CONFIG_DATA: u16 = 0xCFC;

/// Read a 32-bit value from PCI configuration space.
#[must_use]
pub fn pci_config_read(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = (u32::from(bus) << 16)
        | (u32::from(slot) << 11)
        | (u32::from(func) << 8)
        | (u32::from(offset) & 0xFC)
        | 0x8000_0000;

    let mut addr_port = Port::new(CONFIG_ADDRESS);
    let mut data_port = Port::new(CONFIG_DATA);

    unsafe {
        addr_port.write(address);
        data_port.read()
    }
}

/// Scan the PCI bus for devices and log them.
pub fn scan_bus() {
    println!("--- Aether PCI Discovery ---");
    for bus in 0..8 {
        for slot in 0..32 {
            let vendor_id = pci_config_read(bus, slot, 0, 0) & 0xFFFF;
            if vendor_id != 0xFFFF {
                let device_id = (pci_config_read(bus, slot, 0, 0) >> 16) & 0xFFFF;
                println!(
                    "PCI Device: Bus {bus} Slot {slot}: Vendor {vendor_id:x} Device {device_id:x}"
                );

                // Register with Hephaestus
                crate::drivers::manager::register(Device {
                    name: format!("PCI-{vendor_id:x}:{device_id:x}"),
                    device_type: DeviceType::Unknown,
                    bus_info: Some(format!("PCI Bus {bus}, Slot {slot}")),
                });
            }
        }
    }
    println!("----------------------------");
}
