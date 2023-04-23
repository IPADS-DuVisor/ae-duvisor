#!/bin/bash
mkdir -p raw

# start firesim
RET=`aws ec2 describe-instances --instance-ids i-05d4ae3c817ac285a --no-cli-pager | grep running`
if [[ -z $RET ]]; then
    RET=`aws ec2 start-instances --instance-ids i-05d4ae3c817ac285a | grep running`
fi
while [[ -z $RET ]]; do
    RET=`aws ec2 describe-instances --instance-ids i-05d4ae3c817ac285a --no-cli-pager | grep running`
    sleep 1
done
echo "Firesim manager started!!!!"

# get ip addr
IP_STRING=`aws ec2 describe-instances --instance-ids i-05d4ae3c817ac285a --no-cli-pager | grep PublicIp | awk '{print $2}' | paste -d " " - - | cut -d '"' -f 2`
IP=`echo $IP_STRING | cut -d " " -f 1`
echo IP is $IP

if [ $1 == "sync" ]; then
    tar -czf mnt-firesim.tar.gz mnt-firesim
    rsync -P -avz -e 'ssh -o "ProxyCommand nc -X 5 -x 192.168.10.1:7890 %h %p" -o StrictHostKeyChecking=no -i ~/aws-scripts/west/firesim.pem' -r br-base-bin-laputa mnt-firesim.tar.gz firesim-scripts centos@${IP}:~/firesim
fi
# running laputa
echo running laputa benchmark
ssh -f -o "ProxyCommand nc -X 5 -x 192.168.10.1:7890 %h %p" -o StrictHostKeyChecking=no -i ~/aws-scripts/west/firesim.pem centos@${IP} "cd firesim/firesim-scripts && nohup ./scripts-laputa/start.sh laputa > ~/laputa.log 2>&1 &"

# running kvm
#echo running kvm benchmark
#ssh -o StrictHostKeyChecking=no -i ~/Downloads/firesim.pem centos@${IP} "cd firesim/firesim-scripts && ./scripts-laputa/start.sh kvm"

## sync remote results
#rsync -avz -e "ssh -o StrictHostKeyChecking=no -i ~/Downloads/firesim.pem" -r centos@${IP}:~/firesim/log-laputa/ raw/ 

# stop firesim
#aws ec2 stop-instances --instance-ids i-05d4ae3c817ac285a --no-cli-pager
#RET=`aws ec2 describe-instances --instance-ids i-05d4ae3c817ac285a --no-cli-pager | grep stopped`
#while [[ -z $RET ]]; do
#    RET=`aws ec2 describe-instances --instance-ids i-05d4ae3c817ac285a --no-cli-pager | grep stopped`
#    sleep 1
#done
#echo "Firesim manager stopped!!!!"
#
# start firesim to change ip addr
#RET=`aws ec2 describe-instances --instance-ids i-05d4ae3c817ac285a --no-cli-pager | grep running`
#if [[ -z $RET ]]; then
#    RET=`aws ec2 start-instances --instance-ids i-05d4ae3c817ac285a | grep running`
#fi
#while [[ -z $RET ]]; do
#    RET=`aws ec2 describe-instances --instance-ids i-05d4ae3c817ac285a --no-cli-pager | grep running`
#    sleep 1
#done
#echo "Firesim manager started!!!!"
#
## get new ip addr
#IP_STRING=`aws ec2 describe-instances --instance-ids i-05d4ae3c817ac285a --no-cli-pager | grep PublicIp | awk '{print $2}' | paste -d " " - - | cut -d '"' -f 2`
#echo IP addr is $IP_STRING
#IP=`echo $IP_STRING | cut -d " " -f 1`
#echo IP is $IP
#
## sync remote firesim files to CI
#rsync -avz -e "ssh -o StrictHostKeyChecking=no -i ~/Downloads/firesim.pem" -r centos@${IP}:~/firesim/log-laputa/ raw/ 
#rsync -avz -e "ssh -o StrictHostKeyChecking=no -i ~/Downloads/firesim.pem" -r centos@${IP}:~/firesim/firesim-scripts/log-laputa/ raw/ 
#
## stop firesim
#aws ec2 stop-instances --instance-ids i-05d4ae3c817ac285a --no-cli-pager
