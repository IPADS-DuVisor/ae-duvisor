#!/bin/bash

mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
mount -t proc proc /proc

#sysctl -p
#mount -t hugetlbfs none /mnt/huge
