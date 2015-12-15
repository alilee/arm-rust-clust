TARGET=arm-none-eabi
CURL=curl
AS=$(TARGET)-as
LD=$(TARGET)-ld
OBJCOPY=$(TARGET)-objcopy
OBJDUMP=$(TARGET)-objdump
MKIMAGE=mkimage
QEMU=qemu-system-arm
GDB=$(TARGET)-gdb

BOARD=versatilepb
CPU=cortex-a9

ASFLAGS = -mcpu=$(CPU) -g
LDFLAGS = --gc-sections

%.o: %.s
	$(AS) $(ASFLAGS) $< -o $@

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -dS $< > $*.code
	$(OBJDUMP) -d $< > $*.s

kernel := build/kernel.bin
tftpboot_rpi := /private/tftpboot/rpi
image := $(tftpboot_rpi)/uImage

rust_os := target/$(TARGET)/debug/libarm.a
linker_script := linker.ld

# init the submodule and checkout the same build as your nightly (see readme.md)
rust_libcore := crates/rust/src/libcore
libcore_src := crates/core/src
sysroot := $(shell rustc --print sysroot)
rustlib_dir := $(sysroot)/lib/rustlib/$(TARGET)/lib
libcore_dest := $(rustlib_dir)/libcore.rlib

sdimage_dir := deploy/sdimage

assembly_source_files := $(wildcard src/*.s)
assembly_object_files := $(patsubst %.s, %.o, $(assembly_source_files))

.PHONY: all clean qemu update-rust tftpd sdimage gdb test doc

all: $(image)

clean:
	@cargo clean
	@rm -rf build
	@rm $(libcore_dest)
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

$(rust_os): $(shell find src/ -type f -name '*.rs') $(shell find crates/aeabi/ -type f -name '*.rs') Cargo.toml
	cargo rustc --target $(TARGET) --verbose -- -g -C opt-level=1 -C target-cpu=$(CPU) --emit asm,link,llvm-ir

qemu: $(kernel)
	$(QEMU) -M $(BOARD) -cpu $(CPU) -m 256M -nographic -s -S -kernel $(kernel)

update-rust:
	multirust update nightly
	rustc --version | sed 's/^.*(\(.*\) .*$$/\1/' > /tmp/rustc-commit.txt
	cd $(rust_libcore) && git fetch && git checkout `cat /tmp/rustc-commit.txt`
	@rm /tmp/rustc-commit.txt
	rm $(libcore_src)
	ln -s ../rust/src/libcore $(libcore_src)
	@mkdir -p $(shell dirname $(libcore_dest))
	rustc $(libcore_src)/lib.rs \
	  --crate-name core \
	  --crate-type lib \
	  --out-dir $(shell dirname $(libcore_dest)) \
	  --emit=link \
	  -g \
	  --target $(TARGET)

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
	$(GDB) -ex 'file $(patsubst %.bin, %.elf, $(kernel))' -ex 'target remote localhost:1234'
