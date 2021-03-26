#!/bin/bash

if [ "x$1" != "x" ] && [ $1 == "configure" ]; then
    ./configure --target-list=riscv64-softmmu
fi

make -j $(nproc)
