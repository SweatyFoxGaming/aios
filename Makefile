# Phoenix OS Makefile

.PHONY: all kernel common clean test build-all size

all: build-all

kernel:
	cargo build -p kernel

common:
	cargo build -p common

build-all:
	cargo build --workspace

test:
	cargo test -p common --target x86_64-unknown-linux-gnu
	cargo build -p kernel --tests

clean:
	cargo clean

size: kernel
	@echo "Kernel Binary Size Information:"
	@size target/x86_64-unknown-none/debug/kernel
	@stat -c "Total size: %s bytes" target/x86_64-unknown-none/debug/kernel
