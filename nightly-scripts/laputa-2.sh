#!/bin/bash

mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
mount -t proc proc /proc

# Setup br0
#ip link add br0 type bridge
#ip link set eth0 master br0
#ip link set br0 up
#ip link set eth0 up
#ip addr add 192.168.254.1/16 dev br0
#ip route add default via 192.168.10.1 dev br0
#/sbin/sshd

# Add vmtap0
#ip tuntap add vmtap0 mode tap user $(whoami)
#ip link set vmtap0 master br0
#ip link set dev vmtap0 up

cd laputa

NR_VCPU=2

./laputa --smp $NR_VCPU \
    --initrd ./test-files-laputa/rootfs-net.img \
    --dtb ./test-files-laputa/smp$NR_VCPU-io.dtb  \
    --kernel ./test-files-laputa/Image \
    --memory 512 \
    --machine laputa_virt
