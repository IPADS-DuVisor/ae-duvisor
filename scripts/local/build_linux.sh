#!/bin/bash

export CROSS_COMPILE=riscv64-linux-gnu-

if [ "x$1" != "x" ] && [ $1 == "configure" ]; then
    make ARCH=riscv mrproper defconfig
    make  ARCH=riscv defconfig
fi
make ARCH=riscv all -j $(nproc)