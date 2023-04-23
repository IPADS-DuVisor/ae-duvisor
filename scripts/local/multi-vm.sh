#!/bin/bash

cd laputa
./laputa --smp 2 --initrd ./test-files-laputa/rootfs-guest.img --dtb ./test-files-laputa/smp2-io.dtb  --kernel ./test-files-laputa/Image --memory 1024 --machine laputa_virt
