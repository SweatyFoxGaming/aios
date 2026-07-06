/// Represents a physical memory address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

/// Represents a virtual memory address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl PhysAddr {
    /// Returns the address as a `u64`.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Checks if the address is aligned to the given boundary.
    #[must_use]
    pub const fn is_aligned(self, align: u64) -> bool {
        self.0 % align == 0
    }

    /// Aligns the address down to the given boundary.
    #[must_use]
    pub const fn align_down(self, align: u64) -> Self {
        Self(self.0 & !(align - 1))
    }

    /// Aligns the address up to the given boundary.
    #[must_use]
    pub const fn align_up(self, align: u64) -> Self {
        Self((self.0 + align - 1) & !(align - 1))
    }
}

impl VirtAddr {
    /// Returns the address as a `u64`.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Checks if the address is aligned to the given boundary.
    #[must_use]
    pub const fn is_aligned(self, align: u64) -> bool {
        self.0 % align == 0
    }

    /// Aligns the address down to the given boundary.
    #[must_use]
    pub const fn align_down(self, align: u64) -> Self {
        Self(self.0 & !(align - 1))
    }

    /// Aligns the address up to the given boundary.
    #[must_use]
    pub const fn align_up(self, align: u64) -> Self {
        Self((self.0 + align - 1) & !(align - 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phys_addr_alignment_helpers() {
        assert!(PhysAddr(0x1000).is_aligned(0x1000));
        assert!(!PhysAddr(0x1001).is_aligned(0x1000));
        assert_eq!(PhysAddr(0x1001).align_down(0x1000), PhysAddr(0x1000));
        assert_eq!(PhysAddr(0x1001).align_up(0x1000), PhysAddr(0x2000));
        assert_eq!(PhysAddr(0x1000).align_up(0x1000), PhysAddr(0x1000));
    }

    #[test]
    fn virt_addr_alignment_helpers() {
        assert!(VirtAddr(0x2000).is_aligned(0x1000));
        assert!(!VirtAddr(0x2001).is_aligned(0x1000));
        assert_eq!(VirtAddr(0x2001).align_down(0x1000), VirtAddr(0x2000));
        assert_eq!(VirtAddr(0x2001).align_up(0x1000), VirtAddr(0x3000));
    }
}
