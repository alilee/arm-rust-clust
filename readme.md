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
* Build: https://www.gnu.org/software/make/
* Boot manager: http://elinux.org/RPi_U-Boot

Multirust (nightly): https://github.com/brson/multirust

    $ brew install multirust

TFTPD:

    $ make tftpd

## Building the SD image

    $ make sdimage

## Updating Rust

You need a libcore for your target architecture in the right place under rustc's sysroot (until multirust includes support for [cross-compilation](https://github.com/brson/multirust/pull/112). The makefile achieves this by using a git submodule holding rust. When you want to update rust, it also checks out the corresponding build of libcore which is used to build libcore.    

    $ make update-rust

