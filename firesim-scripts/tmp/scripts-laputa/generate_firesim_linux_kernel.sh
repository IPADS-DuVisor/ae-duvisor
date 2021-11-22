#!/bin/bash
LINUX_COMMIT=${LINUX_COMMIT:-}
OPENSBI_COMMIT=${OPENSBI_COMMIT:-}

# update linux
cd boards/firechip/linux-laputa
git pull
git checkout $LINUX_COMMIT
cp ../.config .
cd -

# update opensbi
cd boards/default/firmware/opensbi
git pull
git checkout $OPENSBI_COMMIT
cd -

RISCV=/home/ldj/firesim/chipyard/riscv-tools-install ./marshal build br-base.json

