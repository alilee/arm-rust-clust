TARGET=arm-none-eabi
AS=$(TARGET)-as
LD=$(TARGET)-ld
OBJCOPY=$(TARGET)-objcopy
OBJDUMP=$(TARGET)-objdump
MKIMAGE=mkimage
QEMU=qemu-system-arm

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
image := build/image/uImage

rust_os := target/$(TARGET)/debug/libarm.a
linker_script := src/linker.ld

sysroot := $(shell rustc --print sysroot)
rustlib_dir := $(sysroot)/lib/rustlib/$(TARGET)/lib
libcore := $(rustlib_dir)/libcore.rlib

assembly_source_files := $(wildcard src/*.s)
assembly_object_files := $(patsubst %.s, %.o, $(assembly_source_files))

.PHONY: all clean cargo

all: $(image)

clean:
	@cargo clean
	@rm -rf build
	@rm $(libcore)

$(image): $(kernel)
	@mkdir -p $(shell dirname $@)
	$(MKIMAGE) -A arm -C gzip -O linux -T kernel -d $< -a 0x10000 -e 0x10000 $@
	chmod 644 $@

$(libcore): $(shell find core/src/ -type f -name '*.rs')
	@mkdir -p $(shell dirname $@)
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
	@cargo build --target $(TARGET) --verbose

qemu: $(kernel)
	$(QEMU) -M $(BOARD) -cpu $(CPU) -m 256M -nographic -s -S -kernel $(kernel)
	