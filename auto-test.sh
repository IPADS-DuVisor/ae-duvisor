#!/bin/bash

function wait_for_tests() {
    local loop_acc=0
    local limit=22

    while true; do
        echo "[$loop_acc] Waiting for 5min..."
        
        sleep 300
        
        ./firesim-scripts/pull_fsim_log.sh | grep laputa-0 | sort | tail -n 1 | \
            xargs -I {} cat server-log/{} | grep "ALL TEST DONE";

        if [ $? -eq 0 ]; then
            echo "ALL TEST DONE!\n"
            break
        fi
        
        if [ $loop_acc -ge $limit ]; then
            echo "TIMEOUT!\n"
            break
        fi
        
        loop_acc=$(($loop_acc + 1))
    done;
}

function test_kvm() {
    echo "./nightly-scripts/kvm-$1.sh"
    
    cp ./nightly-scripts/kvm-$1.sh ./mnt-firesim/kvm_linux.sh

    ./firesim-scripts/kvm.sh sync
}

function test_ulh() {
    echo "./nightly-scripts/laputa-$1.sh"
    
    cp ./nightly-scripts/laputa-$1.sh ./mnt-firesim/laputa_linux.sh

    ./firesim-scripts/laputa.sh sync
}

function reset_firesim() {
    ~/aws-scripts/west/reset.sh
}

# Sync existing logs
./firesim-scripts/pull_fsim_log.sh

# Start tests
for i in 1 2 4 8; do
    reset_firesim
    test_kvm $i
    wait_for_tests
    
    reset_firesim
    test_ulh $i
    wait_for_tests
done
