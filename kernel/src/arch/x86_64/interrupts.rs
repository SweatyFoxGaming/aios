//! Interrupt Descriptor Table (IDT) implementation.
use crate::println;
use alloc::string::ToString;
use common::security::Token;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

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
        idt
    };
}

/// Initialize the IDT.
pub fn init_idt() {
    IDT.load();
}

/// Helper to log exceptions to the audit log (The Reflective Mechanism).
fn reflective_audit(name: &'static str, details: &str) {
    // We use a dummy token for now, or the current task's token in the future
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
    println!("{stack_frame:#?}");

    reflective_audit("Page Fault", "Instruction/Data access error");

    // In a real OS, we would handle demand paging here.
    // For now, we loop to prevent further chaos.
    #[allow(clippy::empty_loop)]
    loop {}
}
