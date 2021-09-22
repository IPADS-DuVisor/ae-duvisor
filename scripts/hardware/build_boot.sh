#!/bin/bash
set -e

export ARCH=riscv
export CROSS_COMPILE=riscv64-linux-gnu-

if test ${1}1 = linux1; then
    cd linux-laputa
    make -j16
    cd ../opensbi-laputa
else
    cd opensbi-laputa
fi

mkdir -p laputa-build

make PLATFORM=generic FW_PAYLOAD_PATH=/home/yuxuanliu/RISC-V/lpt-hw/laputa/linux-laputa/arch/riscv/boot/Image O=laputa-build

sudo dd if=laputa-build/platform/generic/firmware/fw_payload.bin of=/dev/sda1 bs=4096

cd ..
