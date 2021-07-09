#!/bin/bash
sudo docker run -it \
    --rm \
    -v $(pwd):/home/ubuntu/laputa \
    -v /home/ldj/mnt/ulh/firecracker-dev:/home/ubuntu/firecracker-dev \
    -w /home/ubuntu/laputa \
    --network host \
    --privileged=true \
    1197744123/laputa:v4 \
    ./scripts/local/copy_laputa_to_vm.sh
