//! ELF loader for Phoenix OS.

use crate::println;
use goblin::elf::Elf;

/// Loads an ELF binary and returns the entry point.
pub fn load_elf(data: &[u8]) -> Result<u64, &'static str> {
    let elf = Elf::parse(data).map_err(|_| "Failed to parse ELF binary")?;

    println!("[Hephaestus] Loading ELF: Entry Point 0x{:x}", elf.entry);

    // In a real implementation, we would iterate over program headers
    // and map them into a new page table.
    for ph in elf.program_headers.iter() {
        if ph.p_type == goblin::elf::program_header::PT_LOAD {
            println!(
                "[Hephaestus] Mapping Segment: 0x{:x} -> 0x{:x} (Size: {})",
                ph.p_paddr, ph.p_vaddr, ph.p_memsz
            );
        }
    }

    Ok(elf.entry)
}
