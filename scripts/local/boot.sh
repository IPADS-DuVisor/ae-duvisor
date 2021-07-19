#!/bin/bash

if [ ${USER}1 == gitlab-runner1 ]; then
    # for CI environment
    PREPARE="$HOME/prepare"
else
    PREPARE="./prepare"
fi

if ! [ -e /sys/class/net/br0 ]; then
    echo -n "Please create br0 first"
    exit 1
fi

if ! [ -e /sys/class/net/tap0 ]; then
    echo -n "Creating tap0"
    sudo ip tuntap add tap0 mode tap user $(whoami)
    sudo ip link set tap0 master br0
    sudo ip link set dev br0 up
    sudo ip link set dev tap0 up
fi

MACADDR=66:22:33:44:55:00
ROMFILE=./qemu-laputa/pc-bios/efi-virtio.rom
#ROMFILE=./qemu-laputa/pc-bios/efi-e1000e.rom

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
    -drive if=none,id=vdisk,file=$PREPARE/ubuntu-vdisk.img,format=raw \
    -device virtio-net-pci,netdev=vnet,mac=$MACADDR,romfile=$ROMFILE \
    -netdev tap,id=vnet,ifname=tap0,script=no
    #-device e1000e,netdev=vnet,mac=$MACADDR,romfile=$ROMFILE \
    #-netdev user,id=vnet,hostfwd=tcp::5555-:22
