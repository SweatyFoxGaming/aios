//! Hardware fingerprinting for Phoenix OS.
//! Gathers CPU features and machine metadata to help JARVIS optimize its performance.

use crate::println;
use alloc::string::String;

/// Represents the hardware identity of the system.
pub struct HardwareFingerprint {
    /// CPU vendor string.
    pub cpu_vendor: String,
    /// Whether the CPU supports 1GB pages.
    pub has_huge_pages: bool,
}

/// Result of a single `cpuid` leaf query.
struct CpuidResult {
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
}

/// Executes `cpuid` for the given leaf/sub-leaf.
///
/// Implemented directly via inline assembly rather than the `raw-cpuid`
/// crate: calling into that crate's compiled code at this point in boot
/// reliably crashed with a different CPU exception every run. That turned
/// out to be a symptom of a much broader issue -- see `safe_alloc.rs` --
/// rather than anything specific to `raw-cpuid`. Kept as hand-written asm
/// regardless since it avoids the dependency and was verified reliable.
fn cpuid(leaf: u32, sub_leaf: u32) -> CpuidResult {
    let (eax, ebx, ecx, edx): (u32, u32, u32, u32);
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {ebx_out:e}, ebx",
            "pop rbx",
            inout("eax") leaf => eax,
            ebx_out = out(reg) ebx,
            inout("ecx") sub_leaf => ecx,
            out("edx") edx,
        );
    }
    CpuidResult { eax, ebx, ecx, edx }
}

/// Gathers the hardware fingerprint of the current machine.
#[must_use]
pub fn gather() -> HardwareFingerprint {
    let vendor_leaf = cpuid(0, 0);
    let vendor_bytes: [u8; 12] =
        unsafe { core::mem::transmute([vendor_leaf.ebx, vendor_leaf.edx, vendor_leaf.ecx]) };
    let vendor_str = core::str::from_utf8(&vendor_bytes).unwrap_or("InvalidVendorString");
    let cpu_vendor = crate::safe_alloc::to_string(vendor_str);

    // EAX=0x80000001, EDX bit 26 indicates 1GiB page support (AMD/Intel).
    let ext_features = cpuid(0x8000_0001, 0);
    let has_huge_pages = ext_features.edx & (1 << 26) != 0;

    HardwareFingerprint {
        cpu_vendor,
        has_huge_pages,
    }
}

/// Log hardware information.
pub fn log_info(fp: &HardwareFingerprint) {
    println!("--- Phoenix Hardware Fingerprint ---");
    println!("CPU Vendor: {}", fp.cpu_vendor);
    println!("Supports 1GB Pages: {}", fp.has_huge_pages);
    println!("------------------------------------");
}
