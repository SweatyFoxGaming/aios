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
/// Workarounds for a toolchain issue affecting heap string/vec append ops.
pub mod safe_alloc;
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

use common::addr::PhysAddr;
use common::security::{Capability, Token};
use common::synapse::Message;
use multiboot2::BootInformation;
use x86_64::VirtAddr;

/// Virtual offset at which the 32-bit bootstrap (`boot32.asm`) identity-maps
/// all physical memory a second time (in addition to the identity mapping at
/// offset 0), giving us an HHDM-style "physical memory offset" the way
/// Limine used to hand us directly -- except this time we built it
/// ourselves, so the constant has to match `KERNEL_VMA_OFFSET` in both
/// `boot32.asm` and `linker.ld` exactly.
const PHYS_MEM_OFFSET: u64 = 0xffff_ffff_8000_0000;

/// Kernel entry point, called from `boot32.asm`'s `long_mode_start` once
/// paging and long mode are active. `multiboot_info_addr` is the physical
/// (identity-mapped, so also valid as a direct pointer) address of the
/// Multiboot2 boot information structure GRUB left for us, passed through
/// from EBX by the assembly bootstrap.
///
/// # Panics
/// Panics if the Multiboot2 info is invalid, no memory map tag is present,
/// or memory management initialization fails.
#[no_mangle]
pub extern "C" fn kernel_main_entry(multiboot_info_addr: usize) -> ! {
    use alloc::boxed::Box;

    println!("Phoenix OS Kernel booting...");
    println!("multiboot_info_addr = {:#x}", multiboot_info_addr);

    // 1. Initialize architecture (GDT, IDT, Time) - does not require heap
    arch::init();
    println!("Architecture initialized (GDT, IDT, PIT).");

    // Initialize the scheduler immediately after arch init, before anything
    // else -- interrupts are never explicitly disabled anywhere in this
    // kernel, so IF may already be set (inherited from GRUB) and the PIT is
    // now running at 100Hz. If the timer fires before this, SCHEDULER's
    // lazy_static would perform its first-ever initialization from inside
    // the interrupt handler, which reliably crashed with a nondeterministic
    // CPU exception. Doing it here, in ordinary code, avoids that entirely.
    sched::init();

    // 2. Parse the Multiboot2 boot information GRUB handed us, and pull the
    // memory map out of it (Multiboot2 has no HHDM concept the way Limine
    // did -- PHYS_MEM_OFFSET above is our own convention, built into the
    // page tables in boot32.asm, not something the bootloader gives us).
    let boot_info = unsafe {
        BootInformation::load(multiboot_info_addr as *const multiboot2::BootInformationHeader)
            .expect("Invalid Multiboot2 boot information")
    };
    let phys_mem_offset = VirtAddr::new(PHYS_MEM_OFFSET);

    // SAFETY: the Multiboot2 info structure lives in GRUB-provided physical
    // memory that nothing else claims or overwrites during the kernel's
    // lifetime (same assumption the original Limine `'static` responses
    // relied on) -- extending the borrow to 'static is sound here.
    let mmap_tag: &'static multiboot2::MemoryMapTag = unsafe {
        core::mem::transmute(
            boot_info
                .memory_map_tag()
                .expect("Multiboot2 memory map tag missing"),
        )
    };

    // 3. Initialize memory management (Physical Frame Allocator, Virtual Paging, and Heap)
    let mut frame_allocator = unsafe { mem::frame::BootFrameAllocator::init(mmap_tag) };
    let mut mapper = unsafe { mem::paging::init(phys_mem_offset) };
    mem::heap::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");

    println!("Memory management initialized (Physical, Virtual, Heap).");

    // Initialize APIC after memory is ready
    unsafe {
        arch::x86_64::apic::init(phys_mem_offset);
    }

    // Verify heap works
    let heap_value = Box::new(42);
    println!("Heap allocation verification: Boxed value = {}", heap_value);
    let initial_addr = PhysAddr(0x1000);
    println!("Initial address verified: {:?}", initial_addr);

    println!("Phoenix OS boot milestone reached: arch + memory management fully initialized.");

    let hardware_fp = arch::x86_64::fingerprint::gather();

    arch::x86_64::morph::morph(&hardware_fp);
    aegis::init();
    let _ = vault::init();

    let mut kernel_token = Token::empty(0);
    kernel_token.grant(Capability::ServiceRegister);
    kernel_token.grant(Capability::MemAlloc);
    kernel_token.grant(Capability::MemMap);
    kernel_token.grant(Capability::SerialWrite);
    kernel_token.grant(Capability::AuditWrite);
    kernel_token.grant(Capability::HardwareInfo);

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
    let _ = services::register_decoy("RestrictedDebugService", &kernel_token);
    let _ = services::register_decoy("GlobalMemoryWrite", &kernel_token);

    if let Some(Ok(fb_tag)) = boot_info.framebuffer_tag() {
        let framebuffer = drivers::display::Framebuffer {
            address: fb_tag.address(),
            pitch: u64::from(fb_tag.pitch()),
            width: u64::from(fb_tag.width()),
            height: u64::from(fb_tag.height()),
        };
        println!(
            "Framebuffer found: {}x{}. Initializing Aura...",
            framebuffer.width, framebuffer.height
        );
        drivers::display::init(framebuffer);
        drivers::display::engine::init();
        drivers::display::transcendent::init();
    }

    arch::x86_64::pci::scan_bus();

    drivers::input::mouse::init();
    drivers::storage::ramdisk::init();
    fs::phoenixfs::init();
    net::init();
    pkg::init();
    updater::check();
    calliope::init();

    ego::set_state(ego::PresenceState::Idle);

    let node_id = mnemosyne::add_node(safe_alloc::to_string("Phoenix Project"));
    let sub_id = mnemosyne::add_node(safe_alloc::to_string("Kernel Implementation"));
    mnemosyne::add_relation(sub_id, node_id, common::memory::Relation::PartOf);

    let raw_input = "research solid state batteries";
    let intent = hermes::parse(raw_input);
    hermes::dispatch(&intent);

    pulse::monitor();
    vesta::check_health();

    events::publish("Kernel Boot Sequence Complete", 0.9f32.to_bits());

    synapse::send(Message::new(
        "KernelCore",
        "AuditLog",
        safe_alloc::to_string("Initial Synapse Probe"),
    ));

    let _ = syscall::handle_syscall(1, 0, 0);
    let _ = aegis::verify();

    sched::process::load("Shell", alloc::vec![0x90, 0x90, 0x90]);
    drivers::display::aura::render_emblem();
    fs::shell::start();

    let mock_user_token = Token::empty(100);
    let _ = services::find("RestrictedDebugService", &mock_user_token);

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
