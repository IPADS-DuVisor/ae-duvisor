#!/bin/zsh

RED="\e[31m"
GREEN="\e[32m"
YELLOW="\e[33m"
ENDCOLOR="\e[0m"

SEQ_FILE=/tmp/seq
CONT=${1:-""}

function rename_log() {
    ls ./server-log | grep "^laputa-0" | sort | tail -n 1 | \
        xargs -I {} cp server-log/{} server-log/"$1-$2".{};
    
    ls ./client-log | grep "^laputa-1" | sort | tail -n 1 | \
        xargs -I {} cp client-log/{} client-log/"$1-$2".{};
}

function wait_for_tests() {
    local loop_acc=0
    #local limit=24
    local limit=8

    while true; do
        echo -ne "\r ${YELLOW} Time spent: $(($loop_acc * 5)) min. ${ENDCOLOR}"
        
        sleep 300
        
        ./firesim-scripts/pull_fsim_log.sh | grep "^laputa-0" | sort | tail -n 1 | \
            xargs -I {} cat server-log/{} | grep "ALL TEST DONE";

        if [ $? -eq 0 ]; then
            echo "\n ${GREEN} ALL TEST DONE! ${ENDCOLOR}\n"
            break
        fi
        
        if [ $loop_acc -ge $limit ]; then
            echo "\n ${RED} TIMEOUT! ${ENDCOLOR}\n"
            break
        fi
        
        loop_acc=$(($loop_acc + 1))
    done;
}

function test_kvm() {
    echo "${YELLOW} ./nightly-scripts/kvm-$1.sh ${ENDCOLOR}"
    
    cp ./nightly-scripts/kvm-$1.sh ./mnt-firesim/kvm_linux.sh

    ./firesim-scripts/kvm.sh sync
}

function test_ulh() {
    echo "${YELLOW} ./nightly-scripts/laputa-$1.sh ${ENDCOLOR}"
    
    cp ./nightly-scripts/laputa-$1.sh ./mnt-firesim/laputa_linux.sh

    ./firesim-scripts/laputa.sh sync
}

function reset_firesim() {
    ~/aws-scripts/west/reset.sh
}

if [ "$CONT" = "cont" ]; then
    wait_for_tests
else
    cp ./nightly-scripts/seq-tmpl $SEQ_FILE
    vim $SEQ_FILE
    
    # Sync existing logs
    ./firesim-scripts/pull_fsim_log.sh
fi

# Start tests
while true; do
    if [ -s $SEQ_FILE ]; then
        head -n 1 $SEQ_FILE | IFS=" " read ARCH VCPU 
        sed -i '1d' $SEQ_FILE
    else
        echo
        break
    fi
    
    reset_firesim
    date
    if [ "$ARCH" = "kvm" ]; then
        test_kvm $VCPU
    elif [ "$ARCH" = "ulh" ]; then
        test_ulh $VCPU
    else
        continue
    fi
    wait_for_tests
    rename_log $ARCH $VCPU
done
