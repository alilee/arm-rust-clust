#!/usr/local/bin/fish

rust-objcopy -O binary $argv[1] $argv[1].bin
echo qemu-system-aarch64 -M virt -cpu cortex-a53 -m 256M -nographic -semihosting -dtb qemu.dtb -kernel $argv[1].bin
qemu-system-aarch64 -M virt -cpu cortex-a53 -m 256M -nographic -semihosting -dtb qemu.dtb -kernel $argv[1].bin
