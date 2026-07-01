#!/bin/bash
set -e

# Configuration
ISO_NAME="phoenix-os.iso"
IMAGE_DIR="target/iso"
KERNEL_BIN="target/x86_64-unknown-none/debug/kernel"
LIMINE_DIR="boot/limine"

echo "Building Phoenix OS ISO..."

# Create the directory structure for the ISO
mkdir -p "$IMAGE_DIR/boot"
mkdir -p "$IMAGE_DIR/EFI/BOOT"

# Copy the kernel
cp "$KERNEL_BIN" "$IMAGE_DIR/boot/kernel"

# Copy Limine configuration and binaries
cp boot/limine.conf "$IMAGE_DIR/boot/"
cp "$LIMINE_DIR/limine-bios.sys" "$IMAGE_DIR/boot/"
cp "$LIMINE_DIR/limine-bios-cd.bin" "$IMAGE_DIR/boot/"
cp "$LIMINE_DIR/limine-uefi-cd.bin" "$IMAGE_DIR/boot/"
cp "$LIMINE_DIR/BOOTX64.EFI" "$IMAGE_DIR/EFI/BOOT/"
cp "$LIMINE_DIR/BOOTIA32.EFI" "$IMAGE_DIR/EFI/BOOT/"

# Check for xorriso
if command -v xorriso >/dev/null 2>&1; then
    echo "Creating ISO with xorriso..."
    xorriso -as mkisofs -b boot/limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table \
        --efi-boot boot/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        "$IMAGE_DIR" -o "$ISO_NAME"
    echo "ISO created: $ISO_NAME"
else
    echo "WARNING: xorriso not found. ISO cannot be created."
    echo "ISO directory structure prepared at $IMAGE_DIR."
fi
