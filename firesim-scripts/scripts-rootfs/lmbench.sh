TIMES=${TIMES:-3}

cd home/ubuntu/lmbench-3.0-a9/bin/riscv64-linux-gnu/
n=0
while [ $n -lt $TIMES ]; do 
    dd if=/dev/zero of=test.txt bs=10M count=1
    ./bw_mem 8M fwr
    ./bw_unix -N 10
    ./bw_pipe
    ./lat_pagefault test.txt
    ./lat_select file
    ./lat_syscall null
    n=$(( n + 1 ))
done
