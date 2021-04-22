#!/bin/bash
if [ ! -d "./prepare" ]; then 
    mkdir -p prepare
    # password: ipads123
    scp -r gitlab-runner@liuyuxuan-pc:~/prepare .
fi

git submodule update --init --recursive