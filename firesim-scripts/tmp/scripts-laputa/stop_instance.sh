#!/bin/bash

INSTANCE=${INSTANCE:-"i-00a986bbe1728b67a"}
STOP=""
STOP=$(aws ec2 stop-instances --instance-id $INSTANCE | grep stop)
while [ -z $STOP ]; do
    STOP=$(aws ec2 stop-instances --instance-id $INSTANCE | grep stop)
sleep 1
done
