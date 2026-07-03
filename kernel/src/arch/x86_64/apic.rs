//! APIC (Advanced Programmable Interrupt Controller) support for Phoenix OS.

use x86_64::registers::model_specific::Msr;
use x86_64::VirtAddr;

/// Base address of the Local APIC.
pub const LAPIC_BASE: u64 = 0xFEE0_0000;

/// Initialize the Local APIC.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the
/// `physical_memory_offset` is correct and that APIC is present.
pub unsafe fn init(physical_memory_offset: VirtAddr) {
    // Enable APIC globally via MSR
    let mut apic_base_msr = Msr::new(0x1B);
    let mut base = apic_base_msr.read();
    base |= 1 << 11; // Enable bit
    apic_base_msr.write(base);

    // Map LAPIC base address in virtual memory
    let lapic_virt = physical_memory_offset + LAPIC_BASE;
    crate::println!(
        "[APIC] Local APIC enabled at virtual address: {:?}",
        lapic_virt
    );

    // TODO: Implement register-level initialization (Spurious IV, Timer)
}

/// Acknowledge an interrupt.
///
/// # Safety
/// This function is unsafe because it interacts with memory-mapped I/O.
pub unsafe fn end_of_interrupt() {
    // In a real implementation, we would write 0 to the EOI register (offset 0xB0)
}
