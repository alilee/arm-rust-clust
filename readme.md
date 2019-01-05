This is a rust baremetal OS project for raspberry pi, hosted on OS X.

# Goal

The goal is a clustered lisp machine running as a sasos on a cluster of arm SOC boards. Host lang under lisp is rust.

# Colophon

### Platforms

* Development host: Mac OS X El Capitan
* Development board: Raspberry Pi 2, Pine64 plus

### Tools

* Shell: http://fishshell.com/
* Package manager: http://brew.sh/
* ARM bintools from here: https://launchpad.net/gcc-arm-embedded
* Build: https://www.gnu.org/software/make/
* Boot manager: http://elinux.org/RPi_U-Boot
* Rust: Multirust (nightly): https://github.com/brson/multirust (available from brew)

## Start TFTPD

So we don't have to mess with updating the kernel on the SD card, we set it up to use a boot manager to grab the kernel from the development machine using TFTP. OS X includes a TFTPD, but you have to start it. The included .plist adds an option to chroot to the server directory so that files are named relative to inside it.

    $ make tftpd

## Building the SD image

This gives you just what you need to start an rpi2, boot the bootloader, and have it start a kernel using TFTP. Key network settings for TFTP in boot.scr. You can probably control this outside if you have a more configurable DHCP host than mine.

    $ make sdimage

## Updating Rust

You need a libcore for your target architecture in the right place under rustc's sysroot (until multirust includes support for [cross-compilation](https://github.com/brson/multirust/pull/112). The makefile achieves this by using a git submodule holding rust. When you want to update rust, it also checks out the corresponding commit of libcore which is used to build libcore.    

    $ make update-rust

## Build kernel

Builds the kernel and copies the u-boot ready image into the TFTPD server directory. Also creates some useful debugging files.

    $ make

# Thanks

* [Julia Evans](http://jvns.ca/blog/2014/03/21/my-rust-os-will-never-be-finished/)
* [Bodil Stokke](https://skillsmatter.com/skillscasts/4484-build-your-own-lisp-for-great-justice)
* [Philipp Oppermann](http://os.phil-opp.com/)
* [Lucas Hartmann](http://interim-os.com/) We follow in his footsteps!
* [Dawid Ciężarkiewicz](https://github.com/dpc/titanos) And Dawid's too!
* [Andre Richter](https://github.com/rust-embedded/rust-raspi3-tutorial)
