# Network Debug/Verification Server — Design

**Date:** 2026-07-05
**Status:** Approved for implementation planning
**Target:** QEMU first (Intel e1000 NIC, confirmed as QEMU's default for this project's boot invocation — `PCI Device: Bus 0 Slot 3: Vendor 8086 Device 100e` in every boot log this session). Real-hardware NIC support is explicitly out of scope until the physical test machine's actual chipset is known.

## 1. Background & scope

`kernel/src/net/` is currently 100% fake — `init()` prints a message, `process_packet`/`send_udp` print messages, no PCI device is ever touched, no real Ethernet/IP/UDP stack exists. This is the same situation `aura.rs` was in before the Ambient UI work.

This spec replaces it with a real, minimal network stack whose sole purpose is letting me (the agent, working headlessly) verify the Ambient UI's Idle/Active state machine and three panels by sending UDP packets to a running QEMU instance and reading back a text state dump — since QEMU's `-display none` + monitor `sendkey` was confirmed this session to not reach the guest keyboard handler at all.

**Non-goals:**
- TCP (no retransmission/congestion control needed for a debug tool; UDP is sufficient and far simpler).
- DHCP (a static IP inside QEMU's user-mode-networking subnet avoids needing a DHCP client).
- Real hardware NIC support (unknown chipset; deferred).
- Any framebuffer/pixel transfer — the state dump is text only, matching what's actually needed (verifying *logic*, not re-implementing a remote display protocol).
- Production-grade robustness (malformed-packet handling can be minimal/best-effort; this is a debug tool, not a hardened network service).

## 2. e1000 driver

New module `kernel/src/net/e1000.rs`. Detected via the existing `arch::x86_64::pci::scan_bus()` (already enumerates PCI devices and prints vendor/device IDs — just needs to recognize `8086:100e` and hand off to this driver instead of only logging it).

### 2.1 Initialization
- Read BAR0 from PCI config space to get the MMIO base address (register-level, not port I/O — e1000 is a memory-mapped device). BAR0's low 4 bits are flags, must be masked off.
- Map that MMIO region: since `boot32.asm` identity-maps the first 4GiB (extended in the Ambient UI work to reach the framebuffer at `0xfd000000`), and e1000 MMIO BARs on QEMU's default `pc` machine are typically also below 4GiB, no *new* paging work should be needed — verified empirically once the driver reads a known register (e.g. `STATUS` at offset `0x0008`) and confirms a sane, non-`0xffffffff` value (`0xffffffff` back from any MMIO read means the address isn't actually mapped/decoded, the standard PCI "no device there" tell).
- Reset the device (`CTRL` register, `RST` bit), wait for it to clear.
- Read the MAC address from the `RAL0`/`RAH0` registers (already programmed by QEMU/firmware; no EEPROM bit-banging needed).
- Set up one TX descriptor ring and one RX descriptor ring (see 2.2), program `TDBAL`/`TDBAH`/`TDLEN`/`TDH`/`TDT` and `RDBAL`/`RDBAH`/`RDLEN`/`RDH`/`RDT` accordingly, enable TX (`TCTL.EN`) and RX (`RCTL.EN`).
- Poll-based, not interrupt-driven: given this kernel's existing `ambient_ui::render()` already runs as a busy loop in `main.rs`'s final loop, add a `net::e1000::poll()` call there that checks for received packets and processes them. This avoids adding a new PCI interrupt line/IDT entry for a debug-only feature — acceptable latency (bounded by loop iteration time, which is already fast) for a tool whose job is answering debug queries, not real-time networking.

### 2.2 DMA buffers and physical addressing
Descriptor rings and packet buffers must be given to the NIC as **physical** addresses, but Rust code accesses them via virtual pointers — these differ because the kernel is linked at `KERNEL_VMA_OFFSET = 0xffffffff80000000` (`kernel/linker.ld`), the same constant as `main.rs`'s `PHYS_MEM_OFFSET`.

