# get laputa main binary name
cargo clean
cargo build --target=riscv64gc-unknown-linux-gnu
laputa_name=`find target/riscv64gc-unknown-linux-gnu/debug/deps/ -type f ! -name '*.*' `

# get laputa all the binary names
cargo test --no-run --target=riscv64gc-unknown-linux-gnu
laputa_names=`find ./target/riscv64gc-unknown-linux-gnu/debug/deps/ -type f ! -name '*.*' `

# Build test images
./testfiles/integration/test_images/build.sh ./testfiles/integration/test_images/build ./testfiles/integration/

# delete laputa main binary name, so that we get laputa tests binary names
laputa_test_names=${laputa_names/$laputa_name}

sudo guestmount -a ~/ubuntu-20.04.2-preinstalled-server-riscv64.qcow2 -m /dev/sda1 /mnt

sudo [ -e /mnt/home/ubuntu/laputa ] && sudo rm -r /mnt/home/ubuntu/laputa
sudo mkdir -p /mnt/home/ubuntu/laputa/tests_bin/
sudo cp -r -t /mnt/home/ubuntu/laputa/ ./testfiles
sudo cp -t /mnt/home/ubuntu/laputa/tests_bin/ $laputa_test_names
sudo cp $laputa_name  /mnt/home/ubuntu/laputa/laputa
sudo cp ./scripts/run_tests.sh  /mnt/home/ubuntu/laputa/

sudo guestunmount  /mnt
