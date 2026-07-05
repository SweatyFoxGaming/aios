//! Workaround for a toolchain/codegen issue where a bulk `memcpy`/`memset`
//! -- as emitted for `Vec`/`String` growth (`extend_from_slice`, `push_str`,
//! `alloc::format!`) or for zero-initializing a stack array/struct field
//! above roughly 128-192 bytes (e.g. `[0u8; 512]`) -- crashes with a
//! nondeterministic CPU exception (divide error, invalid opcode, page
//! fault; a different one each run) on this bare-metal, `no_std`
//! freestanding target.
//!
//! Root-caused via extensive isolation:
//! - A single, pre-sized, *uninitialized* allocation (`Vec::with_capacity`,
//!   `Box::new`) always works; direct pointer writes into it always work.
//! - A stack array literal zero-init (`[0; N]`) works for N up to ~128
//!   bytes, and fails for N at 192 and above -- consistent with LLVM's
//!   threshold for lowering array-zeroing to a `memset` call rather than
//!   inline stores.
//! - `core::ptr::copy_nonoverlapping`/`__cpuid_count`/hand-written
//!   `rep movsb` all work when called directly; `Vec::extend_from_slice`
//!   (which goes through the same `memcpy` under the hood via a deeper,
//!   specialization-dispatched call chain) reliably fails.
//!
//! The common thread is calls into `compiler_builtins::mem::{memcpy,memset}`
//! for buffers above that threshold; below it, LLVM inlines the copy/zero as
//! plain stores and everything works. The helpers below stay under that
//! threshold everywhere, and specifically never zero-initialize a large
//! buffer: `Vec::with_capacity` allocates without zeroing, and every byte
//! actually used is written explicitly one at a time.
//!
//! `println!`/`print!` are unaffected by any of this since they write
//! directly to the serial port via `format_args!`, never touching the heap.

use alloc::string::String;
use alloc::vec::Vec;

/// Returns a heap-allocated, zeroed buffer of `n` bytes, safe for any size.
///
/// Deliberately a `Vec<u8>`, not a stack `[u8; N]`: returning a large array
/// by value requires copying it out of the callee's frame into the
/// caller's, which -- even when the array's *contents* were zeroed
/// byte-by-byte -- still lowers to a bulk `memcpy` for the return itself,
/// crashing on this target (see module docs). `Vec::with_capacity` doesn't
/// zero memory at all, so the explicit byte loop below is the only write
/// that happens, and the `Vec`'s buffer is heap-allocated once and moved
/// (pointer-sized), never bulk-copied.
#[must_use]
pub fn zeroed_vec(n: usize) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(n);
    unsafe {
        let ptr = buf.as_mut_ptr();
        for i in 0..n {
            ptr.add(i).write(0);
        }
        buf.set_len(n);
    }
    buf
}

/// Safe replacement for `dst.copy_from_slice(src)`.
///
/// `[T]::copy_from_slice` reliably crashes on this target -- same failure
/// mode as `Vec::extend_from_slice` (see module docs): its real,
/// specialization-dispatched implementation fails even for a 4-byte copy,
/// while this identical byte-by-byte replica (and a direct, non-dispatched
/// `core::ptr::copy_nonoverlapping` call) always works. Panics if lengths
/// differ, matching `copy_from_slice`'s contract.
pub fn copy_from_slice(dst: &mut [u8], src: &[u8]) {
    assert_eq!(dst.len(), src.len());
    for i in 0..dst.len() {
        dst[i] = src[i];
    }
}

/// Upper bound on any single string this kernel builds via these helpers.
/// Kept below the size at which LLVM lowers array-zeroing/bulk-copy to a
/// `memcpy`/`memset` call (safe up to ~128 bytes; 192 already fails).
const MAX_LEN: usize = 128;

/// Builds a `String` by writing UTF-8 bytes directly into a single,
/// fixed-capacity, uninitialized buffer -- no zeroing, no
/// `push`/`extend_from_slice`/reallocation ever occurs.
fn build(f: impl FnOnce(&mut [u8]) -> usize) -> String {
    let mut buf: Vec<u8> = Vec::with_capacity(MAX_LEN);
    let written = unsafe {
        let slice = core::slice::from_raw_parts_mut(buf.as_mut_ptr(), MAX_LEN);
        let n = f(slice).min(MAX_LEN);
        buf.set_len(n);
        n
    };
    debug_assert!(written <= MAX_LEN);
    // SAFETY: `f` only ever writes valid UTF-8 (raw ASCII bytes or
    // pre-validated str fragments in every call site in this kernel).
    unsafe { String::from_utf8_unchecked(buf) }
}

/// Safe replacement for `s.to_string()` / `String::from(s)`.
pub fn to_string(s: &str) -> String {
    build(|dst| {
        let bytes = s.as_bytes();
        let n = bytes.len().min(dst.len());
        for i in 0..n {
            dst[i] = bytes[i];
        }
        n
    })
}

/// Safe replacement for `a.to_string() + b`.
pub fn concat2(a: &str, b: &str) -> String {
    build(|dst| {
        let mut i = 0;
        for &byte in a.as_bytes().iter().chain(b.as_bytes()) {
            if i >= dst.len() {
                break;
            }
            dst[i] = byte;
            i += 1;
        }
        i
    })
}

/// Safe replacement for `a.to_string() + b + c`.
pub fn concat3(a: &str, b: &str, c: &str) -> String {
    build(|dst| {
        let mut i = 0;
        for &byte in a.as_bytes().iter().chain(b.as_bytes()).chain(c.as_bytes()) {
            if i >= dst.len() {
                break;
            }
            dst[i] = byte;
            i += 1;
        }
        i
    })
}

/// A `core::fmt::Write` sink over a preallocated, uninitialized buffer --
/// never zero-inits or reallocates. Used by `safe_format!` in place of
/// `alloc::format!`.
pub struct SafeWriter {
    buf: Vec<u8>,
}

impl SafeWriter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(MAX_LEN),
        }
    }

    #[must_use]
    pub fn into_string(self) -> String {
        // SAFETY: only ASCII/UTF-8 written via write_str below.
        unsafe { String::from_utf8_unchecked(self.buf) }
    }
}

impl Default for SafeWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Write for SafeWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &byte in s.as_bytes() {
            if self.buf.len() >= MAX_LEN {
                break;
            }
            // SAFETY: buf was allocated with capacity MAX_LEN and we never
            // exceed it (checked above); writing directly avoids
            // `Vec::push`'s growth path.
            unsafe {
                let len = self.buf.len();
                self.buf.as_mut_ptr().add(len).write(byte);
                self.buf.set_len(len + 1);
            }
        }
        Ok(())
    }
}

/// Safe replacement for `alloc::format!(...)`.
#[macro_export]
macro_rules! safe_format {
    ($($arg:tt)*) => {{
        use core::fmt::Write as _;
        let mut w = $crate::safe_alloc::SafeWriter::new();
        let _ = write!(w, $($arg)*);
        w.into_string()
    }};
}
