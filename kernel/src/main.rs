#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner::runner)]
#![reexport_test_harness_main = "test_main"]

mod panic;
mod test_runner;

use common::addr::PhysAddr;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initial verification of integration
    let _initial_addr = PhysAddr(0x1000);

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
