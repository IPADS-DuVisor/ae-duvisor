#!/bin/bash

if [ ! -d "./build" ]; then
    ./configure --target-list=riscv64-softmmu
fi

make -j $(nproc)
