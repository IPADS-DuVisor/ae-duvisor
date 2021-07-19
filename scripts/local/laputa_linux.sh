#!/bin/bash

mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
mount -t proc proc /proc

ip link set eth0 up
ip addr add 192.168.254.1/16 dev eth0
ip route add default via 192.168.10.1 dev eth0

/sbin/sshd

cd laputa
#./laputa --smp 1 --initrd ./test-files-laputa/rootfs-vm.img --dtb ./test-files-laputa/vmlinux.dtb  --kernel ./test-files-laputa/Image --memory 1024 --machine laputa_virt
./laputa --smp 1 --initrd ./test-files-laputa/rootfs-net.img --dtb ./test-files-laputa/vmlinux.dtb  --kernel ./test-files-laputa/Image --memory 1024 --machine laputa_virt
