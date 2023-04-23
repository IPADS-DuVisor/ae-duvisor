#!/bin/bash

ip link set dev eth0 address 00:12:6d:00:00:04

ip link set dev eth0 up

ip addr add 172.16.0.4/16 dev eth0

ip route add default via 172.16.0.2 dev eth0

ip link set lo up
