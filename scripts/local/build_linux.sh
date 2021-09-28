#!/bin/bash
cd linux-laputa
sed -i "s/CONFIG_ULH_FPGA=y/CONFIG_ULH_QEMU=y/g" .config
sed -i "s/# CONFIG_ULH_QEMU is not set/# CONFIG_ULH_FPGA is not set/g" .config

export ARCH=riscv
export CROSS_COMPILE=riscv64-linux-gnu-

make -j16

if [ $? -ne 0 ]; then
    exit -1
fi
cd -
