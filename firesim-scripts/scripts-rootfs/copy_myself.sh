cd ~/sim_slot_0

sudo e2fsck -yf linux-uniform0-br-base.img

sudo mount linux-uniform0-br-base.img  linux-uniform0-br-base-mnt

cd linux-uniform0-br-base-mnt/

sudo e2fsck -yf ubuntu-vdisk.img

sudo mount ubuntu-vdisk.img root/

sudo cp -rf ~/mnt-firesim/* root/

cd root/

sudo e2fsck -yf blk-dev.img

sudo mount blk-dev.img root/

sudo cp -rf ~/scripts-rootfs/* root/

sleep 1

sudo umount root/

cd ..

sleep 1

sudo umount root/

cd ..

sudo umount linux-uniform0-br-base-mnt

cd ..

echo copy succeed for slot 0

cd ~/sim_slot_1

sudo e2fsck -yf linux-uniform1-br-base.img

sudo mount linux-uniform1-br-base.img  linux-uniform1-br-base-mnt

cd linux-uniform1-br-base-mnt/

sudo mount ubuntu-vdisk.img root/

cd root/

sudo e2fsck -yf blk-dev.img

sudo mount blk-dev.img root/

sudo cp -rf ~/scripts-rootfs/* root/

sleep 1

sudo umount root/

cd ..

sleep 1

sudo umount root/

cd ..

sudo umount linux-uniform1-br-base-mnt

cd ..

echo copy succeed for slot 1
