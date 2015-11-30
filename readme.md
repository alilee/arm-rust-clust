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

    brew install multirust

Boot manager: http://elinux.org/RPi_U-Boot

    brew install u-boot-tools
    git clone git://git.denx.de/u-boot.git
    set -x CROSS_COMPILE arm-none-eabi
    make rpi_2_defconfig
    make -j8 -s

SD image:

    bootcode.bin  # rpi
    start.elf     # rpi
    kernel.img    # mv u-boot.bin kernel.img
    boot.scr.uimg # make boot.scr.uimg

TFTP:

    sudo launchctl load -F tftpd.plist
    sudo launchctl start com.apple.tftpd
    sudo ln -s /private/tftpboot/rpi build/image

# Setup

You need a libcore for your target architecture in the right place under rustc's sysroot (until multirust includes support for [cross-compilation](https://github.com/brson/multirust/pull/112). The makefile achieves this by expecting a symlink to the core directory of the same checkout as multirust's instance of nightly.     

    $ ll core/*
    -rw-r--r--@ 1 alilee  staff   112B 17 Nov 20:13 Cargo.toml
    lrwxr-xr-x  1 alilee  staff    22B 17 Nov 19:58 src -> ../../rust/src/libcore

This means you need to checkout the same version of the rust sourcecode that you are running.

    $ rustc --version
    $ cd ~/rust
    $ git pull
    $ git checkout xxxx
    