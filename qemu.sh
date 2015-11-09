#!/bin/sh 

BOARD=realview-pb-a8
IMAGE=test.bin
QEMU=qemu-system-arm
$QEMU -M $BOARD -m 128M -nographic -s -S -kernel $IMAGE
