cd app && make && cd -

mkdir -p mnt
sudo mount ./prepare/ubuntu-vdisk.img ./mnt
sudo cp -r app ./mnt/laputa
sudo umount ./mnt