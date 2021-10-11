#!/bin/bash

sudo rm -r target
sudo rm -r tests/integration/test_images/build
sudo rm tests/integration/*.img
first_arg=$1

if [ -z "$first_arg" ]; then
    build_level=""
    build_path=debug
elif test ${first_arg} = release; then
    build_level="--release"
    build_path=release
else
    echo "Wrong arg."
    exit
fi

if [ `hostname` == liuyuxuan ]; then
    # for CI environment
    PREPARE=${PREPARE:-"$HOME/prepare"}
#    build_level=""
#    build_path=debug
else
    PREPARE="./prepare"
fi

echo prepare dirctory is ${PREPARE}

echo `hostname`

build_level="--release"
build_path=release

echo $build_level

cargo clean
cargo build --target=riscv64gc-unknown-linux-gnu $build_level --features "xilinx"
laputa_name=`find target/riscv64gc-unknown-linux-gnu/${build_path}/deps/ -type f ! -name '*.*' `
laputa_name_basename=`basename $laputa_name`

# get laputa all the binary names
cargo test --no-run --target=riscv64gc-unknown-linux-gnu $build_level --features "xilinx"
laputa_names=`find ./target/riscv64gc-unknown-linux-gnu/${build_path}/deps/ -type f ! -name '*.*' `
## Build test images
sudo rm -r ./tests/integration/test_images/build
./tests/integration/test_images/build.sh ./tests/integration/test_images/build ./tests/integration/

# delete laputa main binary name, so that we get laputa tests binary names
laputa_test_names=${laputa_names/$laputa_name}

mkdir -p mnt-firesim

#sudo mount $PREPARE/ubuntu-vdisk.img ./mnt
sudo rm -r ./mnt-firesim/laputa
sudo mkdir -p ./mnt-firesim/laputa/tests_bin
# copy scripts used by laputa
sudo cp -rf scripts/export/*  ./mnt-firesim/
# copy laputa binary
sudo cp $laputa_name ./mnt-firesim/laputa
sudo mv ./mnt-firesim/laputa/$laputa_name_basename ./mnt-firesim/laputa/laputa
sudo cp -r src ./mnt-firesim/laputa/
sudo cp -r tests ./mnt-firesim/laputa/
sudo cp -r test-files-laputa ./mnt-firesim/laputa/
sudo rm ./mnt-firesim/laputa/test-files-laputa/800m-image.img
sudo rm ./mnt-firesim/laputa/test-files-laputa/fake.img
sudo rm ./mnt-firesim/laputa/test-files-laputa/rootfs-vm-wrong.img
