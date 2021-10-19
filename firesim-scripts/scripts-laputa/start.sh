#!/bin/bash
set -e

FPGA_IP=192.168.3.5

trap error ERR
function error {
    pkill screen
}

trap ctrl_c INT
function ctrl_c {
    pkill screen
}

if [[ -z $1 || $1 != "kvm" && $1 != laputa ]]; then
    echo please choose kvm or laputa
    exit -1
fi


cd ~/firesim
source sourceme-f1-manager.sh
cd -
./scripts-laputa/start_instance.sh


while ! ssh $FPGA_IP "./switch_to_${1}.sh"
do
        echo "Trying again..."
        sleep 2
done

scp -r ~/firesim/firesim-scripts/scripts-rootfs $FPGA_IP:~/
scp ~/firesim/mnt-firesim.tar.gz $FPGA_IP:~/
ssh $FPGA_IP "sudo tar -vxzf mnt-firesim.tar.gz"
scp ~/firesim/br-base-bin $FPGA_IP:~/

ssh $FPGA_IP "./scripts-rootfs/copy_myself.sh"

ssh $FPGA_IP "./switch_to_${1}.sh"

./scripts-laputa/start_workload.sh

mkdir -p ~/firesim/log-laputa
mkdir -p ~/firesim/firesim-scripts/log-laputa
LOG_NAME="~/firesim/log-laputa/`date +%Y-%m-%d-%T`"
LOG_NAME1="./log-laputa/`date +%Y-%m-%d-%T`"
./scripts-laputa/1_core.expect $1 | tee $LOG_NAME1 | tee $LOG_NAME

./scripts-laputa/stop_workload.exp
./scripts-laputa/stop_instance.sh

pkill screen