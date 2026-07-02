//! `x86_64` architecture-specific code.
pub mod fingerprint;
pub mod gdt;
pub mod interrupts;
pub mod pci;
pub mod time;

/// Initialize the `x86_64` architecture.
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    time::init(100); // 100 Hz (10ms ticks)
}
