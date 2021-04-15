#!/bin/bash
cd qemu-laputa && ./scripts-laputa/local/docker_build_qemu.sh && cp ./build/qemu-system-riscv64 ~/bin/ && cd ../