cd ~/sim_slot_0

sudo mount linux-uniform-kvm0-br-base.img  linux-uniform-kvm0-br-base-mnt

cd linux-uniform-kvm0-br-base-mnt/

sudo mount ubuntu-vdisk.img root/

sudo cp -rf ~/mnt-firesim/* root/

cd root/

sudo mount blk-dev.img root/

sudo cp -f ~/scripts-rootfs/* root/

sleep 1

sudo umount root/

cd ..

sleep 1

sudo umount root/

cd ..

sudo umount linux-uniform-kvm0-br-base-mnt

cd ..

echo copy succeed for slot 0

cd ~/sim_slot_1

sudo mount linux-uniform-kvm1-br-base.img  linux-uniform-kvm1-br-base-mnt

cd linux-uniform-kvm1-br-base-mnt/

sudo mount ubuntu-vdisk.img root/

cd root/

sudo mount blk-dev.img root/

sudo cp -f ~/scripts-rootfs/* root/

sleep 1

sudo umount root/

cd ..

sleep 1

sudo umount root/

cd ..

sudo umount linux-uniform-kvm1-br-base-mnt

cd ..

echo copy succeed for slot 1
