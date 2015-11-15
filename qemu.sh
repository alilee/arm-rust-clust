#!/bin/sh 

BOARD=versatileab
CPU=cortex-a8
IMAGE=test.bin
QEMU=qemu-system-arm
$QEMU -M $BOARD -cpu $CPU -m 128M -nographic -s -S -kernel $IMAGE
