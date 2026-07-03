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

## Evolutionary Systems

Phoenix OS features an advanced, evolutionary kernel designed for the next generation of AI-native computing:

- **Silicon Morphing**: Dynamic kernel optimization that hot-swaps critical code paths (like `memcpy` and `memset`) based on hardware features (AVX/SSE) discovered at runtime.
- **Lethe Memory Pruning**: significance-based memory management that preserves important AI context while aggressively pruning low-relevance data in the Mnemosyne knowledge graph during resource pressure.
- **Ghost Shell Self-Healing**: Automated recovery architecture where the Vesta homeostasis manager monitors system health and triggers healing sequences to restore failed services autonomously.
- **Ambient UI (Invisible Partner)**: A revolutionary interface philosophy that rejects traditional desktop metaphors. The system remains in a calm, architectural idle state (The Phoenix Emblem) and materializes tools only in response to expressed user intent.
- **Zero-Copy Synapse IPC**: High-performance, shared-memory messaging that transfers ownership of physical memory frames between agents, optimizing for large AI context windows without redundant data copies.

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
