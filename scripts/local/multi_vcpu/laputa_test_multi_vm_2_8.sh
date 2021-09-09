#!/bin/bash

# trap ctrl-c and call ctrl_c()
trap ctrl_c INT

function ctrl_c() {
	pkill screen
}

screen -S virt -d -m &
VIRT_PID=$!
echo "virt screen session pid is ${VIRT_PID}"
screen -S host-0 -d -m &
HOST_PID=$!
echo "host screen session pid is ${HOST_PID}"
screen -S host-1 -d -m &
HOST_PID=$!
echo "host screen session pid is ${HOST_PID}"
screen -S host-2 -d -m &
HOST_PID=$!
echo "host screen session pid is ${HOST_PID}"
screen -S host-3 -d -m &
HOST_PID=$!
echo "host screen session pid is ${HOST_PID}"
screen -S host-4 -d -m &
HOST_PID=$!
echo "host screen session pid is ${HOST_PID}"
screen -S host-5 -d -m &
HOST_PID=$!
echo "host screen session pid is ${HOST_PID}"
screen -S host-6 -d -m &
HOST_PID=$!
echo "host screen session pid is ${HOST_PID}"
screen -S host-7 -d -m &
HOST_PID=$!
echo "host screen session pid is ${HOST_PID}"
./scripts/expect_wrapper.sh ./scripts/local/multi_vcpu/laputa_test_multi_vm_2_8.exp
ret=$?
pkill screen
exit $ret
