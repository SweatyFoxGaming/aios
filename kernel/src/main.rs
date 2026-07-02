//! Phoenix OS Kernel.
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner::runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

extern crate alloc;

/// Architecture-specific code.
pub mod arch;
/// Audit logging.
pub mod audit;
/// Hardware drivers.
pub mod drivers;
/// Self-awareness and identity.
pub mod ego;
/// Neural event bus.
pub mod events;
/// Memory management.
pub mod mem;
mod panic;
/// Resource governor.
pub mod pulse;
/// Process scheduling.
pub mod sched;
/// Serial communication.
pub mod serial;
/// Service registry and discovery.
pub mod services;
/// Synapse IPC.
pub mod synapse;
mod test_runner;

use alloc::string::ToString;
use common::addr::PhysAddr;
use common::security::{Capability, Token};
use common::synapse::Message;
use limine::{FramebufferRequest, HhdmRequest, MemmapRequest};
use x86_64::VirtAddr;

// Limine requests
#[used]
#[link_section = ".limine_reqs"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new(0);

#[used]
#[link_section = ".limine_reqs"]
static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new(0);

#[used]
#[link_section = ".limine_reqs"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new(0);

/// Kernel entry point.
///
/// # Panics
/// Panics if the HHDM request fails or memory management initialization fails.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    use alloc::boxed::Box;

    println!("Phoenix OS Kernel booting...");

    // 1. Initialize architecture (GDT, IDT) - does not require heap
    arch::init();
    println!("Architecture initialized (GDT, IDT).");

    // 2. Discover physical memory layout and HHDM
    let phys_mem_offset = HHDM_REQUEST.get_response().get().map_or_else(
        || {
            panic!("HHDM request failed");
        },
        |hhdm_response| VirtAddr::new(hhdm_response.offset),
    );

    let mmap_response = MEMORY_MAP_REQUEST
        .get_response()
        .get()
        .expect("Memory map request failed");

    // 3. Initialize memory management (Physical Frame Allocator, Virtual Paging, and Heap)
    let mut frame_allocator = unsafe { mem::frame::BootFrameAllocator::init(mmap_response) };
    let mut mapper = unsafe { mem::paging::init(phys_mem_offset) };
    mem::heap::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");

    println!("Memory management initialized (Physical, Virtual, Heap).");

    // 4. Gather hardware identity - requires heap for String
    let hardware_fp = arch::x86_64::fingerprint::gather();

    // 5. Setup Security Tokens - requires heap if using Vec or complex types
    let mut kernel_token = Token::empty(0);
    kernel_token.grant(Capability::ServiceRegister);
    kernel_token.grant(Capability::MemAlloc);
    kernel_token.grant(Capability::MemMap);
    kernel_token.grant(Capability::SerialWrite);
    kernel_token.grant(Capability::AuditWrite);
    kernel_token.grant(Capability::HardwareInfo);

    // 6. Register foundational services - requires heap for ServiceRegistry (Vec)
    let _ = services::register("KernelCore", 1, &kernel_token);
    let _ = services::register("LogService", 1, &kernel_token);
    let _ = services::register_secure("MemoryService", 1, Capability::MemAlloc, &kernel_token);
    let _ = services::register("ProcessService", 1, &kernel_token);
    let _ = services::register_secure(
        "HardwareIdentityService",
        1,
        Capability::HardwareInfo,
        &kernel_token,
    );
    let _ = services::register("SynapseService", 1, &kernel_token);
    let _ = services::register("EgoService", 1, &kernel_token);
    let _ = services::register("PulseService", 1, &kernel_token);
    let _ = services::register("DisplayService", 1, &kernel_token);

    // 7. Initialize scheduler
    sched::init();
    println!("Scheduler initialized.");

    // Check for framebuffer and initialize Aura
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response().get() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().first() {
            println!(
                "Framebuffer found: {}x{}. Initializing Aura...",
                framebuffer.width, framebuffer.height
            );
            drivers::display::init(framebuffer);
        }
    }

    // Set final ego state
    ego::set_state(ego::PresenceState::Idle);

    // Start resource monitoring
    pulse::monitor();

    // Publish high-significance boot event
    events::publish("Kernel Boot Sequence Complete".to_string(), 0.9);

    // Test Synapse IPC
    synapse::send(Message::new(
        "KernelCore",
        "AuditLog",
        "Initial Synapse Probe".to_string(),
    ));

    // Final initialization logs
    arch::x86_64::fingerprint::log_info(&hardware_fp);
    services::list_services();
    audit::print_logs();
    events::list_events();
    synapse::debug_bus();
    ego::log_status();
    pulse::log_status();

    let initial_addr = PhysAddr(0x1000);
    println!("Initial address verified: {:?}", initial_addr);

    // Verify heap works
    let heap_value = Box::new(42);
    println!("Heap allocation verification: Boxed value = {}", heap_value);

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
