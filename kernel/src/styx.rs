//! Styx: Userspace transition and Ring 3 management for Phoenix OS.

use crate::arch::x86_64::gdt;
use x86_64::instructions::interrupts;

/// Jump to userspace (Ring 3).
///
/// # Safety
/// This function is unsafe as it performs a direct transition to a lower privilege level
/// and assumes the existence of valid user-mode stack and code.
pub unsafe fn jump_to_userspace(entry_point: u64, stack_ptr: u64) -> ! {
    let selectors = gdt::get_selectors();

    // Disable interrupts before transition
    interrupts::disable();

    // Assemble the iretq stack frame
    // [SS] (User Data)
    // [RSP] (User Stack Pointer)
    // [RFLAGS] (Interrupts Enabled | Fixed bit 1)
    // [CS] (User Code)
    // [RIP] (Entry Point)

    // Using inline assembly for the transition
    core::arch::asm!(
        "push {ss}",
        "push {rsp}",
        "push 0x202", // RFLAGS with IF (interrupts) enabled
        "push {cs}",
        "push {rip}",
        "iretq",
        ss = in(reg) u64::from(selectors.user_data_selector.0),
        rsp = in(reg) stack_ptr,
        cs = in(reg) u64::from(selectors.user_code_selector.0),
        rip = in(reg) entry_point,
        options(noreturn)
    );
}
