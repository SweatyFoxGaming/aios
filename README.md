# Phoenix OS

Phoenix OS is a lightweight, AI-native operating system designed for low-end hardware, centered around the **JARVIS** orchestration layer.

## Vision

Phoenix OS is built from the ground up to support JARVIS, an AI assistant that serves as the primary user interface and system orchestrator. While the OS remains fully functional for maintenance and recovery without AI, it is optimized to provide a robust, API-first foundation for autonomous and proactive intelligence.

## Core Principles

- **AI-Native Interface**: Natural language and voice-first interaction via JARVIS.
- **API-First Architecture**: Every system service exposes a stable, structured API.
- **Capability-Based Security**: Granular, auditable permissions for users and AI sub-systems.
- **Lightweight**: Optimized for hardware with as little as 1 GB RAM and dual-core CPUs.
- **Privacy-First**: Local learning and offline capabilities by default.

## System Requirements

- **CPU**: Dual-core x86_64
- **RAM**: 2 GB recommended, 1 GB minimum
- **Storage**: 16 GB
- **Graphics**: Integrated graphics
- **Boot**: UEFI (Primary), Limine Bootloader

## Project Structure

- `boot/`: Bootloader configuration (Limine).
- `kernel/`: API-first core operating system kernel (Rust, no_std).
- `common/`: Shared structured data schemas and utilities.
- `arch/`: Architecture-specific code.
- `drivers/`: Hardware drivers.
- `filesystem/`: Structured VFS and filesystem implementations.
- `userspace/`: Base libraries, JARVIS services, and applications.
- `docs/`: Comprehensive architecture and JARVIS platform documentation.

## License

This project is licensed under the Apache License 2.0.
