#!/bin/bash
echo $@
docker run -it \
    --rm \
    -v $(pwd):/laputa \
    -w /laputa \
    --device /dev/net/tun:/dev/net/tun \
    --cap-add=NET_ADMIN \
    --network host \
    1197744123/laputa:v4 \
    ./scripts/local/laputa_test.sh $@
