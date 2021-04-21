cd linux-laputa/tools/laputa-test
make 
cd -

mkdir -p mnt
sudo mount ./prepare/ubuntu-vdisk.img ./mnt
sudo cp linux-laputa/tools/laputa-test/ioctl_test ./mnt/laputa
sudo umount ./mnt