#!/usr/bin/env python3
import os
import urllib.request
import tarfile
import shutil

LIMINE_VERSION = "v8.x-binary" # Use the latest binary release branch
# Alternatively, use a specific version for stability
LIMINE_TAG = "v8.0.2"
LIMINE_URL = f"https://github.com/limine-bootloader/limine/archive/refs/tags/{LIMINE_TAG}.tar.gz"

BOOT_DIR = "boot/limine"

def download_limine():
    if os.path.exists(BOOT_DIR):
        print(f"Limine already exists in {BOOT_DIR}")
        return

    os.makedirs("target/limine", exist_ok=True)
    tar_path = "target/limine/limine.tar.gz"

    print(f"Downloading Limine {LIMINE_TAG}...")
    urllib.request.urlretrieve(LIMINE_URL, tar_path)

    print("Extracting Limine...")
    with tarfile.open(tar_path, "r:gz") as tar:
        tar.extractall(path="target/limine")

    # The folder inside the tar is limine-<version>
    extracted_dir = f"target/limine/limine-{LIMINE_TAG.lstrip('v')}"

    os.makedirs(BOOT_DIR, exist_ok=True)

    # We need specific files for the bootloader
    required_files = [
        "limine-bios.sys",
        "limine-bios-cd.bin",
        "limine-uefi-cd.bin",
        "BOOTX64.EFI",
        "BOOTIA32.EFI"
    ]

    # Check for binary-only files if they exist, or use the source-like ones if needed
    # Actually, the 'limine' repo source doesn't contain the .sys/.bin files pre-built.
    # The 'limine' team provides a 'limine' branch called 'binary' which has them.

    print("Limine source extracted. Note: Binary files usually come from the 'binary' branch.")
    # I will attempt to download from the binary branch instead if possible.

def download_limine_binaries():
    # Use the 'v8.x-binary' branch which contains prebuilt binaries
    BRANCH = "v8.x-binary"
    FILES = [
        "limine-bios.sys",
        "limine-bios-cd.bin",
        "limine-uefi-cd.bin",
        "BOOTX64.EFI",
        "BOOTIA32.EFI"
    ]

    os.makedirs(BOOT_DIR, exist_ok=True)
    base_url = f"https://github.com/limine-bootloader/limine/raw/{BRANCH}/"

    for f in FILES:
        target_path = os.path.join(BOOT_DIR, f)
        if not os.path.exists(target_path):
            print(f"Downloading {f}...")
            urllib.request.urlretrieve(base_url + f, target_path)

    print("Limine binaries ready.")

if __name__ == "__main__":
    download_limine_binaries()
