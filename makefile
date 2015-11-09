CPU=arm926ej-s

AS=arm-none-eabi-as
CC=arm-none-eabi-gcc
LD=arm-none-eabi-ld
OBJCOPY=arm-none-eabi-objcopy
OBJDUMP=arm-none-eabi-objdump

OBJS=$(patsubst %,%.o,$(basename $(wildcard *.[cs])))

%.o: %.s
	$(AS) -mcpu=$(CPU) -g $< -o $@

%.o: %.c
	$(CC) -c -mcpu=$(CPU) -g $< -o $@

%.elf: %.ld $(OBJS)
	$(LD) -T $^ -o $@

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@
	$(OBJDUMP) -dS $< > $*.code

build: test.bin

clean: 
	$(RM) *.bin *.elf *.o

