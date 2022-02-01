# HOST = x86_64-unknown-linux-gnu
HOST = x86_64-apple-darwin
TARGET = aarch64-unknown-none-softfloat
TARGET_DIR = target/$(TARGET)/debug/deps

QEMU = qemu-system-aarch64
QEMU_SMP = -smp 2
QEMU_DISK = -global virtio-mmio.force-legacy=false -device virtio-blk-device,drive=drive0,id=virtblk0,num-queues=4,packed=on -drive file=disk.qcow2,if=none,id=drive0

GDB = rust-gdb

BINTOOLS = rust
OBJCOPY = $(BINTOOLS)-objcopy
OBJDUMP = $(BINTOOLS)-objdump

BOARD = virt
CPU = cortex-a53
MEM = 64M

kernel := target/$(TARGET)/debug/kernel
linker.ld := src/archs/aarch64/linker.ld
SOURCES := $(shell find . -name '*.rs') $(linker.ld)

.PHONY: all check build unit_test doctest test clean qemu gdb run real_clean complexity

all: test build


### test, build and run ###

check:
	cargo check

build: $(kernel).bin

$(kernel): $(SOURCES)
	cargo build

doctest:
	cargo test --quiet --doc --target=$(HOST)

unit_test: doctest
	cargo test --quiet --lib --target=$(HOST)

define KERNEL_TEST_RUNNER
#!/usr/bin/env fish
## DO NOT EDIT - generated in Makefile

mkdir -p test_output

$(OBJCOPY) -O binary $$argv[1] $$argv[1].bin
$(OBJDUMP) -d $$argv[1] > test_output/(basename $$argv[1].s)
$(QEMU) -M $(BOARD) -cpu $(CPU) -m $(MEM) -nographic $(QEMU_SMP) $(QEMU_DISK) -semihosting -dtb qemu.dtb -d guest_errors -D /tmp/qemu.log -kernel $$argv[1].bin > test_output/(basename $$argv[1].out)
set result $$status
#if test $$result -ne 0
#    cat $$argv[1].out
#	$(OBJDUMP) -dS $$argv[1] > $$argv[1].code
#	$(OBJDUMP) -d $$argv[1] > $$argv[1].s
#end
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
	rm -fr test_output
	cargo clean

%.bin: % $(linker.ld)
	$(OBJCOPY) -O binary $< $@
	echo $(OBJDUMP) -dS $< > $*.code
	$(OBJDUMP) -d $< > $*.s


### debugging ###

disk.qcow2:
	qemu-img create -f qcow2 $@ 1G

qemu.rawdtb: Makefile disk.qcow2
	$(QEMU) -machine $(BOARD),dumpdtb=$@ -cpu $(CPU) -m $(MEM) -nographic $(QEMU_SMP) $(QEMU_DISK)

%.dtb: %.rawdtb
	dtc -I dtb -O dtb $< > $@

%.dts: %.dtb
	dtc -I dtb -O dts $< -o $@

qemu: $(kernel).bin qemu.dtb
	$(QEMU) -M $(BOARD) -cpu $(CPU) -m $(MEM) -nographic $(QEMU_SMP) $(QEMU_DISK) -semihosting -s -S -dtb qemu.dtb -kernel $<

qemu_test: qemu.dtb
	mkdir -p test_output
	rm -f test_output/$(TEST_SUBJECT)-*
	rm -f $(TARGET_DIR)/$(TEST_SUBJECT)-*
	cargo build --test $(TEST_SUBJECT)
	$(OBJCOPY) -O binary $(TARGET_DIR)/$(TEST_SUBJECT)-???????????????? $(TARGET_DIR)/$(TEST_SUBJECT)-????????????????.bin
	$(OBJDUMP) -d $(TARGET_DIR)/$(TEST_SUBJECT)-???????????????? > test_output/$(TEST_SUBJECT).s
	$(QEMU) -M $(BOARD) -cpu $(CPU) -m $(MEM) -nographic $(QEMU_SMP) $(QEMU_DISK) -semihosting -dtb qemu.dtb -s -S -kernel $(TARGET_DIR)/$(TEST_SUBJECT)-*.bin

gdb:
	$(GDB) -iex 'file $(shell find . -name '$(TEST_SUBJECT)-????????????????')' -iex 'target remote localhost:1234' -iex 'layout split' -iex 'layout next'

run: $(kernel).bin qemu.dtb
	$(QEMU) -M $(BOARD) -cpu $(CPU) -m $(MEM) -nographic $(QEMU_SMP) $(QEMU_DISK) -semihosting -dtb qemu.dtb -kernel $<

real_clean: clean
	rm -f *.rawdtb
	rm -f *.dtb
	rm -f *.dts


### source analysis ###

complexity:
	scc -w --exclude-dir .git,.idea --by-file -s complexity
