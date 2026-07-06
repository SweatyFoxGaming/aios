//! Heap allocator for Phoenix OS.

use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
};
use x86_64::VirtAddr;

/// Start address of the kernel heap.
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// Size of the kernel heap. Needs to comfortably fit the in-memory RAM disk
/// (4MiB) plus everything else the kernel allocates.
pub const HEAP_SIZE: usize = 32 * 1024 * 1024; // 32 MiB

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize the kernel heap.
///
/// # Errors
/// Returns `MapToError::FrameAllocationFailed` if no physical frame could be allocated.
/// Returns other `MapToError` if mapping the page failed.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    // Basic ASLR for the heap: add a random offset if an entropy source were available.
    // For now, we use a fixed offset as a placeholder for the logic.
    let aslr_offset: u64 = 0; // TODO: Implement RDRAND for true ASLR
    let heap_start_addr = VirtAddr::new(HEAP_START as u64 + aslr_offset);

    let page_range = {
        let heap_end = heap_start_addr + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start_addr);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        // Security: Set NO_EXECUTE to prevent code execution from the heap.
        // NOTE: requires EFER.NXE to be set (see boot32.asm's enable_paging) --
        // without it this bit is reserved in the PTE and setting it is a
        // hardware-level "malformed page table" fault, not a real mapping bug.
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;

        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR
            .lock()
            .init(heap_start_addr.as_mut_ptr(), HEAP_SIZE);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::boxed::Box;
    use crate::safe_alloc;

    #[test_case]
    fn heap_allocation_round_trip() {
        let b = Box::new(12345u64);
        assert_eq!(*b, 12345);
        drop(b);
    }

    #[test_case]
    fn heap_handles_large_zeroed_vec() {
        // Exercises heap + the safe_alloc memset-crash workaround together
        // at a size well beyond the documented ~192-byte threshold.
        let buf = safe_alloc::zeroed_vec(64 * 1024);
        assert_eq!(buf.len(), 64 * 1024);
    }

    #[test_case]
    fn heap_constants_match_documented_values() {
        assert_eq!(HEAP_START, 0x_4444_4444_0000);
        assert_eq!(HEAP_SIZE, 32 * 1024 * 1024);
    }
}
