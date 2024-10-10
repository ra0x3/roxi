#!/bin/sh

if [ "$(uname)" = "Darwin" ]; then
    echo "Detected macOS, skipping iptables refresh."
else
    echo "Detected Linux, refreshing iptables."

    sudo iptables -F
    sudo iptables -t nat -F
    sudo iptables -X
    sudo iptables -t nat -X
    sudo iptables -P INPUT ACCEPT
    sudo iptables -P FORWARD ACCEPT
    sudo iptables -P OUTPUT ACCEPT

    sudo iptables -L -v
    sudo iptables -t nat -L -v

fi
