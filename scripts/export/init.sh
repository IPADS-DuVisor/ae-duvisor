#!/bin/bash

IPADDR=192.168.254.5
WIDTH=16
GATEWAY=192.168.10.1

echo "Mounting Procfs..."

mount -t proc proc /proc
mount -t devtmpfs devtmpfs /dev
mount -t sysfs sysfs /sys

echo "Initializing Network..."

# Setup loopback
ip link set lo up

# Setup br0
ip link add br0 type bridge
ip link set eth0 master br0
#brctl addif br0 eth0
ip link set br0 up
ip link set eth0 up
ip addr add $IPADDR/$WIDTH dev br0 
ip route add default via $GATEWAY dev eth0

for i in {0..7}; do
    ip tuntap add vmtap${i} mode tap
    ip link set vmtap${i} master br0
    ip link set dev vmtap${i} up
done

## Add vmtap0-7
#tunctl -t vmtap0
#brctl addif br0 vmtap0
#ip link set dev vmtap0 up
#
#tunctl -t vmtap1
#brctl addif br0 vmtap1
#ip link set dev vmtap1 up
#
#tunctl -t vmtap2
#brctl addif br0 vmtap2
#ip link set dev vmtap2 up
#
#tunctl -t vmtap3
#brctl addif br0 vmtap3
#ip link set dev vmtap3 up
#
#tunctl -t vmtap4
#brctl addif br0 vmtap4
#ip link set dev vmtap4 up
#
#tunctl -t vmtap5
#brctl addif br0 vmtap5
#ip link set dev vmtap5 up
#
#tunctl -t vmtap6
#brctl addif br0 vmtap6
#ip link set dev vmtap6 up
#
#tunctl -t vmtap7
#brctl addif br0 vmtap7
#ip link set dev vmtap7 up

bash

