#!/bin/bash

first_arg=$1

if [ -z "$first_arg" ]; then
    build_version="release"
elif test ${first_arg} = debug; then
    build_version=""
fi

sudo docker run -it \
    --rm \
    -v $(pwd):/home/ubuntu/laputa \
    -w /home/ubuntu/laputa \
    --network host \
    --privileged=true \
    1197744123/laputa:v4 \
    ./scripts/local/copy_laputa_to_vm.sh $build_version
