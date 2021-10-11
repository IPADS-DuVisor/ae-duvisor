#!/bin/bash

INSTANCE=${INSTANCE:-"i-042f4d39babb9e026"}
STOP=""
STOP=$(aws ec2 stop-instances --instance-id $INSTANCE | grep stop)
while [ -z $STOP ]; do
    STOP=$(aws ec2 stop-instances --instance-id $INSTANCE | grep stop)
sleep 1
done
