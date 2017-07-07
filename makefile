TARGET=aarch64-unknown-linux-gnu
CURL=curl
AS=$(TARGET)-as
GCC=$(TARGET)-gcc
LD=$(TARGET)-ld
OBJCOPY=$(TARGET)-objcopy
OBJDUMP=$(TARGET)-objdump
MKIMAGE=mkimage
QEMU=qemu-system-aarch64
GDB=gdb

BOARD=virt
CPU=cortex-a53

ASFLAGS = -mcpu=$(CPU) -g -a
CFLAGS = -mcpu=$(CPU) -g
LDFLAGS = --gc-sections

%.bin: %
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -dS $< > $*.code
	$(OBJDUMP) -d $< > $*.s

kernel := target/$(TARGET)/debug/arc
tftpboot_rpi := /private/tftpboot/rpi
image := $(tftpboot_rpi)/uImage

sdimage_dir := deploy/sdimage

.PHONY: all clean qemu update-rust tftpd sdimage gdb test doc

all: $(kernel).bin

clean:
	@cargo clean
	@rm -rf $(sdimage_dir)
	@rm $(image)

test:
	@cargo test

doc:
	@cargo doc --open

$(image): $(kernel).bin
	$(MKIMAGE) -A arm -C gzip -O linux -T kernel -d $< -a 0x10000 -e 0x10000 $@
	@chmod 644 $@

$(kernel):
	cargo build

qemu: $(kernel).bin
	$(QEMU) -M $(BOARD) -cpu $(CPU) -m 256M -nographic -s -S -kernel $<

gdb: $(kernel)
	$(GDB) -iex 'file $(kernel)' -iex 'target remote localhost:1234'

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
