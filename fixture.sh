#!/bin/sh
#set up for a debugging run.
# Since DHCP runs on port 67, this does sudo-level operations
setcap CAP_NET_BIND_SERVICE+eip target/debug/rustboot
virsh destroy client
virsh start client
tcpdump port 67 -i virbr0 -vvvv
