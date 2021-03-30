#!/bin/bash
docker run -it --rm -v $(pwd):/laputa -w /laputa --network host 1197744123/laputa:v1 ./scripts/local/laputa_test.exp
