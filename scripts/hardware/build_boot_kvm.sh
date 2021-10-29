#!/bin/bash
set -e

export ARCH=riscv
export CROSS_COMPILE=riscv64-linux-gnu-

if test ${1}1 = linux1; then
    cd linux-kvm
    make -j16
    cd ../opensbi-laputa
else 
    cd opensbi-laputa
fi


make PLATFORM=generic FW_PAYLOAD_PATH=/home/yuxuanliu/RISC-V/laputa/linux-kvm/arch/riscv/boot/Image

sudo dd if=build/platform/generic/firmware/fw_payload.bin of=/dev/sdc1 bs=4096

cd ..
