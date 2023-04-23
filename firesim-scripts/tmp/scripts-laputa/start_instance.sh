#!/bin/bash

INSTANCE=${INSTANCE:-"i-00a986bbe1728b67a"}

RUNNING=$(aws ec2 describe-instance-status --instance-id $INSTANCE | grep running)
echo starting instance... RUNNING: $RUNNING

echo WAITING.... \n\n\n\n

if [[ -z $RUNNING ]]; then
aws ec2 start-instances --instance-id i-00a986bbe1728b67a
fi


while [[ -z $RUNNING ]]; do

RUNNING=$(aws ec2 describe-instance-status --instance-id $INSTANCE | grep running)
sleep 1
done


echo instance started
