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

check:
	cargo check

build: $(kernel).bin

unit_test:
	cargo test --lib --target=$(HOST)

define KERNEL_TEST_RUNNER
#!/usr/local/bin/fish

$(OBJCOPY) -O binary $$argv[1] $$argv[1].bin
$(QEMU) -M $(BOARD) -cpu $(CPU) -m 256M -nographic -semihosting -dtb qemu.dtb -kernel $$argv[1].bin > $$argv[1].out
set result $$status
if test $$result -ne 0
	cat $$argv[1].out
	$(OBJDUMP) -dS $$argv[1] > $$argv[1].code
	$(OBJDUMP) -d $$argv[1] > $$argv[1].s
end
exit $$result
endef

export KERNEL_TEST_RUNNER
target/kernel_test_runner.sh: Makefile
	@mkdir -p target
	@echo "$$KERNEL_TEST_RUNNER" > target/kernel_test_runner.sh
	@chmod +x target/kernel_test_runner.sh

test: unit_test qemu.dtb target/kernel_test_runner.sh
	cargo test --tests

clean:
	cargo clean
	rm *.rawdtb
	rm *.dtb
	rm *.dts

SOURCES := $(shell find . -name '*.rs') linker.ld
kernel := target/$(TARGET)/debug/kernel

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

