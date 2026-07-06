use crate::qemu_exit::{exit_qemu, QemuExitCode};

pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        crate::println!("{}...", core::any::type_name::<T>());
        self();
        crate::println!("[ok]");
    }
}

#[allow(dead_code)]
pub fn runner(tests: &[&dyn Testable]) {
    crate::println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}
