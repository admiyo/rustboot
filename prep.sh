#!/bin/sh
#set up for a debugging run.
# Since DHCP runs on port 67, this does sudo-level operations
sudo setcap CAP_NET_BIND_SERVICE+eip target/debug/rustboot
RUST_BACKTRACE=1  ./target/debug/rustboot -v -w
