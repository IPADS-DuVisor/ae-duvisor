#!/bin/bash

# Remove log
sudo rm -f ./exec_log

# Killall screen
# sudo killall screen

# Execute expect
echo $1
$1 | tee exec_log | tr "\r" "\n"

# Check Return value
if [ ${PIPESTATUS[0]} -eq 0 ]; then
        cat ./exec_log | tr "\r" "\n"
else
        cat ./exec_log | tr "\r" "\n"
        exit -1
fi
