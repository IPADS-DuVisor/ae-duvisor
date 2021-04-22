#!/bin/bash
cd linux-laputa
./scripts-laputa/docker_build_linux.sh 
if [ $? -ne 0 ]; then
    exit -1
fi
cd -