#!/bin/bash

./scripts/local/docker_build_laputa_firesim.sh
cd test-files-laputa
cp ../tests/integration/virq_utimer_test.img .
dd if=virq_utimer_test.img of=virq_test2.img bs=4096 skip=1 count=1
cd ..
./scripts/local/docker_build_laputa_firesim.sh
./firesim-scripts/laputa.sh sync
