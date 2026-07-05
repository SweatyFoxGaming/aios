//! Driver Proxy for Phoenix OS.
//! Facilitates communication with Ring 3 drivers via Synapse IPC.

use crate::drivers::manager::{Device, DeviceType};
use crate::println;
use crate::safe_alloc::{concat2, to_string};
use alloc::string::String;

/// Represents a proxy for a driver running in userspace.
pub struct DriverProxy {
    /// Name of the device.
    pub device_name: String,
    /// ID of the capability token assigned to this driver.
    pub capability_token_id: u64,
}

impl DriverProxy {
    /// Create a new driver proxy.
    #[must_use]
    pub fn new(device_name: String, token_id: u64) -> Self {
        Self {
            device_name,
            capability_token_id: token_id,
        }
    }

    /// Forward a command to the userspace driver.
    pub fn dispatch_command(&self, command: &str) {
        println!(
            "[DriverProxy] Forwarding command to {}: {}",
            self.device_name, command
        );

        // Use Synapse to send a message to the Ring 3 driver
        crate::synapse::send(common::synapse::Message::new(
            "KernelProxy",
            "UserSpaceDriver",
            concat2("Command: ", command),
        ));
    }
}

/// Initialize Stark Isolation for a specific device.
pub fn isolate_device(name: &str, device_type: DeviceType) {
    println!(
        "[Stark Isolation] Moving device {name} [{device_type:?}] to Ring 3 proxy..."
    );

    let proxy = DriverProxy::new(to_string(name), 0);
    proxy.dispatch_command("INIT_ISOLATED");

    // Register the proxy as the authoritative handle for this device
    crate::drivers::manager::register(Device {
        name: concat2("Proxy-", name),
        device_type,
        bus_info: Some(to_string("Isolated via Stark")),
    });
}
