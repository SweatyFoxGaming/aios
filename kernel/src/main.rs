//! Phoenix OS Kernel.
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner::runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

/// Architecture-specific code.
pub mod arch;
/// Memory management.
pub mod mem;
mod panic;
/// Serial communication.
pub mod serial;
/// Service registry and discovery.
pub mod services;
mod test_runner;

use common::addr::PhysAddr;
use limine::{FramebufferRequest, MemmapRequest};
use x86_64::structures::paging::FrameAllocator;

// Limine requests
#[used]
#[link_section = ".limine_reqs"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new(0);

#[used]
#[link_section = ".limine_reqs"]
static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new(0);

/// Kernel entry point.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Phoenix OS Kernel booting...");

    // Initialize architecture
    arch::init();
    println!("Architecture initialized (GDT, IDT).");

    // Register foundational services
    let _ = services::register("KernelCore", 1);
    let _ = services::register("LogService", 1);
    let _ = services::register("MemoryService", 1);
    services::list_services();

    // Check for framebuffer
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response().get() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().first() {
            println!(
                "Framebuffer found: {}x{}",
                framebuffer.width, framebuffer.height
            );
        }
    }

    // Check for memory map
    if let Some(mmap_response) = MEMORY_MAP_REQUEST.get_response().get() {
        println!(
            "Memory map found with {} entries",
            mmap_response.entry_count
        );

        // Initialize frame allocator
        let mut frame_allocator = unsafe { mem::frame::BootFrameAllocator::init(mmap_response) };
        println!("Physical memory management initialized.");

        // Test allocation
        if let Some(frame) = frame_allocator.allocate_frame() {
            println!("Test allocation successful: {:?}", frame);
        }
    }

    let initial_addr = PhysAddr(0x1000);
    println!("Initial address verified: {:?}", initial_addr);

    #[cfg(test)]
    test_main();

    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1, 1);
    }
}
