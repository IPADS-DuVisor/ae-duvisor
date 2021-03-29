#!/bin/bash
docker run -it --rm -v $(pwd):/home/ubuntu/laputa -w /home/ubuntu/laputa/linux-laputa --network host 1197744123/laputa:v1 ../scripts/local/build_linux.sh
