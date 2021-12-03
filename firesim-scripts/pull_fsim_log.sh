#!/bin/bash

IP_STRING=`aws ec2 describe-instances --instance-ids i-00a986bbe1728b67a --no-cli-pager | grep PublicIp | awk '{print $2}' | paste -d " " - - | cut -d '"' -f 2`
IP=`echo $IP_STRING | cut -d " " -f 1`
echo IP is $IP

rsync -P -avz -e 'ssh -o "ProxyCommand nc -X 5 -x 192.168.10.1:7890 %h %p" -o StrictHostKeyChecking=no -i ~/aws-scripts/west/firesim.pem' centos@${IP}:~/sim_slot_0/laputa-0-* ./server-log

rsync -P -avz -e 'ssh -o "ProxyCommand nc -X 5 -x 192.168.10.1:7890 %h %p" -o StrictHostKeyChecking=no -i ~/aws-scripts/west/firesim.pem' centos@${IP}:~/sim_slot_1/laputa-1-* ./client-log
