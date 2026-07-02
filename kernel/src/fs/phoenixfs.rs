//! PhoenixFS: A simple native filesystem for Phoenix OS.

use crate::println;
use crate::drivers::storage::ramdisk;
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

const MAGIC: u32 = 0x50484E58; // "PHNX"

#[repr(C)]
struct Superblock {
    magic: u32,
    version: u32,
    total_sectors: u32,
    root_dir_sector: u32,
}

/// Initialize the native filesystem.
pub fn init() {
    println!("[Iris] Initializing PhoenixFS...");

    let mut sb_buffer = [0u8; 512];
    if ramdisk::read(0, &mut sb_buffer).is_ok() {
        let magic = u32::from_le_bytes([sb_buffer[0], sb_buffer[1], sb_buffer[2], sb_buffer[3]]);
        if magic == MAGIC {
            println!("[Iris] PhoenixFS volume detected.");
        } else {
            println!("[Iris] No PhoenixFS volume found. Formatting...");
            format();
        }
    }
}

fn format() {
    let sb = Superblock {
        magic: MAGIC,
        version: 1,
        total_sectors: 8192, // 4MB / 512
        root_dir_sector: 1,
    };

    let mut buffer = [0u8; 512];
    buffer[0..4].copy_from_slice(&sb.magic.to_le_bytes());
    buffer[4..8].copy_from_slice(&sb.version.to_le_bytes());
    buffer[8..12].copy_from_slice(&sb.total_sectors.to_le_bytes());
    buffer[12..16].copy_from_slice(&sb.root_dir_sector.to_le_bytes());

    let _ = ramdisk::write(0, &buffer);
    println!("[Iris] PhoenixFS format complete.");
}