This kernel already relies on exactly this relationship elsewhere: `mem/paging.rs`'s `active_level_4_table` computes `virt = physical_memory_offset + phys.as_u64()` to reach the CR3 page table, and `boot32.asm`'s page tables (`p4_table` etc.) are addressed directly as physical addresses because they're placed in the identity-mapped low region. For a **Rust-level static** (in the kernel's own `.bss`/`.data`, linked at the higher half), the reverse holds: `physical_addr = (&STATIC as *const _ as u64) - KERNEL_VMA_OFFSET`. `kernel/linker.ld` confirms this directly — every section is emitted `AT(ADDR(.section) - KERNEL_VMA_OFFSET)`, i.e. its physical load address is exactly its virtual address minus that offset.

Declare descriptor rings and packet buffers as `MaybeUninit` statics (never zero-initialized at declaration — `MaybeUninit::uninit()` compiles to no store instructions at all, unlike `[0; N]` for N above ~192 bytes, which lowers to a `memset` call that reliably crashes on this target, per `safe_alloc.rs`'s extensively-documented root cause). Every field actually used is written explicitly before the NIC or any Rust code reads it — descriptor fields via a manual per-field loop (matching the established pattern throughout this kernel), packet buffer contents via `safe_alloc::copy_from_slice` when building an outgoing frame.

```rust
const NUM_RX_DESC: usize = 8;
const NUM_TX_DESC: usize = 8;
const BUF_SIZE: usize = 2048; // one Ethernet frame, generous

static mut RX_DESC_RING: MaybeUninit<[RxDescriptor; NUM_RX_DESC]> = MaybeUninit::uninit();
static mut TX_DESC_RING: MaybeUninit<[TxDescriptor; NUM_TX_DESC]> = MaybeUninit::uninit();
static mut RX_BUFFERS: MaybeUninit<[[u8; BUF_SIZE]; NUM_RX_DESC]> = MaybeUninit::uninit();
static mut TX_BUFFERS: MaybeUninit<[[u8; BUF_SIZE]; NUM_TX_DESC]> = MaybeUninit::uninit();

fn phys_addr_of<T>(r: &T) -> u64 {
    (r as *const T as u64) - crate::PHYS_MEM_OFFSET
}
```

(`RxDescriptor`/`TxDescriptor` are `#[repr(C)]` structs matching the e1000 hardware descriptor layout — 16 bytes each, documented in the implementation plan with exact field offsets from the Intel datasheet / OSDev wiki.)

### 2.3 Send/receive
- `send_frame(dst_mac: [u8; 6], ethertype: u16, payload: &[u8])`: writes payload into the next TX buffer slot, builds the Ethernet header (dst MAC, this NIC's src MAC from `RAL0`/`RAH0`, ethertype), sets up the TX descriptor (address, length, `EOP`/`RS` flags), advances `TDT`.
- `poll()`: checks the current RX descriptor's status bit (`DD`, descriptor done); if set, hands the buffer contents to the Ethernet-layer dispatcher (2.3 below), then resets the descriptor's status and advances `RDT` so the NIC can reuse that slot.

## 3. Minimal protocol stack

New modules under `kernel/src/net/`: `ethernet.rs`, `arp.rs`, `ipv4.rs` (extends the existing `udp.rs`, replacing its fake `process_packet`/`send_udp`).

- **Ethernet** (`ethernet.rs`): parses the 14-byte header (dst MAC, src MAC, ethertype), dispatches to ARP (`0x0806`) or IPv4 (`0x0800`); on send, prepends this header via `e1000::send_frame`.
- **ARP** (`arp.rs`): responds to ARP requests for this kernel's static IP; sends an ARP request (and blocks briefly via `e1000::poll()` in a bounded retry loop) to resolve the host's MAC address before the first UDP send, caching the one (host) MAC address this debug tool ever needs — no full ARP cache/table needed, this always talks to exactly one peer (the QEMU SLIRP gateway/host).
- **IPv4** (`ipv4.rs`): builds/parses the 20-byte header (no options), computes the header checksum (standard one's-complement sum — a single, non-looped computation over a fixed 20-byte header, not a repeated `{:.N}` float-format pattern, so it doesn't hit that other documented bug), dispatches UDP (protocol 17) payloads to `udp.rs`.
- **UDP** (`udp.rs`, replacing the fake version): parses/builds the 8-byte header (src/dst port, length, checksum — checksum can be `0`/disabled per RFC 768, simplest valid option for a debug tool), dispatches payloads on the debug port to `net::debug_server` (4 below).

### 3.1 Addressing
Static configuration, no DHCP:
- Guest IP: `10.0.2.15` (QEMU user-mode networking's documented default guest address)
- Gateway/host IP: `10.0.2.2` (QEMU SLIRP's documented default host address)
- Guest MAC: whatever QEMU's e1000 model already programmed into `RAL0`/`RAH0` (read, not invented)

## 4. Debug protocol

Fixed debug UDP port (e.g. `7878`, chosen arbitrarily, documented in the implementation plan). New module `kernel/src/net/debug_server.rs`.

Simple, single-byte-tagged text commands (ASCII, not a binary format — trivially inspectable/typeable while developing, and this is a debug tool, not something needing wire-format efficiency):

| Command (client → kernel) | Meaning |
|---|---|
| `K<char>` | Inject one character as if typed (routed through the same `ambient_ui::on_key(Some(char), false)` path a real keystroke takes) |
| `E` | Inject Escape (`ambient_ui::on_key(None, true)`) |
| `M<x>,<y>,<button>` | Inject a mouse position + button state (decimal ASCII, comma-separated; routed through the same state update `mouse.rs`'s interrupt handler performs, then `ambient_ui::on_mouse_activity()`) |
| `S` | Request a state dump |

Reply to `S` (kernel → client), one line per field, built via `safe_format!`/`safe_alloc` (never `alloc::format!`/`.to_string()`, per this kernel's established constraints):

```
STATE:<Idle|Active>
FOCUS:<Command|Chat>
CMD_INPUT:<current command panel input buffer>
CMD_SCROLLBACK:<line>|<line>|...
CHAT_SCROLLBACK:<line>|<line>|...
EGO:<PresenceState debug repr>
PRESSURE:<memory pressure value>
```

This requires small, additive accessor functions on `ambient_ui.rs`'s existing private statics (`COMMAND_INPUT`, `COMMAND_SCROLLBACK`, `CHAT_SCROLLBACK`, `current_state()`/`current_focus()` already `pub`) — no architectural change to the Ambient UI itself, purely additive read access for the debug server.

## 5. QEMU invocation

```bash
qemu-system-x86_64 -cdrom phoenix-os-grub.iso -display none -serial file:/tmp/boot.log -no-reboot \
  -netdev user,id=net0,hostfwd=udp::7878-:7878 \
  -device e1000,netdev=net0
```

`hostfwd=udp::7878-:7878` forwards UDP port 7878 on the sandbox's `localhost` to port 7878 on the guest's `10.0.2.15` — I send debug commands to `127.0.0.1:7878` from a small Python test script, no different in spirit from the QEMU monitor screendump scripts already used this session.

## 6. Testing

- Build clean, full boot regression (existing standard for this project: serial log reaches stable idle with zero exceptions) — the e1000 driver initializing must not become a new boot blocker, mirroring the "must not regress boot" bar every prior task in this session has held to.
- A small Python UDP client script (analogous to the existing screendump-capture scripts) sending `S`, `K` sequences, `E`, and `M` commands, verifying the state dump reflects each injected event correctly — this becomes the **actual working replacement** for the interactive checklist the Ambient UI plan had to leave to a human, letting me verify Task 7-9's panel logic myself for the first time.
- Explicit non-goal: this is not tested against real hardware in this pass (see Non-goals).
