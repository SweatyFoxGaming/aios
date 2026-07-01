# Phoenix OS

Phoenix OS is a lightweight operating system designed from the ground up for low-end hardware, focusing on speed, stability, and modularity.

## Objectives

- **Lightweight**: Optimized for hardware with as little as 1 GB RAM and dual-core x86_64 CPUs.
- **Modular**: Components are clearly separated and can be developed/tested independently.
- **Modern**: Built with Rust to ensure memory safety and reliability.
- **Fast**: Minimal background services and efficient resource management.

## System Requirements

- **CPU**: Dual-core x86_64
- **RAM**: 2 GB recommended, 1 GB minimum
- **Storage**: 16 GB
- **Graphics**: Integrated graphics (no dedicated GPU required)
- **Boot**: UEFI (Primary), Legacy BIOS (Secondary)

## Project Structure

- `boot/`: Bootloader configuration and assets.
- `kernel/`: Core operating system kernel (Rust, no_std).
- `common/`: Shared libraries used by the kernel and userspace.
- `arch/`: Architecture-specific code.
- `drivers/`: Hardware drivers.
- `filesystem/`: VFS and filesystem implementations.
- `userspace/`: Base libraries and applications.
- `docs/`: Comprehensive documentation.

## Getting Started

### Prerequisites

- Rust (stable/nightly)
- `llvm-tools-preview`
- `qemu-system-x86_64` (for testing)

### Building

```bash
cargo build --workspace
```

## License

This project is licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.
