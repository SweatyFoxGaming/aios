//! Foundation library for Phoenix OS.
#![cfg_attr(not(test), no_std)]

/// Memory address types.
pub mod addr;
/// Error handling types.
pub mod error;

/// Adds two numbers.
#[must_use]
pub const fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
