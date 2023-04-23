#!/bin/bash
docker run -it --rm -v $(pwd):/laputa -w /laputa --privileged --network host 1197744123/laputa:v4 ./scripts/local/build_app.sh
