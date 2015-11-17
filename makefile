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

%.o: %.s
	$(AS) $(ASFLAGS) -g $< -o $@

%.o: %.c
	$(CC) -c $(CFLAGS) -g $< -o $@

%.o: %.rs lib-arm/libcore.rlib
	$(RC) --target=$(TARGET) $(CFLAGS) -L lib-arm -g $< -o $@

%.elf: %.ld $(OBJS)
	$(LD) -T $^ -o $@

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -dS $< > $*.code

build: test.bin

clean: 
	$(RM) *.bin *.o *.code

lib-arm/libcore.rlib: 
	$(RC) -C opt-level=2 -Z no-landing-pads --target $(TARGET) -g libcore/lib.rs --out-dir lib-arm

