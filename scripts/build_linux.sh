#!/bin/bash
cd linux-laputa
./scripts-laputa/docker_build_linux.sh 
if [ $? -ne 0 ]; then
    exit -1
fi
cd ../
rm -r mnt
mkdir -p mnt
sudo mount ~/linux-kernel-vdisk.img mnt/
echo "mnt directory mounted"
sudo cp -r linux-laputa mnt/
sudo umount mnt
echo "mnt directory umounted"