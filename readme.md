This is a rust baremetal OS project for raspberry pi, hosted on OS X.

# Goal

The goal is a clustered lisp machine running as a sasos on a cluster of raspberry pis. Host lang under lisp is rust.

# Colophon

### Platforms

* Development host: Mac OS X El Capitan
* Development board: Raspberry Pi 2

### Tools

* Shell: http://fishshell.com/
* Package manager: http://brew.sh/
* LLVM cross compilers from here: https://launchpad.net/gcc-arm-embedded

Multirust (nightly): 

    $ brew install multirust

Boot manager: http://elinux.org/RPi_U-Boot

    $ brew install u-boot-tools
    $ git clone git://git.denx.de/u-boot.git
    $ set -x CROSS_COMPILE arm-none-eabi
    $ make rpi_2_defconfig
    $ make -j8 -s   # u-boot.bin

SD image:

    bootcode.bin  # rpi
    start.elf     # rpi
    kernel.img    # mv u-boot.bin kernel.img
    boot.scr.uimg # make boot.scr.uimg

TFTPD:

    $ make tftp

## Updating Rust

You need a libcore for your target architecture in the right place under rustc's sysroot (until multirust includes support for [cross-compilation](https://github.com/brson/multirust/pull/112). The makefile achieves this by using a git submodule holding rust. When you want to update rust, it also checks out the corresponding build of libcore which is used to build libcore.    

    $ make update-rust
