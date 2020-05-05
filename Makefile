HOST = x86_64-apple-darwin
TARGET = aarch64-unknown-none-softfloat
QEMU = qemu-system-aarch64

BINTOOLS = rust
OBJCOPY = $(BINTOOLS)-objcopy
OBJDUMP = $(BINTOOLS)-objdump

BOARD = virt,gic_version=2
CPU = cortex-a53

SOURCES := $(shell find . -name '*.rs') linker.ld

kernel := target/$(TARGET)/debug/rust-clust

.PHONY: all build test clean

all: test build

build: $(kernel).bin

test:
	cd kernel && cargo test --all-targets --color=always --target=$(HOST)

clean:
	cargo clean

$(kernel): $(SOURCES)
	cargo build

%.bin: % linker.ld
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -dS $< > $*.code
	$(OBJDUMP) -d $< > $*.s