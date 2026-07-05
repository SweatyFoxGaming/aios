//! Interrupt Descriptor Table (IDT) implementation.
use crate::println;
use alloc::string::ToString;
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

/// Initialize the IDT.
pub fn init_idt() {
    IDT.load();
}

/// Helper to log exceptions to the audit log (The Reflective Mechanism).
fn reflective_audit(name: &'static str, details: &str) {
    let system_token = Token::empty(0);
    crate::audit::log(
        &system_token,
        "EXCEPTION: ".to_string() + name + " - " + details,
        "Reflected",
    );
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{stack_frame:#?}");
    reflective_audit("Breakpoint", "Handled");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    reflective_audit("Double Fault", "CRITICAL");
    panic!("EXCEPTION: DOUBLE FAULT\n{stack_frame:#?}");
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DIVIDE ERROR\n{stack_frame:#?}");
    reflective_audit("Divide By Zero", "Handled");
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: INVALID OPCODE\n{stack_frame:#?}");
    reflective_audit("Invalid Opcode", "Handled");
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
