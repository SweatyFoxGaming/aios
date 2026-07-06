//! QEMU `isa-debug-exit` device support for the kernel's `#[test_case]`
//! harness (see `test_runner.rs`). Writing a value to I/O port `0xf4`, with
//! the `-device isa-debug-exit,iobase=0xf4,iosize=0x04` QEMU flag present,
//! makes QEMU exit with process code `(value << 1) | 1` -- `Success` (0x10)
//! -> exit 33, `Failed` (0x11) -> exit 35. This device is declared purely on
//! the QEMU command line (see `scripts/test_kernel.sh`); it needs no
//! Cargo.toml/linker changes and has no effect on the production ISO, which
//! never passes that flag.
//!
//! This module is compiled unconditionally (not `#[cfg(test)]`), same as
//! `test_runner`: it's only ever called from `test_runner::runner`, which is
//! itself dead code outside a test build, but `test_runner` is declared
//! unconditionally in `main.rs` (the `#![test_runner(...)]` attribute needs
//! it to always resolve), so its callees must compile unconditionally too.

use x86_64::instructions::port::PortWriteOnly;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

#[allow(dead_code)]
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    unsafe {
        let mut port: PortWriteOnly<u32> = PortWriteOnly::new(0xf4);
        port.write(exit_code as u32);
    }
    // Fallback only -- isa-debug-exit should already have terminated QEMU.
    loop {
        x86_64::instructions::hlt();
    }
}
