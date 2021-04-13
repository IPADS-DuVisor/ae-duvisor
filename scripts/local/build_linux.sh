#!/bin/bash
cd linux-laputa && ./scripts-laputa/local/docker_build_linux.sh && cd ../
rm -r mnt
mkdir -p mnt
sudo mount prepare/linux-kernel-vdisk.img mnt/
sudo cp -r linux-laputa mnt/
sudo umount mnt
