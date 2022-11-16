#!/bin/bash

cd src/devices/src/kvmtool-port/
make
cd ../../../..
./scripts/local/docker_build_laputa_firesim.sh
./firesim-scripts/laputa.sh sync
