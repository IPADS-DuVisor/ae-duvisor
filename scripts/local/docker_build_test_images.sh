#!/bin/bash
docker run -it --rm -v $(pwd):/home/ubuntu/laputa -w /home/ubuntu/laputa --network host 1197744123/laputa:v4 ./unitestfiles/test_images/build.sh ./unitestfiles/test_images/build ./unitestfiles/ $1
