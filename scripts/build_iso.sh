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

# `insmod vga` / `terminal_output console` / `gfxpayload=text` are required
# on at least some real hardware: grub-mkrescue's default gfxterm/video-mode
# probing can fail outright ("error: no suitable video mode found") on
# hardware whose BIOS/VESA doesn't support the modes GRUB tries, which
# freezes the boot before the menu even appears -- QEMU's emulated VBE
# doesn't hit this, so it went unnoticed until tested on real hardware.
cat > "$IMAGE_DIR/boot/grub/grub.cfg" <<'EOF'
insmod vga
terminal_output console
set gfxpayload=text
set timeout=1
set default=0
menuentry "Phoenix OS" {
    multiboot2 /boot/kernel.elf
    boot
}
EOF

if command -v grub-mkrescue >/dev/null 2>&1; then
    # -d /usr/lib/grub/i386-pc forces a BIOS-only image (no EFI boot
    # catalog at all). Without this, grub-mkrescue builds a hybrid
    # BIOS+EFI image whenever x86_64-efi grub modules are installed, and
    # on hardware that boots that USB via UEFI, GRUB's EFI path does its
    # own separate GOP video-mode negotiation that the BIOS-oriented
    # `insmod vga`/`terminal_output console` fix above doesn't touch --
    # and pure UEFI has no legacy VGA text mode to fall back to at all.
    # This kernel's boot32.asm bootstrap is BIOS-only (32-bit protected
    # mode handoff), so there's no UEFI path worth keeping anyway.
    grub-mkrescue -d /usr/lib/grub/i386-pc -o "$ISO_NAME" "$IMAGE_DIR"
    echo "ISO created: $ISO_NAME"
else
    echo "WARNING: grub-mkrescue not found. ISO cannot be created."
    echo "ISO directory structure prepared at $IMAGE_DIR."
fi
