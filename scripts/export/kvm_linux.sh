#!/bin/bash

mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
mount -t proc proc /proc

./lkvm-static run -m 512 -c8 --console serial -p "console=ttyS0 earlycon=uart8250,mmio,0x3f8" -k ./laputa/test-files-laputa/Image --initrd ./laputa/test-files-laputa/rootfs-net.img -d /blk-dev.img \
--network trans=mmio,mode=tap,tapif=tap0
