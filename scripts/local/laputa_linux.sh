#!/bin/bash

mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
mount -t proc proc /proc

cd laputa
./laputa --smp 1 --initrd ./test-files-laputa/rootfs-vm.img --dtb ./test-files-laputa/vmlinux.dtb  --kernel ./test-files-laputa/Image --memory 1024 --machine laputa_virt
