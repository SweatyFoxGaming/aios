# Phoenix OS Makefile

.PHONY: all kernel common clean test build-all size iso setup-limine

all: build-all

setup-limine:
	python3 scripts/setup_limine.py

kernel:
	cargo build -p kernel

common:
	cargo build -p common

build-all:
	cargo build --workspace

test:
	cargo test -p common --target x86_64-unknown-linux-gnu
	./scripts/test_kernel.sh
	./scripts/build_iso.sh
	./scripts/test_boot.sh

clean:
	cargo clean
	rm -rf target/iso phoenix-os.iso

size: kernel
	@echo "Kernel Binary Size Information:"
	@size target/x86_64-unknown-none/debug/kernel
	@stat -c "Total size: %s bytes" target/x86_64-unknown-none/debug/kernel

iso: kernel setup-limine
	./scripts/build_iso.sh
