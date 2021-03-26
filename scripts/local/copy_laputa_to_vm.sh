#!/bin/bash
cargo clean
cargo build --target=riscv64gc-unknown-linux-gnu
laputa_name=`find target/riscv64gc-unknown-linux-gnu/debug/deps/ -type f ! -name '*.*' `
laputa_name_basename=`basename $laputa_name`

# get laputa all the binary names
cargo test --no-run --target=riscv64gc-unknown-linux-gnu
laputa_names=`find ./target/riscv64gc-unknown-linux-gnu/debug/deps/ -type f ! -name '*.*' `

# delete laputa main binary name, so that we get laputa tests binary names
laputa_test_names=${laputa_names/$laputa_name}

rsync -av -e 'ssh -p 2333'  --exclude='.*' --exclude='target' --exclude='prepare' --exclude='scripts' $PWD ubuntu@localhost:~/

# used for local test
ssh -p 2333 ubuntu@localhost "mkdir -p laputa/tests_bin "
scp -P 2333 scripts/run_tests.sh $laputa_name ubuntu@localhost:~/laputa
scp -P 2333 $laputa_test_names ubuntu@localhost:~/laputa/tests_bin/
ssh -p 2333 ubuntu@localhost "cd laputa && mv $laputa_name_basename laputa"