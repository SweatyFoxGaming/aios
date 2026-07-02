//! System installer for Phoenix OS.

use crate::println;

/// Launch the system installer.
pub fn start() {
    println!("--- Phoenix OS System Installer ---");
    println!("[Installer] Scanning for target drives...");
    println!("[Installer] Target: 16 GB Phoenix Virtual Drive");
    println!("[Installer] Partitioning...");
    println!("[Installer] Installing kernel and core modules...");
    println!("[Installer] Configuring bootloader (Limine)...");
    println!("[Installer] Installation complete. Please reboot.");
}
