TARGET=arm-none-eabi
AS=$(TARGET)-as
LD=$(TARGET)-ld
OBJCOPY=$(TARGET)-objcopy
OBJDUMP=$(TARGET)-objdump

ASFLAGS = -mcpu=cortex-a8 -g

%.o: %.s
	$(AS) $(ASFLAGS) $< -o $@

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -dS $< > $*.code

kernel := build/kernel.bin

rust_os := target/$(TARGET)/debug/libarm.a
linker_script := src/linker.ld

assembly_source_files := $(wildcard src/*.s)
assembly_object_files := $(patsubst %.s, %.o, $(assembly_source_files))

.PHONY: all clean cargo

all: $(kernel)

clean:
	@cargo clean
	@rm -rf build

build/kernel.elf: cargo $(rust_os) $(assembly_object_files) $(linker_script)
	@mkdir -p $(shell dirname $@)
	$(LD) --gc-sections -T $(linker_script) -o $@ $(assembly_object_files) $(rust_os)

cargo:
	@cargo rustc --target $(TARGET) -- -g


# 	@cargo rustc --target $(TARGET) -- -g -Z no-landing-pads --emit=obj
