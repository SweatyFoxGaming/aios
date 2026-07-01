/// Represents a physical memory address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

/// Represents a virtual memory address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl PhysAddr {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl VirtAddr {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}
