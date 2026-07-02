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
    let mut disk = RamDisk {
        data: Vec::with_capacity(DISK_SIZE),
    };
    disk.data.resize(DISK_SIZE, 0);
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

    buffer.copy_from_slice(&disk.data[start..start + buffer.len()]);
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

    disk.data[start..start + buffer.len()].copy_from_slice(buffer);
    Ok(())
}
