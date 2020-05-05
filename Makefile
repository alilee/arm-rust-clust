HOST = x86_64-apple-darwin
TARGET = aarch64-unknown-none-softfloat
QEMU = qemu-system-aarch64
GDB = gdb

BINTOOLS = rust
OBJCOPY = $(BINTOOLS)-objcopy
OBJDUMP = $(BINTOOLS)-objdump

BOARD = virt
CPU = cortex-a53

.PHONY: all build test clean qemu gdb run

all: test build

### test, build and run ###

build: $(kernel).bin

test:
	cd kernel && cargo test --all-targets --color=always --target=$(HOST)

clean:
	cargo clean
	rm *.rawdtb
	rm *.dtb

SOURCES := $(shell find . -name '*.rs') linker.ld
kernel := target/$(TARGET)/debug/rust-clust

$(kernel): $(SOURCES)
	cargo build

%.bin: % linker.ld
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -dS $< > $*.code
	$(OBJDUMP) -d $< > $*.s

### debugging ###

qemu.rawdtb:
	$(QEMU) -M $(BOARD),dumpdtb=$@ -cpu $(CPU) -m 256M

%.dtb: %.rawdtb
	dtc -I dtb -O dtb $< > $@

%.dts: %.dtb
	dtc -I dtb -O dts $< -o $@

qemu: $(kernel).bin qemu.dtb
	$(QEMU) -M $(BOARD) -cpu $(CPU) -m 256M -nographic -semihosting -s -S -dtb qemu.dtb -kernel $<

gdb: $(kernel)
	$(GDB) -iex 'file $(kernel)' -iex 'target remote localhost:1234'

run: $(kernel).bin qemu.dtb
	$(QEMU) -M $(BOARD) -cpu $(CPU) -m 256M -nographic -semihosting -dtb qemu.dtb -kernel $<

