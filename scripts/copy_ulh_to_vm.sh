sudo losetup /dev/loop14 ~/ubuntu-20.04.2-preinstalled-server-riscv64.img
sudo kpartx -a /dev/loop14
sudo mount /dev/mapper/loop14p1 /mnt

sudo [ -e /mnt/home/ubuntu/laputa ] && sudo rm -r /mnt/home/ubuntu/laputa
sudo cp -r . /mnt/home/ubuntu/laputa

sudo umount /mnt
sudo kpartx -d /dev/loop14
sudo losetup -d /dev/loop14
