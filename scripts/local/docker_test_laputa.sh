#!/bin/bash
echo $1
docker run -it --rm -v $(pwd):/laputa -w /laputa --network host 1197744123/laputa:v4 ./scripts/local/laputa_test.exp $1
