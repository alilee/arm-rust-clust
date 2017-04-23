TARGET=aarch64-unknown-linux-gnu
CURL=curl
AS=$(TARGET)-as
GCC=$(TARGET)-gcc
LD=$(TARGET)-ld
OBJCOPY=$(TARGET)-objcopy
OBJDUMP=$(TARGET)-objdump
MKIMAGE=mkimage
QEMU=qemu-system-aarch64
# GDB=$(TARGET)-gdb
GDB=gdb

BOARD=virt
CPU=cortex-a53

ASFLAGS = -mcpu=$(CPU) -g
CFLAGS = -mcpu=$(CPU) -g
LDFLAGS = --gc-sections

build/%.o: %.s
	@mkdir -p $(shell dirname $@)
	$(AS) $(ASFLAGS) $< -o $@

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -dS $< > $*.code
	$(OBJDUMP) -d $< > $*.s

kernel := build/kernel.bin
tftpboot_rpi := /private/tftpboot/rpi
image := $(tftpboot_rpi)/uImage

rust_os := target/$(TARGET)/debug/libarc.a
linker_script := linker.ld

# # init the submodule and checkout the same build as your nightly (see readme.md)
# sysroot := $(shell rustc --print sysroot)
# rustlib_dir := $(sysroot)/lib/rustlib/$(TARGET)/lib
# rust_crate := externals/rust

# # libcore
# libcore_src := externals/core/src
# libcore_dest := $(rustlib_dir)/libcore.rlib

sdimage_dir := deploy/sdimage

assembly_source_files := $(shell find src -name '*.s')
assembly_object_files := $(patsubst %.s, build/%.o, $(assembly_source_files))

.PHONY: all clean qemu update-rust tftpd sdimage gdb test doc

all: $(image)

clean:
	@cargo clean
	@rm -rf build
	@rm -rf $(sdimage_dir)

test:
	@cargo test

doc:
	@cargo doc --open

$(image): $(kernel)
	$(MKIMAGE) -A arm -C gzip -O linux -T kernel -d $< -a 0x10000 -e 0x10000 $@
	@chmod 644 $@

build/kernel.elf: $(rust_os) $(assembly_object_files) $(linker_script)
	@mkdir -p $(shell dirname $@)
	$(LD) $(LDFLAGS) -T $(linker_script) -o $@ $(assembly_object_files) $(rust_os)

$(rust_os): $(shell find src/ -type f -name '*.rs') Cargo.toml
	cargo build --target=$(TARGET)

qemu: $(kernel)
	$(QEMU) -M $(BOARD) -cpu $(CPU) -m 256M -nographic -s -S -kernel $(kernel)

tftpd:
	sudo launchctl load -F deploy/tftpd.plist
	sudo launchctl start com.apple.tftpd
	sudo mkdir -p $(tftpboot_rpi)
	sudo chown `whoami`:staff $(tftpboot_rpi)

deploy/u-boot/u-boot.bin:
	cd u-boot && CROSS_COMPILE=$(TARGET)- make rpi_2_defconfig all

$(sdimage_dir)/bootcode.bin:
	$(CURL) -fso $@ https://github.com/raspberrypi/firmware/blob/master/boot/bootcode.bin?raw=true

$(sdimage_dir)/start.elf:
	$(CURL) -fso $@ https://github.com/raspberrypi/firmware/blob/master/boot/start.elf?raw=true

$(sdimage_dir)/kernel.img: deploy/u-boot/u-boot.bin
	cp $< $@

$(sdimage_dir):
	@mkdir -p $@

$(sdimage_dir)/boot.scr.uimg: deploy/boot.scr
	$(MKIMAGE) -A arm -C none -O linux -T script -n $< -d $< $@

sdimage: $(sdimage_dir) $(sdimage_dir)/kernel.img $(sdimage_dir)/boot.scr.uimg $(sdimage_dir)/bootcode.bin $(sdimage_dir)/start.elf

gdb:
	$(GDB) -iex 'file $(patsubst %.bin, %.elf, $(kernel))' -iex 'target remote localhost:1234'

deploy/u-boot/simpleimage-pine64-latest.img:
	@mkdir -p $(@D)
	wget -q https://www.stdin.xyz/downloads/people/longsleep/pine64-images/simpleimage-pine64-latest.img.xz -O $@.xz
	xz -d $@.xz

deploy/u-boot/boot0.img: deploy/u-boot/simpleimage-pine64-latest.img
	dd if=$< bs=1k skip=8 count=64 of=$(@D)/boot0.img

deploy/u-boot/uboot.img: deploy/u-boot/simpleimage-pine64-latest.img
	dd if=$< bs=1k skip=19096 count=1384 of=$(@D)/uboot.img
