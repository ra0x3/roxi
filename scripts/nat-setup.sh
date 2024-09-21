#!/bin/bash

usage() {
    echo "Usage: $0 /path/to/wg0.conf"
    exit 1
}

get_wireguard_ip() {
    local wg_conf="$1"

    if [[ ! -f "$wg_conf" ]]; then
        echo "Error: $wg_conf not found!"
        usage
    fi

    local wg_ip=$(grep '^Address' "$wg_conf" | awk -F ' = ' '{print $2}')

    if [[ -z "$wg_ip" ]]; then
        echo "Error: Could not find 'Address' in $wg_conf."
        usage
    fi

    echo "$wg_ip"
}

setup_linux() {
    local wg_ip="$1"
    echo "Setting up NAT and IP forwarding for Linux with IP range: $wg_ip..."

    sudo sysctl -w net.ipv4.ip_forward=1

    sudo iptables -A FORWARD -i wg0 -o eth0 -j ACCEPT
    sudo iptables -A FORWARD -i eth0 -o wg0 -j ACCEPT
    sudo iptables -t nat -A POSTROUTING -s "$wg_ip" -o eth0 -j MASQUERADE

    echo "NAT and IP forwarding setup complete for Linux."
}

setup_macos() {
    local wg_ip="$1"
    echo "Setting up NAT and IP forwarding for macOS with IP range: $wg_ip..."

    sudo sysctl -w net.inet.ip.forwarding=1

    sudo cp /etc/pf.conf /etc/pf.conf.backup

    if ! grep -q "nat on en0 from $wg_ip to any -> (en0)" /etc/pf.conf; then
        echo "Adding NAT rule to /etc/pf.conf..."

        sudo awk '/# NAT rules go here/ { print "nat on en0 from '"$wg_ip"' to any -> (en0)"; }1' /etc/pf.conf > /tmp/pf.conf
        sudo mv /tmp/pf.conf /etc/pf.conf
    else
        echo "NAT rule already exists in /etc/pf.conf"
    fi

    sudo pfctl -f /etc/pf.conf
    sudo pfctl -e

    echo "NAT and IP forwarding setup complete for macOS."
}

if [[ $# -ne 1 ]]; then
    usage
fi

wg_conf_path="$1"
wg_ip=$(get_wireguard_ip "$wg_conf_path")

echo "Extracted WireGuard IP: $wg_ip"

OS="$(uname -s)"
case "${OS}" in
    Linux*)     setup_linux "$wg_ip" ;;
    Darwin*)    setup_macos "$wg_ip" ;;
    *)          echo "Unsupported operating system: ${OS}" ;;
esac
