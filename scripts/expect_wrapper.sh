#!/bin/sh

# Remove log
sudo rm -f ./exec_log

# Killall screen
# sudo killall screen

# Execute expect
echo $1
$1 > exec_log

# Check Return value
if [ $? -eq 0 ]; then
        cat ./exec_log | tr "\r" "\n"
else
        cat ./exec_log | tr "\r" "\n"
        exit -1
fi
