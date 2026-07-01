//! Hardware fingerprinting for Phoenix OS.
//! Gathers CPU features and machine metadata to help JARVIS optimize its performance.

use crate::println;
use alloc::string::String;
use alloc::string::ToString;
use raw_cpuid::CpuId;

/// Represents the hardware identity of the system.
pub struct HardwareFingerprint {
    /// CPU vendor string.
    pub cpu_vendor: String,
    /// Whether the CPU supports 1GB pages.
    pub has_huge_pages: bool,
}

/// Gathers the hardware fingerprint of the current machine.
#[must_use]
pub fn gather() -> HardwareFingerprint {
    let cpuid = CpuId::new();

    let cpu_vendor = cpuid
        .get_vendor_info()
        .map_or_else(|| "Unknown".to_string(), |vi| vi.as_str().to_string());

    let has_huge_pages = cpuid
        .get_extended_processor_and_feature_identifiers()
        .is_some_and(|fi| fi.has_1gib_pages());

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
