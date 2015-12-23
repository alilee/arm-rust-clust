TARGET=arm-none-eabi
CURL=curl
AS=$(TARGET)-as
GCC=$(TARGET)-gcc
LD=$(TARGET)-ld
OBJCOPY=$(TARGET)-objcopy
OBJDUMP=$(TARGET)-objdump
MKIMAGE=mkimage
QEMU=qemu-system-arm
GDB=$(TARGET)-gdb
arm-libgcc=/usr/local/opt/gcc-arm-none-eabi/lib/gcc/arm-none-eabi/4.9.3/libgcc.a

BOARD=versatilepb
CPU=cortex-a9

ASFLAGS = -mcpu=$(CPU) -g
CFLAGS = -mcpu=$(CPU) -g
LDFLAGS = --gc-sections

build/%.o: %.c
	@mkdir -p $(shell dirname $@)
	$(GCC) $(CFLAGS) -c $< -o $@

build/%.o: %.s
	@mkdir -p $(shell dirname $@)
	$(AS) $(ASFLAGS) $< -o $@

build/%.o: %.S
	@mkdir -p $(shell dirname $@)
	$(GCC) -E $(CFLAGS) $< | $(AS) $(ASFLAGS) -o $@

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
sysroot := $(shell rustc --print sysroot)
rustlib_dir := $(sysroot)/lib/rustlib/$(TARGET)/lib
rust_crate := externals/rust

# libcore
libcore_src := externals/core/src
libcore_dest := $(rustlib_dir)/libcore.rlib

sdimage_dir := deploy/sdimage

c_source_files := $(shell find src -name '*.c')
assembly_source_files := $(shell find src -name '*.s')
assemblypp_source_files := $(shell find src -name '*.S')
c_object_files := $(patsubst %.c, build/%.o, $(c_source_files)) 
assembly_object_files := $(patsubst %.s, build/%.o, $(assembly_source_files)) 
assemblypp_object_files := $(patsubst %.S, build/%.o, $(assemblypp_source_files))

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

build/kernel.elf: $(rust_os) $(assembly_object_files) $(assemblypp_object_files) $(c_object_files) $(linker_script)
	@mkdir -p $(shell dirname $@)
	$(LD) $(LDFLAGS) -T $(linker_script) -o $@ $(assembly_object_files) $(rust_os) $(assemblypp_object_files) $(c_object_files)

$(rust_os): $(shell find src/ -type f -name '*.rs') Cargo.toml
	cargo rustc --target $(TARGET) --verbose -- -C opt-level=1 -C target-cpu=$(CPU) --emit asm,link,llvm-ir

qemu: $(kernel)
	$(QEMU) -M $(BOARD) -cpu $(CPU) -m 256M -nographic -s -S -kernel $(kernel)

update-rust:
	multirust update nightly
	rustc --version | sed 's/^.*(\(.*\) .*$$/\1/' > /tmp/rustc-commit.txt
	cd $(rust_crate) && git fetch && git checkout `cat /tmp/rustc-commit.txt`
	@rm /tmp/rustc-commit.txt
	@mkdir -p $(shell dirname $(libcore_dest))
	@rm -f $(libcore_src)
	@mkdir -p $(shell dirname $(libcore_src))
	@ln -s ../rust/src/libcore $(libcore_src)
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
