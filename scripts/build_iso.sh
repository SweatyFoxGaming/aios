#!/bin/bash
set -e

# Builds a bootable GRUB2/Multiboot2 ISO for Phoenix OS.
#
# Superseded the old Limine-based build (see git history) when the kernel
# switched to a hand-written 32-bit bootstrap (kernel/boot32.asm) +
# GRUB2/Multiboot2. Run from the repo root after building the kernel:
#   cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none
#   ./scripts/build_iso.sh
#
# CONFIRMED WORKING on real hardware (Dell laptop) via Legacy/CSM boot
# mode specifically -- boot the flashed USB with Legacy Boot/CSM enabled
# in firmware setup, not UEFI. UEFI boot on that same machine hangs hard
# at GRUB's "WARNING: no console will be available to OS" message even
# with every video/gfxterm module stripped from the image (see below);
# QEMU/OVMF never reproduced that hang (the same warning is harmless
# there), so it's a real-firmware-specific issue, not anything fixable
# from this script or the kernel side. Legacy/CSM sidesteps it entirely
# and boots cleanly end-to-end, VGA text console rendering correctly.

ISO_NAME="${ISO_NAME:-phoenix-os-grub.iso}"
IMAGE_DIR="${IMAGE_DIR:-isodir}"
KERNEL_BIN="${KERNEL_BIN:-target/x86_64-unknown-none/debug/kernel}"

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
    # On the actual test machine, the video-mode warning is a hard freeze,
    # not just a warning like under QEMU/OVMF -- meaning the real
    # firmware's EFI GOP query itself hangs rather than failing cleanly.
    # `terminal_output console` in grub.cfg above can't prevent this: it
    # only takes effect once grub.cfg is sourced, but GRUB's "normal"
    # module tries to establish a terminal (defaulting to gfxterm/video if
    # those modules are present at all) as it enters interactive/menu mode,
    # which happens as grub.cfg's menuentry is processed -- before
    # `terminal_output console` has a chance to matter if the gfxterm
    # probe itself is what's hanging.
    #
    # Explicit --install-modules excludes every video/gfxterm/GOP module
    # from the image entirely, so GRUB has nothing to probe with in the
    # first place and must use plain console output unconditionally. This
    # is a fixed minimal set (menu/multiboot2/partition/filesystem/search
    # essentials only) rather than the default "all", which normally
    # includes every video module GRUB ships.
    MODULES="normal multiboot2 multiboot part_gpt part_msdos part_apple iso9660 fat search search_fs_uuid search_fs_file search_label configfile echo ls reboot halt boot terminal"
    grub-mkrescue --install-modules="$MODULES" -o "$ISO_NAME" "$IMAGE_DIR"
    echo "ISO created: $ISO_NAME"
else
    echo "WARNING: grub-mkrescue not found. ISO cannot be created."
    echo "ISO directory structure prepared at $IMAGE_DIR."
fi
