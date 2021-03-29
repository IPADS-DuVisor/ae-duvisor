#!/bin/bash

export CROSS_COMPILE=riscv64-linux-gnu-

if [ ! -f "./.config" ]; then
    make ARCH=riscv mrproper defconfig
    make  ARCH=riscv defconfig
fi
make ARCH=riscv all -j $(nproc)