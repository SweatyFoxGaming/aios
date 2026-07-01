//! `x86_64` architecture-specific code.
pub mod gdt;
pub mod interrupts;

/// Initialize the `x86_64` architecture.
pub fn init() {
    gdt::init();
    interrupts::init_idt();
}
