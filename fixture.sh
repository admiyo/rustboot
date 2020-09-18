#!/bin/sh
#set up for a debugging run.
# Since DHCP runs on port 67, this does sudo-level operations
sudo setcap CAP_NET_BIND_SERVICE+eip target/debug/rustboot
sudo virsh destroy client
sudo virsh start client
sudo tcpdump port 67 -i virbr0 -vvvv
