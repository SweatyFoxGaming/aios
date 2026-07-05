#!/bin/bash
set -e

# Builds a bootable GRUB2/Multiboot2 ISO for Phoenix OS.
#
# Superseded the old Limine-based build (see git history) when the kernel
# switched to a hand-written 32-bit bootstrap (kernel/boot32.asm) +
# GRUB2/Multiboot2. Run from the repo root after building the kernel:
#   cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none
#   ./scripts/build_iso.sh

ISO_NAME="phoenix-os-grub.iso"
IMAGE_DIR="isodir"
KERNEL_BIN="target/x86_64-unknown-none/debug/kernel"

echo "Building Phoenix OS ISO..."

mkdir -p "$IMAGE_DIR/boot/grub"
cp "$KERNEL_BIN" "$IMAGE_DIR/boot/kernel.elf"

# Must keep the hybrid BIOS+EFI image (default grub-mkrescue behavior):
# an earlier attempt forced BIOS-only (-d i386-pc) on the theory that
# this kernel's boot32.asm bootstrap is BIOS-only anyway, but the test
# machine didn't recognize that USB as bootable AT ALL -- its firmware
# requires UEFI boot for removable media, with no usable legacy/CSM
# fallback. GRUB's multiboot2 loader normalizes CPU state back down to
# 32-bit protected mode before jumping to the kernel regardless of
# whether it got there via BIOS or UEFI, so boot32.asm doesn't need to
# know or care which path was used.
#
# `gfxpayload=text` (an earlier attempt) is BIOS/VGA-specific -- pure
# UEFI has no legacy VGA text mode at all, so requesting it likely
# caused its own "no suitable video mode" failure under UEFI. `keep`
# leaves whatever GOP mode is already active instead of negotiating a
# new one, which is universally safe under both BIOS and UEFI.
cat > "$IMAGE_DIR/boot/grub/grub.cfg" <<'EOF'
terminal_output console
set gfxpayload=keep
set timeout=1
set default=0
menuentry "Phoenix OS" {
    multiboot2 /boot/kernel.elf
    boot
}
EOF

if command -v grub-mkrescue >/dev/null 2>&1; then
    grub-mkrescue -o "$ISO_NAME" "$IMAGE_DIR"
    echo "ISO created: $ISO_NAME"
else
    echo "WARNING: grub-mkrescue not found. ISO cannot be created."
    echo "ISO directory structure prepared at $IMAGE_DIR."
fi
