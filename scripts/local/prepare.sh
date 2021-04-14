#!/bin/bash
if [ ! -d "./prepare" ]; then 
    mkdir -p prepare
    # password: ipads123
    scp gitlab-runner@liuyuxuan-pc:~/bin/ubuntu-20.04.2-preinstalled-server-riscv64.img ./prepare/
    qemu-img create -f raw ./prepare/linux-kernel-vdisk.img 25G
    mkfs.ext4 ./prepare/linux-kernel-vdisk.img
fi

git submodule update --init --recursive