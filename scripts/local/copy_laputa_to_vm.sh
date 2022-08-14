#!/bin/bash

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

echo $build_level

cargo clean
cargo build --target=riscv64gc-unknown-linux-gnu $build_level --features "qemu"
laputa_name=`find target/riscv64gc-unknown-linux-gnu/${build_path}/deps/ -type f ! -name '*.*' `
laputa_name_basename=`basename $laputa_name`

# get laputa all the binary names
cargo test --no-run --target=riscv64gc-unknown-linux-gnu $build_level --features "qemu"
laputa_names=`find ./target/riscv64gc-unknown-linux-gnu/${build_path}/deps/ -type f ! -name '*.*' `

## Build test images
sudo rm -r ./tests/integration/test_images/build
./tests/integration/test_images/build.sh ./tests/integration/test_images/build ./tests/integration/

# delete laputa main binary name, so that we get laputa tests binary names
laputa_test_names=${laputa_names/$laputa_name}
mkdir -p mnt-local
sudo mount $PREPARE/ubuntu-vdisk.img ./mnt-local
sudo rm -r ./mnt-local/laputa
sudo mkdir -p ./mnt-local/laputa/tests_bin
sudo cp scripts/local/run_tests.sh $laputa_name ./mnt-local/laputa
sudo cp $laputa_test_names ./mnt-local/laputa/tests_bin/
sudo mv ./mnt-local/laputa/$laputa_name_basename ./mnt-local/laputa/laputa
sudo cp -r src ./mnt-local/laputa/
sudo cp -r tests ./mnt-local/laputa/
sudo cp -r test-files-laputa ./mnt-local/laputa/

sudo cp -r test-files-laputa/multi-vm-test ./mnt-local/

sudo umount ./mnt-local
