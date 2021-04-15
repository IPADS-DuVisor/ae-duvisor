#!/bin/bash
docker run -it --rm -v $(pwd):/home/ubuntu/laputa -w /home/ubuntu/laputa --network host 1197744123/laputa:v4 ./testfiles/integration/test_images/build.sh ./testfiles/integration/test_images/build ./testfiles/integration/ $1
