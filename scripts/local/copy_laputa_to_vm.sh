#!/bin/bash

first_arg=$1

if test ${first_arg}a = releasea; then
    profile_opt="--release"
    profile=release
else
    profile_opt=""
    profile=debug
fi

if [ ${USER}1 == gitlab-runner1 ]; then
    # for CI environment
    PREPARE="$HOME/prepare"
    profile_opt=""
    profile=debug
else
    PREPARE="./prepare"
fi

echo $profile_opt $profile

cargo clean
cargo build --target=riscv64gc-unknown-linux-gnu $profile_opt
laputa_name=`find target/riscv64gc-unknown-linux-gnu/${profile}/deps/ -type f ! -name '*.*' `
laputa_name_basename=`basename $laputa_name`

# get laputa all the binary names
cargo test --no-run --target=riscv64gc-unknown-linux-gnu $profile_opt
laputa_names=`find ./target/riscv64gc-unknown-linux-gnu/${profile}/deps/ -type f ! -name '*.*' `

## Build test images
sudo rm -r ./tests/integration/test_images/build
./tests/integration/test_images/build.sh ./tests/integration/test_images/build ./tests/integration/

# delete laputa main binary name, so that we get laputa tests binary names
laputa_test_names=${laputa_names/$laputa_name}
mkdir -p mnt
sudo mount $PREPARE/ubuntu-vdisk.img ./mnt
sudo rm -r ./mnt/laputa
sudo mkdir -p ./mnt/laputa/tests_bin
sudo cp scripts/local/run_tests.sh $laputa_name ./mnt/laputa
sudo cp scripts/local/up.sh ./mnt/
sudo cp $laputa_test_names ./mnt/laputa/tests_bin/
sudo mv ./mnt/laputa/$laputa_name_basename ./mnt/laputa/laputa
sudo cp -r src ./mnt/laputa/
sudo cp -r tests ./mnt/laputa/
sudo cp -r test-files-laputa ./mnt/laputa/
sudo umount ./mnt
