use crate::println;
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[failed]");
    println!("Error: {}", info);
    crate::qemu_exit::exit_qemu(crate::qemu_exit::QemuExitCode::Failed);
}
