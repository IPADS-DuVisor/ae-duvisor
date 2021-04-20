#!/bin/bash

./qemu-laputa/build/riscv64-softmmu/qemu-system-riscv64 \
    -nographic \
    -cpu rv64,x-h=true,x-z=true \
    -smp 4 \
    -m 16G \
    -machine virt \
    -bios /usr/lib/riscv64-linux-gnu/opensbi/generic/fw_jump.elf \
    -kernel ./linux-laputa/arch/riscv/boot/Image \
    -initrd ./prepare/rootfs.img \
    -append "root=/dev/ram rw console=ttyS0 earlycon=sbi" \
    -device virtio-blk-pci,drive=vdisk \
    -drive if=none,id=vdisk,file=./prepare/ubuntu-vdisk.img,format=raw
