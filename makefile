CPU=arm926ej-s

TARGET=arm-none-eabi
AS=arm-none-eabi-as
CC=arm-none-eabi-gcc
RC=rustc
LD=arm-none-eabi-ld
OBJCOPY=arm-none-eabi-objcopy
OBJDUMP=arm-none-eabi-objdump

OBJS=$(patsubst %,%.o,$(basename $(wildcard *.[cs] *.rs)))

ASFLAGS = -mcpu=cortex-a8
CFLAGS = -Og

build/%.o: src/%.s
	@mkdir -p $(shell dirname $@)
	$(AS) $(ASFLAGS) -g $< -o $@

%.elf: %.ld $(OBJS)
	$(LD) -T $^ -o $@

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -dS $< > $*.code

kernel := build/kernel
rust_os := target/$(TARGET)/debug/arm.o
linker_script := src/linker.ld

assembly_source_files := $(wildcard src/*.s)
assembly_object_files := $(patsubst src/%.s, build/%.o, $(assembly_source_files))

.PHONY: all clean run iso cargo

all: $(kernel)

clean:
	@cargo clean
	@rm -rf build

$(kernel).elf: cargo $(rust_os) $(assembly_object_files) $(linker_script)
	$(LD) -nt -nostdlib --gc-sections -T $(linker_script) -o $@ $(rust_os) $(assembly_object_files)

$(kernel): $(kernel).elf
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -dS $< > $(kernel).code

cargo:
	@cargo rustc --target $(TARGET) -- -Z no-landing-pads --emit=obj
