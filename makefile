TARGET=arm-none-eabi
AS=$(TARGET)-as
LD=$(TARGET)-ld
OBJCOPY=$(TARGET)-objcopy
OBJDUMP=$(TARGET)-objdump

ASFLAGS = -mcpu=cortex-a8 -g
LDFLAGS = --gc-sections

%.o: %.s
	$(AS) $(ASFLAGS) $< -o $@

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -dS $< > $*.code
	$(OBJDUMP) -d $< > $*.s

kernel := build/kernel.bin

rust_os := target/$(TARGET)/debug/libarm.a
linker_script := src/linker.ld

sysroot := $(shell rustc --print sysroot)
rustlib := $(sysroot)/lib/rustlib/$(TARGET)/lib

assembly_source_files := $(wildcard src/*.s)
assembly_object_files := $(patsubst %.s, %.o, $(assembly_source_files))

.PHONY: all clean cargo

all: $(kernel)

clean:
	@cargo clean
	@rm -rf build
	@rm $(rustlib)/libcore.rlib

$(rustlib)/libcore.rlib: $(shell find core/src/ -type f -name '*.rs')
	rustc core/src/lib.rs \
	  --crate-name core \
	  --crate-type lib \
	  --out-dir $(rustlib) \
	  --emit=link \
	  -g \
	  --target $(TARGET) 

build/kernel.elf: $(rust_os) $(assembly_object_files) $(linker_script)
	@mkdir -p $(shell dirname $@)
	$(LD) $(LDFLAGS) -T $(linker_script) -o $@ $(assembly_object_files) $(rust_os)

$(rust_os): $(wildcard src/*.rs) Cargo.toml $(rustlib)/libcore.rlib
	@cargo build --target $(TARGET) --verbose

