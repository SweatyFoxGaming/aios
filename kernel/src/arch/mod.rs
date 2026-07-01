//! `x86_64` architecture-specific code for Phoenix OS.

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

/// Initialize the current architecture.
pub fn init() {
    #[cfg(target_arch = "x86_64")]
    x86_64::init();
}
