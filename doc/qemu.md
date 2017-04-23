
make qemu

Ctrl-a x
  Exit emulator
--------




--------
./aarch64-softmmu/qemu-system-aarch64 -machine virt -cpu cortex-a57 -machine type=virt -nographic -smp 1 -m 2048 -kernel aarch64-linux-3.15rc2-buildroot.img  --append "console=ttyAMA0"

qemu-system-aarch64 -machine virt -cpu cortex-a53 -machine type=virt -nographic -smp 1 -m 2048 -kernel aarch64-current-linux-initrd-guest.img  --append "console=ttyAMA0"

qemu-system-aarch64 -machine virt -cpu cortex-a57 -machine type=virt -nographic -smp 1 -m 2048 -kernel aarch64-current-linux-initrd-guest.img --append "console=ttyAMA0" -fsdev local,id=r,path=.,security_model=none -device virtio-9p-device,fsdev=r,mount_tag=r

qemu-system-aarch64 -machine virt -cpu cortex-a53 -machine type=virt -nographic -smp 1 -m 2048 -kernel aarch64-linux-3.15rc2-buildroot.img --append "console=ttyAMA0" -virtfs local,id=r,path=./mnt,security_model=none


--------
qemu-system-aarch64 -m 1024 -cpu cortex-a57 -nographic -machine virt \
  -kernel linaro/Image -append 'root=/dev/vda2 rw rootwait mem=1024M console=ttyAMA0,38400n8' \
  -netdev user,id=user0 -device virtio-net-device,netdev=user0  -device virtio-blk-device,drive=disk \
  -drive if=none,id=disk,file=linaro/vexpress64-openembedded_minimal-armv8-gcc-4.9_20150620-722.img


mkdir debian
wget http://http.us.debian.org/debian/dists/jessie/main/installer-arm64/current/images/netboot/debian-installer/arm64/linux -O debian/linux
wget http://http.us.debian.org/debian/dists/jessie/main/installer-arm64/current/images/netboot/debian-installer/arm64/initrd.gz -O debian/initrd.gz
qemu-img create -f raw debian/root.img 10G  
qemu-system-aarch64 -m 1024 -cpu cortex-a57 -nographic -machine virt \
  -kernel debian/linux -initrd debian/initrd.gz -append 'mem=1024M console=ttyAMA0,38400n8' \
  -netdev user,id=user0 -device virtio-net-device,netdev=user0 \
  -device virtio-blk-device,drive=disk -drive if=none,id=disk,file=debian/root.img,format=raw \
  -redir tcp:8022::22

qemu-system-aarch64 -m 1024 -cpu cortex-a57 -nographic -machine virt \
  -kernel linaro/Image -append 'root=/dev/vda1 rootwait rw mem=1024M console=ttyAMA0,38400n8' \
  -netdev user,id=user0 -device virtio-net-device,netdev=user0 \
  -device virtio-blk-device,drive=disk -drive if=none,id=disk,file=debian/root.img,format=raw \
  -redir tcp:8022::22


run:

qemu-system-aarch64 -m 1024 -cpu cortex-a57 -nographic -machine virt \
  -kernel linaro/Image -append 'root=/dev/vda2 rw mem=1024M console=ttyAMA0,38400n8' \
  -netdev user,id=user0 -device virtio-net-device,netdev=user0 \
  -device virtio-blk-device,drive=disk -drive if=none,id=disk,file=hda.img \
  -redir tcp:8022::22


qemu-system-aarch64 -m 1024 -cpu cortex-a57 -nographic -machine virt \
  -kernel linaro/Image -append 'root=/dev/vda2 rw rootwait mem=1024M console=ttyAMA0,38400n8' \
  -netdev user,id=user0 -device virtio-net-device,netdev=user0 \
  -device virtio-blk-device,drive=d1 -drive if=none,id=d1,file=linaro/root.img,format=raw \
  -redir tcp:8022::22

  -device virtio-blk-device,drive=d2 -drive if=none,id=d2,file=vdb-linaro.img \






--------
qemu-img create -f qcow hda-ubuntu.img 10G  
qemu-system-aarch64 -m 1024 -cpu cortex-a57 -nographic -machine virt \
  -kernel ubuntu/vmlinuz-4.2.0-23-generic -append 'root=/dev/vda2 rootwait rw mem=1024M console=ttyAMA0,38400n8' \
  -netdev user,id=user0 -device virtio-net-device,netdev=user0 \
  -device virtio-blk-device,drive=disk -drive if=none,id=disk,file=ubuntu/vda.img \
  -redir tcp:8022::22





-----------


../qemu.git/aarch64-softmmu/qemu-system-aarch64 -cpu cortex-a57 -machine type=ranchu -m 4096 -kernel ./ranchu-kernel/Image -append 'console=ttyAMA0,38400 keep
_bootcon' -monitor stdio -initrd ranchu-build/ramdisk.img -drive index=2,id=userdata,file=ranchu-build/userdata.img -device virtio-blk-device,drive=us
erdata -device virtio-blk-device,drive=cache -drive index=1,id=cache,file=ranchu-build/cache.img -device virtio-blk-device,drive=system -drive index=0,id=system,file=ranchu-build/system.img -
netdev user,id=mynet -device virtio-net-device,netdev=mynet -show-cursor
