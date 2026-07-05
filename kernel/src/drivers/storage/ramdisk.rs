//! RAM disk storage driver for Phoenix OS.

use crate::println;
use alloc::vec::Vec;
use spin::Mutex;

const SECTOR_SIZE: usize = 512;
const DISK_SIZE: usize = 1024 * 1024 * 4; // 4 MB RamDisk

struct RamDisk {
    data: Vec<u8>,
}

static RAM_DISK: Mutex<Option<RamDisk>> = Mutex::new(None);

/// Initialize the RAM disk.
pub fn init() {
    println!("[Hephaestus] Initializing RAM Disk (4MB)...");
    let mut data: Vec<u8> = Vec::with_capacity(DISK_SIZE);
    // `Vec::resize(DISK_SIZE, 0)` lowers to a single bulk `memset` call for a
    // Copy element, which crashes on this target above a small size
    // threshold (see safe_alloc.rs) -- zero byte-by-byte instead, which
    // never triggers that codegen path.
    unsafe {
        let ptr = data.as_mut_ptr();
        for i in 0..DISK_SIZE {
            ptr.add(i).write(0);
        }
        data.set_len(DISK_SIZE);
    }
    let disk = RamDisk { data };
    *RAM_DISK.lock() = Some(disk);
}

/// Read from the RAM disk.
pub fn read(sector: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
    let disk_lock = RAM_DISK.lock();
    let disk = disk_lock.as_ref().ok_or("RamDisk not initialized")?;

    let start = sector * SECTOR_SIZE;
    if start + buffer.len() > disk.data.len() {
        return Err("Read out of bounds");
    }

    // `copy_from_slice` lowers to a bulk `memcpy` call, which crashes on
    // this target above a small size threshold (see safe_alloc.rs) --
    // byte-by-byte instead, which never triggers that codegen path.
    for i in 0..buffer.len() {
        buffer[i] = disk.data[start + i];
    }
    Ok(())
}

/// Write to the RAM disk.
pub fn write(sector: usize, buffer: &[u8]) -> Result<(), &'static str> {
    let mut disk_lock = RAM_DISK.lock();
    let disk = disk_lock.as_mut().ok_or("RamDisk not initialized")?;

    let start = sector * SECTOR_SIZE;
    if start + buffer.len() > disk.data.len() {
        return Err("Write out of bounds");
    }

    // See read() above: byte-by-byte to avoid the bulk `memcpy` codegen path.
    for (i, &byte) in buffer.iter().enumerate() {
        disk.data[start + i] = byte;
    }
    Ok(())
}
