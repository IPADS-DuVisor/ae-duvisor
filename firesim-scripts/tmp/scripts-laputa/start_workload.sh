#!/bin/bash

firesim infrasetup
screen -S workload -d -m
./scripts-laputa/start_workload.exp
