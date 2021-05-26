#!/bin/bash

if [ $USER == gitlab-runner ]; then
    # for CI environment
    PREPARE="$HOME/prepare"
else
    PREPARE="./prepare"
fi

./qemu-laputa/build/riscv64-softmmu/qemu-system-riscv64 \
    -nographic \
    -cpu rv64,x-h=true,x-z=true \
    -smp 4 \
    -m 8G \
    -machine virt \
    -bios ./opensbi-laputa/build/platform/generic/firmware/fw_jump.elf \
    -kernel ./linux-laputa/arch/riscv/boot/Image \
    -initrd $PREPARE/rootfs.img \
    -append "root=/dev/ram rw console=ttyS0 earlycon=sbi" \
    -device virtio-blk-pci,drive=vdisk \
    -drive if=none,id=vdisk,file=$PREPARE/ubuntu-vdisk.img,format=raw
