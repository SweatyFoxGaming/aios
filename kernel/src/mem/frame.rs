//! Physical frame allocator for Phoenix OS.

use limine::MemmapResponse;
use limine::MemoryMapEntryType;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

/// A simple frame allocator that uses a bitmap or a similar structure.
/// For now, we implement a basic "Bump" allocator based on the memory map.
pub struct BootFrameAllocator {
    mmap: &'static MemmapResponse,
    next: usize,
}

impl BootFrameAllocator {
    /// Create a new frame allocator from the Limine memory map.
    ///
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid.
    #[must_use]
    pub const unsafe fn init(mmap: &'static MemmapResponse) -> Self {
        Self { mmap, next: 0 }
    }

    /// Returns an iterator over the usable frames in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.mmap.memmap().iter();
        let usable_regions = regions.filter(|r| r.typ == MemoryMapEntryType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.base..(r.base + r.len));
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
