#!/bin/bash
cd linux-laputa
./scripts-laputa/local/docker_build_linux.sh 
if [ $? -ne 0 ]; then
    exit -1
fi
cd ../
rm -r mnt
mkdir -p mnt
sudo mount prepare/linux-kernel-vdisk.img mnt/
echo "mnt directory mounted"
sudo cp -r linux-laputa mnt/
sudo umount mnt
echo "mnt directory umounted"