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
libcore_src := rust/src/libcore
sysroot := $(shell rustc --print sysroot)
rustlib_dir := $(sysroot)/lib/rustlib/$(TARGET)/lib
libcore := $(rustlib_dir)/libcore.rlib

assembly_source_files := $(wildcard src/*.s)
assembly_object_files := $(patsubst %.s, %.o, $(assembly_source_files))

.PHONY: all clean qemu update-rust tftpd sdimage gdb

all: $(image)

clean:
	@cargo clean
	@rm -rf build
	@rm $(libcore)
	@rm -rf sdimage

$(image): $(kernel)
	$(MKIMAGE) -A arm -C gzip -O linux -T kernel -d $< -a 0x10000 -e 0x10000 $@
	@chmod 644 $@

$(libcore): $(shell find $(libcore_src)/ -type f -name '*.rs')
	@mkdir -p $(shell dirname $@)
	rm core/src
	ln -s ../$(libcore_src) core/src
	rustc core/src/lib.rs \
	  --crate-name core \
	  --crate-type lib \
	  --out-dir $(shell dirname $@) \
	  --emit=link \
	  -g \
	  --target $(TARGET) 

build/kernel.elf: $(rust_os) $(assembly_object_files) $(linker_script)
	@mkdir -p $(shell dirname $@)
	$(LD) $(LDFLAGS) -T $(linker_script) -o $@ $(assembly_object_files) $(rust_os)

$(rust_os): $(wildcard src/*.rs) Cargo.toml $(libcore)
	cargo rustc --target $(TARGET) --verbose -- -C opt-level=1 -C target-cpu=$(CPU)

qemu: $(kernel)
	$(QEMU) -M $(BOARD) -cpu $(CPU) -m 256M -nographic -s -S -kernel $(kernel)

update-rust:
	multirust update
	rustc --version | sed 's/^.*(\(.*\) .*$$/\1/' > rustc-commit.txt
	cd rust && git fetch && git checkout `cat ../rustc-commit.txt`
	@rm rustc-commit.txt

tftpd:
	sudo launchctl load -F tftpd.plist
	sudo launchctl start com.apple.tftpd
	sudo mkdir -p $(tftpboot_rpi)
	sudo chown `whoami`:staff $(tftpboot_rpi)
	
u-boot/u-boot.bin:
	cd u-boot && CROSS_COMPILE=$(TARGET)- make rpi_2_defconfig all

sdimage/bootcode.bin: 
	@mkdir -p sdimage
	$(CURL) -fso $@ https://github.com/raspberrypi/firmware/blob/master/boot/bootcode.bin?raw=true

sdimage/start.elf:
	@mkdir -p sdimage 
	$(CURL) -fso $@ https://github.com/raspberrypi/firmware/blob/master/boot/start.elf?raw=true

sdimage/kernel.img: u-boot/u-boot.bin
	@mkdir -p sdimage
	cp $< $@

sdimage/boot.scr.uimg: boot.scr
	@mkdir -p sdimage
	$(MKIMAGE) -A arm -C none -O linux -T script -n $< -d $< $@
	
sdimage: sdimage/kernel.img sdimage/boot.scr.uimg sdimage/bootcode.bin sdimage/start.elf

gdb:
	$(GDB) -ex 'file $(patsubst %.bin, %.elf, $(kernel))' -ex 'target remote localhost:1234'
