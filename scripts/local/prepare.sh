#!/bin/bash
if [ ! -d "./prepare" ]; then 
    mkdir -p prepare
    # password: ipads123
    scp gitlab-runner@liuyuxuan-pc:~/bin/ubuntu-20.04.2-preinstalled-server-riscv64.qcow2 ./prepare/
fi

git submodule update --init --recursive