# arm-rust-clust
A bare-metal operating system for a cluster of small ARM boards, like the Raspberry Pi or CHIP.

## Goal
This is an etude in Rust and operating system development, with a focus on a single address 
space (SASOS) approach. I would like to put a statically-typed lisp as the OS's second 
language after Rust.

## How to begin
Start with the makefile.

## What is good (so far)
1. It think the test setup is good. Unit tests are run on the host. Rust integration tests are run under QEMU.
1. The arch abstraction is good, which helps the policy abstraction.
1. The use of Rust is more complete than most. We start the processor in Rust, and really only need global_asm
for exception handler entry.
1. Documentation and test coverage are significantly improved in this second try.
1. My implementation of map_translation handles blocks and contiguous ranges.

## What could improve
1. Reducing unsafe code even further, and justifying the necessary use of unsafe.
1. Would be good to build the x86 kernel in tandem, but I have avoided xbuild for arm, so I'll wait.
1. I am not sure enough of traits, monomorphisation and statics to be confident in module interfaces.

## Influences and thanks
* [Julia Evans](http://jvns.ca/blog/2014/03/21/my-rust-os-will-never-be-finished/)
* [Bodil Stokke](https://skillsmatter.com/skillscasts/4484-build-your-own-lisp-for-great-justice)
* [Philipp Oppermann](http://os.phil-opp.com/) Rust and great, but x86-centric
* [Lucas Hartmann](http://interim-os.com/) We follow in his footsteps!
* [Dawid Ciężarkiewicz](https://github.com/dpc/titanos) And Dawid's too!
* [Andre Richter](https://github.com/rust-embedded/rust-raspi3-tutorial) And cortex-a crate.
* [rCoreOS](https://github.com/rcore-os/rCore) Multi-arch
* [Ilya Kartashov](https://lowenware.com/leos/) 
