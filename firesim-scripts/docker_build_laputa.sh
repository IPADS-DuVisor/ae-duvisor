#!/bin/bash

first_arg=$1

if [ ${USER}1 == gitlab-runner1 ]; then
    # for CI environment
    IT=""
    EXTRA_V=" -v /home/gitlab-runner:/home/gitlab-runner "
else
    IT=" -it "
fi

if [ -z "$first_arg" ]; then
    build_level="release"
    PREPARE="-e PREPARE=${HOME}/laputa/prepare "
elif test ${first_arg} = debug; then
    build_level=""
    PREPARE="-e PREPARE=${HOME}/prepare "
else
    echo "Wrong arg."
    exit
fi

sudo docker run  ${IT} \
    ${PREPARE} \
    -e HOST_HOSTNAME=`hostname` \
    --rm \
    -v $(pwd):/home/ubuntu/laputa \
    ${EXTRA_V} 	\
    -w /home/ubuntu/laputa \
    --network host \
    --privileged=true \
    1197744123/laputa:v4 \
    ./scripts/firesim-scripts/copy_laputa_to_vm.sh
