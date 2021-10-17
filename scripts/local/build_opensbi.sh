#!/bin/bash
cd opensbi-laputa
export CROSS_COMPILE=riscv64-unknown-linux-gnu-
make PLATFORM=generic
if [ $? -ne 0 ]; then
    exit -1
fi
cd -
