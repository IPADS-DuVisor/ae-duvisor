#!/bin/bash

./qemu-laputa/build/riscv64-softmmu/qemu-system-riscv64 \
-machine virt -nographic -m 2048 -smp 4 \
-bios /usr/lib/riscv64-linux-gnu/opensbi/generic/fw_jump.elf \
-kernel /usr/lib/u-boot/qemu-riscv64_smode/uboot.elf \
-device virtio-net-device,netdev=eth0 -netdev user,id=eth0,hostfwd=tcp::2333-:22 \
-drive file=./prepare/ubuntu-20.04.2-preinstalled-server-riscv64.img,format=raw,if=virtio \
-drive file=./prepare/linux-kernel-vdisk.img,format=raw,if=virtio

