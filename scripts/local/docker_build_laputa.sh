#!/bin/bash
docker run -it --rm -v $(pwd):/home/ubuntu/laputa -w /home/ubuntu/laputa --network host 1197744123/laputa:v2 ./scripts/local/copy_laputa_to_vm.sh
