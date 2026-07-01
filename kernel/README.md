# Phoenix OS - Kernel

The Phoenix OS kernel is a lightweight, `no_std` Rust kernel designed for x86_64 architecture.

## Architecture

The kernel follows a modular design, with clear separation between architecture-independent logic and architecture-specific implementations.

### Key Components

- **Bootstrapping**: Entry point in `src/main.rs`, handles initial transition from bootloader.
- **Panic Handling**: Custom panic handler to manage system failures gracefully.
- **Testing**: Integrated kernel-space testing framework using `custom_test_frameworks`.

## Current Status

- Initial `no_std` environment established.
- Bare-metal build target configured.
- Testing framework skeleton implemented.

## Building

To build the kernel for the target architecture:

```bash
cargo build -p kernel
```
