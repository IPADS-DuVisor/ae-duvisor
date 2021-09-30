#!/bin/bash
cd linux-laputa
cp .config-qemu .config
export ARCH=riscv
export CROSS_COMPILE=riscv64-linux-gnu-

make -j16

if [ $? -ne 0 ]; then
    exit -1
fi
cd -
