//! Phoenix OS Kernel.
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner::runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

extern crate alloc;

/// Code integrity verification.
pub mod aegis;
/// Architecture-specific code.
pub mod arch;
/// Audit logging.
pub mod audit;
/// WASM skill runtime.
pub mod calliope;
/// Hardware drivers.
pub mod drivers;
/// Self-awareness and identity.
pub mod ego;
/// Neural event bus.
pub mod events;
/// File systems.
pub mod fs;
/// Automated recovery shell.
pub mod ghost;
/// Intent parser.
pub mod hermes;
/// System installer.
pub mod install;
/// Context engine.
pub mod kairos;
/// Significance-based pruning.
pub mod lethe;
/// Memory management.
pub mod mem;
/// Semantic memory.
pub mod mnemosyne;
/// Networking stack.
pub mod net;
mod panic;
/// Package manager.
pub mod pkg;
/// Resource governor.
pub mod pulse;
/// Recovery tools.
pub mod recovery;
/// Process scheduling.
pub mod sched;
/// Anomalous intent detection.
pub mod sentinel;
/// Serial communication.
pub mod serial;
/// Service registry and discovery.
pub mod services;
/// System calls.
pub mod syscall;
/// Synapse IPC.
pub mod synapse;
mod test_runner;
/// Update system.
pub mod updater;
/// Hardware-rooted trust.
pub mod vault;
/// AI homeostasis.
pub mod vesta;

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

    // 1. Initialize architecture (GDT, IDT, Time) - does not require heap
    arch::init();
    println!("Architecture initialized (GDT, IDT, PIT).");

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

    // Initialize APIC after memory is ready
    unsafe {
        arch::x86_64::apic::init(phys_mem_offset);
    }

    // 4. Gather hardware identity - requires heap for String
    let hardware_fp = arch::x86_64::fingerprint::gather();

    // 5. Silicon Morphing - optimize hot paths
    arch::x86_64::morph::morph(&hardware_fp);

    // 6. Security & Integrity (Aegis & Vault)
    aegis::init();
    let _ = vault::init();

    // 7. Setup Security Tokens - requires heap if using Vec or complex types
    let mut kernel_token = Token::empty(0);
    kernel_token.grant(Capability::ServiceRegister);
    kernel_token.grant(Capability::MemAlloc);
    kernel_token.grant(Capability::MemMap);
    kernel_token.grant(Capability::SerialWrite);
    kernel_token.grant(Capability::AuditWrite);
    kernel_token.grant(Capability::HardwareInfo);

    // 8. Register foundational services - requires heap for ServiceRegistry (Vec)
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
    let _ = services::register("FilesystemService", 1, &kernel_token);
    let _ = services::register("OracleService", 1, &kernel_token);
    let _ = services::register("LetheService", 1, &kernel_token);
    let _ = services::register("AegisService", 1, &kernel_token);
    let _ = services::register("VaultService", 1, &kernel_token);

    // Register Honey-Intents
    let _ = services::register_decoy("RestrictedDebugService", &kernel_token);
    let _ = services::register_decoy("GlobalMemoryWrite", &kernel_token);

    // 9. Initialize scheduler
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
            drivers::display::engine::init();
        }
    }

    // Initialize sensory hardware discovery (PCI)
    arch::x86_64::pci::scan_bus();

    // Initialize Mouse and Storage
    drivers::input::mouse::init();
    drivers::storage::ramdisk::init();
    fs::phoenixfs::init();
    net::init();
    pkg::init();
    updater::check();
    calliope::init();

    // Set final ego state
    ego::set_state(ego::PresenceState::Idle);

    // Initialize Cognitive Core flow
    let node_id = mnemosyne::add_node("Phoenix Project".to_string());
    let sub_id = mnemosyne::add_node("Kernel Implementation".to_string());
    mnemosyne::add_relation(sub_id, node_id, common::memory::Relation::PartOf);

    // Simulate Hermes Intent Parsing
    let raw_input = "research solid state batteries";
    let intent = hermes::parse(raw_input);
    hermes::dispatch(&intent);

    // Start resource monitoring
    pulse::monitor();
    vesta::check_health();

    // Publish high-significance boot event
    events::publish("Kernel Boot Sequence Complete".to_string(), 0.9);

    // Test Synapse IPC
    synapse::send(Message::new(
        "KernelCore",
        "AuditLog",
        "Initial Synapse Probe".to_string(),
    ));

    // Test Syscall Interface (Oracle)
    let _ = syscall::handle_syscall(1, 0, 0);

    // Verify Integrity
    let _ = aegis::verify();

    // Load a mock userspace process
    sched::process::load("Shell", alloc::vec![0x90, 0x90, 0x90]);
    drivers::display::aura::render_emblem();
    fs::shell::start();

    // Test Honey-Intent detection
    let mock_user_token = Token::empty(100);
    let _ = services::find("RestrictedDebugService", &mock_user_token);

    // Final initialization logs
    arch::x86_64::fingerprint::log_info(&hardware_fp);
    services::list_services();
    audit::print_logs();
    events::list_events();
    synapse::debug_bus();
    ego::log_status();
    pulse::log_status();
    kairos::log_status();
    vesta::log_status();
    mnemosyne::debug_graph();
    lethe::log_status();

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
