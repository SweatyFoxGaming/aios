//! Physical frame allocator for Phoenix OS.

use limine::MemmapResponse;
use limine::MemoryMapEntryType;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

/// A simple frame allocator that uses a bitmap or a similar structure.
/// For now, we implement a basic "Bump" allocator based on the memory map.
pub struct BootFrameAllocator {
    mmap: &'static MemmapResponse,
    next_entry: usize,
    next_offset: u64,
}

impl BootFrameAllocator {
    /// Create a new frame allocator from the Limine memory map.
    ///
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid.
    #[must_use]
    pub const unsafe fn init(mmap: &'static MemmapResponse) -> Self {
        Self {
            mmap,
            next_entry: 0,
            next_offset: 0,
        }
    }

    /// Finds the next usable frame in the memory map.
    fn find_next_frame(&mut self) -> Option<PhysFrame> {
        let entries = self.mmap.memmap();

        while self.next_entry < entries.len() {
            let entry = &entries[self.next_entry];
            if entry.typ == MemoryMapEntryType::Usable && self.next_offset < entry.len {
                let frame_addr = entry.base + self.next_offset;
                self.next_offset += 4096;
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
