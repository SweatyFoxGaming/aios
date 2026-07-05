//! Physical frame allocator for Phoenix OS.

use multiboot2::{MemoryArea, MemoryAreaType, MemoryMapTag};
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

extern "C" {
    /// Physical address just past the kernel's own loaded image, defined in
    /// linker.ld. Only the ADDRESS of this symbol is meaningful -- it has no
    /// real storage/value, so it's never read as data.
    static kernel_physical_end: u8;
}

/// A simple frame allocator that uses a bitmap or a similar structure.
/// For now, we implement a basic "Bump" allocator based on the memory map.
pub struct BootFrameAllocator {
    mmap: &'static MemoryMapTag,
    next_entry: usize,
    next_offset: u64,
    /// Frames below this physical address are never handed out: Multiboot2's
    /// raw memory map (unlike Limine's) doesn't exclude the kernel's own
    /// loaded image from "usable" regions, so without this the allocator
    /// would hand out frames the kernel itself lives in -- silently
    /// corrupting live code/data/page-tables as soon as they're written to.
    reserved_below: u64,
}

impl BootFrameAllocator {
    /// Create a new frame allocator from the Multiboot2 memory map.
    ///
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid.
    #[must_use]
    pub unsafe fn init(mmap: &'static MemoryMapTag) -> Self {
        Self {
            mmap,
            next_entry: 0,
            next_offset: 0,
            reserved_below: core::ptr::addr_of!(kernel_physical_end) as u64,
        }
    }

    /// Finds the next usable frame in the memory map.
    fn find_next_frame(&mut self) -> Option<PhysFrame> {
        let entries: &[MemoryArea] = self.mmap.memory_areas();

        while self.next_entry < entries.len() {
            let entry = &entries[self.next_entry];
            if MemoryAreaType::from(entry.typ()) == MemoryAreaType::Available
                && self.next_offset < entry.size()
            {
                let frame_addr = entry.start_address() + self.next_offset;
                self.next_offset += 4096;
                if frame_addr < self.reserved_below {
                    continue;
                }
                return Some(PhysFrame::containing_address(PhysAddr::new(frame_addr)));
            }
            // Move to next entry
            self.next_entry += 1;
            self.next_offset = 0;
        }
        None
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.find_next_frame()
    }
}
