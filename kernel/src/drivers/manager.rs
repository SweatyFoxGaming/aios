//! Hephaestus: Modular Device Manager for Phoenix OS.
//! Manages driver lifecycles and registration.

use crate::println;
use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

/// Type of hardware device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Network interface.
    Network,
    /// Storage controller.
    Storage,
    /// Display adapter.
    Display,
    /// Input device (Keyboard/Mouse).
    Input,
    /// Unknown device.
    Unknown,
}

/// Represents a discovered hardware device.
#[derive(Debug, Clone)]
pub struct Device {
    /// Name of the device.
    pub name: String,
    /// Category of the device.
    pub device_type: DeviceType,
    /// PCI address (if applicable).
    pub bus_info: Option<String>,
}

lazy_static! {
    // Preallocated so `DEVICES.lock().push` below never needs to grow the
    // `Vec` -- growing an existing heap allocation crashes on this target
    // (see safe_alloc.rs).
    static ref DEVICES: Mutex<Vec<Device>> = Mutex::new(Vec::with_capacity(64));
}

/// Register a new device with the manager.
pub fn register(device: Device) {
    println!(
        "[Hephaestus] Registering device: {} [{:?}]",
        device.name, device.device_type
    );
    DEVICES.lock().push(device);
}

/// List all discovered devices.
pub fn list_devices() {
    let devices = DEVICES.lock();
    println!("--- Hephaestus Device Registry ---");
    for dev in devices.iter() {
        println!("{}: {:?} ({:?})", dev.name, dev.device_type, dev.bus_info);
    }
    println!("----------------------------------");
}
