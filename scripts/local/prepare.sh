#!/bin/bash
mkdir -p prepare
# password: ipads123
scp gitlab-runner@liuyuxuan-pc:~/bin/ubuntu-20.04.2-preinstalled-server-riscv64.qcow2 ./prepare/

# IMPORTANT!!!!! , this should be updated to compile from submodules
scp gitlab-runner@liuyuxuan-pc:~/bin/qemu-system-riscv64 ./prepare/
