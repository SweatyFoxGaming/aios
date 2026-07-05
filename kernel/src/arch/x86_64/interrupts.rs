//! Interrupt Descriptor Table (IDT) implementation.
use crate::println;
use crate::safe_alloc::{concat2, concat3};
use common::security::Token;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

/// Interrupt indices for the PIC.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    /// PIT Timer interrupt.
    Timer = 32,
    /// Keyboard interrupt.
    Keyboard = 33,
    /// Mouse interrupt.
    Mouse = 44,
}

impl From<InterruptIndex> for u8 {
    fn from(index: InterruptIndex) -> u8 {
        index as u8
    }
}

impl From<InterruptIndex> for usize {
    fn from(index: InterruptIndex) -> usize {
        index as usize
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        // The rest of the CPU exception vectors were previously left
        // "not present" -- any of them firing (e.g. a #GP from an
        // FPU/SSE-related instruction, since this target disables SSE) would
        // hit a missing IDT entry, which itself raises #GP, which ALSO hits
        // a missing handler, escalating straight past our double-fault
        // handler into an unrecoverable triple-fault/CPU-reset loop. That
        // is what was actually behind every "garbage instruction pointer" /
        // cascading-fault symptom seen while debugging fingerprint::gather().
        idt.device_not_available
            .set_handler_fn(device_not_available_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.x87_floating_point
            .set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.simd_floating_point
            .set_handler_fn(simd_floating_point_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(crate::arch::x86_64::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        // Hardware interrupts
        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Mouse as usize].set_handler_fn(mouse_interrupt_handler);

        idt
    };
}

/// Prints the fields of an `InterruptStackFrame` individually.
///
/// The whole-struct `{:#?}` pretty-Debug path causes a cascading re-fault
/// (see page_fault_handler's history) -- always use this instead.
fn print_stack_frame(stack_frame: &InterruptStackFrame) {
    println!("instruction_pointer: {:?}", stack_frame.instruction_pointer);
    println!("code_segment: {:?}", stack_frame.code_segment);
    println!("stack_pointer: {:?}", stack_frame.stack_pointer);
}

/// Initialize the IDT.
pub fn init_idt() {
    IDT.load();
}

/// Helper to log exceptions to the audit log (The Reflective Mechanism).
fn reflective_audit(name: &'static str, details: &str) {
    let system_token = Token::empty(0);
    crate::audit::log(
        &system_token,
        concat2(&concat3("EXCEPTION: ", name, " - "), details),
        "Reflected",
    );
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT");
    print_stack_frame(&stack_frame);
    reflective_audit("Breakpoint", "Handled");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    println!("EXCEPTION: DOUBLE FAULT");
    print_stack_frame(&stack_frame);
    reflective_audit("Double Fault", "CRITICAL");
    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DIVIDE ERROR");
    print_stack_frame(&stack_frame);
    reflective_audit("Divide By Zero", "Handled");
    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: INVALID OPCODE");
    print_stack_frame(&stack_frame);
    reflective_audit("Invalid Opcode", "Handled");
    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DEVICE NOT AVAILABLE (FPU/SSE state accessed without CR0.TS clear)");
    print_stack_frame(&stack_frame);
    reflective_audit("Device Not Available", "Halted");
    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    println!("EXCEPTION: INVALID TSS, error_code={error_code:#x}");
    print_stack_frame(&stack_frame);
    reflective_audit("Invalid TSS", "Halted");
    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: SEGMENT NOT PRESENT, error_code={error_code:#x}");
    print_stack_frame(&stack_frame);
    reflective_audit("Segment Not Present", "Halted");
    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: STACK SEGMENT FAULT, error_code={error_code:#x}");
    print_stack_frame(&stack_frame);
    reflective_audit("Stack Segment Fault", "Halted");
    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: GENERAL PROTECTION FAULT, error_code={error_code:#x}");
    print_stack_frame(&stack_frame);
    reflective_audit("General Protection Fault", "Halted");
    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: X87 FLOATING POINT");
    print_stack_frame(&stack_frame);
    reflective_audit("x87 Floating Point", "Halted");
    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    println!("EXCEPTION: ALIGNMENT CHECK, error_code={error_code:#x}");
    print_stack_frame(&stack_frame);
    reflective_audit("Alignment Check", "Halted");
    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: SIMD FLOATING POINT");
    print_stack_frame(&stack_frame);
    reflective_audit("SIMD Floating Point", "Halted");
    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    // Printing the whole InterruptStackFrame via {:#?} causes a cascading
    // re-fault (repeated invocations of this handler with a shrinking stack
    // pointer and an instruction_pointer that doesn't correspond to any code
    // in this binary) -- root cause not yet found, deferred along with the
    // fingerprint::gather() investigation (see main.rs). Print individual
    // fields instead; this avoids the hang and still gives the one value
    // that actually matters for diagnosis (where the fault happened).
    println!("instruction_pointer: {:?}", stack_frame.instruction_pointer);
    println!("code_segment: {:?}", stack_frame.code_segment);
    println!("stack_pointer: {:?}", stack_frame.stack_pointer);

    reflective_audit("Page Fault", "Instruction/Data access error");

    #[allow(clippy::empty_loop)]
    loop {}
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Acknowledge interrupt via APIC or PIC
    unsafe {
        crate::arch::x86_64::apic::end_of_interrupt();
    }

    // Trigger the scheduler
    crate::sched::SCHEDULER.lock().schedule();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::drivers::input::keyboard::handle_interrupt();
    unsafe {
        crate::arch::x86_64::apic::end_of_interrupt();
    }
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::drivers::input::mouse::handle_interrupt();
    unsafe {
        crate::arch::x86_64::apic::end_of_interrupt();
    }
}
