//! Morph: Dynamic Silicon Morphing for Phoenix OS.

use crate::arch::x86_64::fingerprint::HardwareFingerprint;
use crate::println;
use spin::RwLock;

/// A function pointer for a "Hot Path" operation (e.g., fast memory copy).
pub type HotPathFn = fn(*mut u8, *const u8, usize);

/// A function pointer for a "Hot Path" fill operation (e.g., fast memory set).
pub type HotPathFillFn = fn(*mut u8, u8, usize);

fn generic_memcpy(dst: *mut u8, src: *const u8, len: usize) {
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, len);
    }
}

fn avx_optimized_memcpy(dst: *mut u8, src: *const u8, len: usize) {
    // Simulated AVX path
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, len);
    }
}

fn generic_memset(dst: *mut u8, val: u8, len: usize) {
    unsafe {
        core::ptr::write_bytes(dst, val, len);
    }
}

fn avx_optimized_memset(dst: *mut u8, val: u8, len: usize) {
    // Simulated AVX path
    unsafe {
        core::ptr::write_bytes(dst, val, len);
    }
}

/// Global dispatch table for morphed functions.
pub struct MorphTable {
    /// Hot-path memory copy function.
    pub memcpy: HotPathFn,
    /// Hot-path memory set function.
    pub memset: HotPathFillFn,
}

static MORPH_TABLE: RwLock<MorphTable> = RwLock::new(MorphTable {
    memcpy: generic_memcpy,
    memset: generic_memset,
});

/// Optimized memory copy via morphed dispatch.
pub fn memcpy(dst: *mut u8, src: *const u8, len: usize) {
    let table = MORPH_TABLE.read();
    (table.memcpy)(dst, src, len);
}

/// Optimized memory set via morphed dispatch.
pub fn memset(dst: *mut u8, val: u8, len: usize) {
    let table = MORPH_TABLE.read();
    (table.memset)(dst, val, len);
}

/// Optimizes the kernel hot-paths based on hardware fingerprint.
pub fn morph(fp: &HardwareFingerprint) {
    println!("[Morph] Analyzing silicon features for optimization...");

    let mut table = MORPH_TABLE.write();
    if fp.has_huge_pages {
        println!("[Morph] Modern silicon detected. Swapping to AVX-optimized paths.");
        table.memcpy = avx_optimized_memcpy;
        table.memset = avx_optimized_memset;
    } else {
        println!("[Morph] Legacy silicon detected. Remaining on generic paths.");
        table.memcpy = generic_memcpy;
        table.memset = generic_memset;
    }
}
