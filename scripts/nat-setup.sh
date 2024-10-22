#!/bin/sh

YELLOW='\033[1;33m'
GREEN='\033[1;32m'
RED='\033[1;31m'
NC='\033[0m' # No Color

usage() {
    echo -e "${YELLOW}Usage: $0 /path/to/wg0.conf${NC}"
    exit 1
}

get_wireguard_ip() {
    local wg_conf="$1"

    if [[ ! -f "$wg_conf" ]]; then
        echo -e "${RED}Error: $wg_conf not found!${NC}"
        usage
    fi

    local wg_ip=$(grep '^Address' "$wg_conf" | awk -F ' = ' '{print $2}')

    if [[ -z "$wg_ip" ]]; then
        echo -e "${RED}Error: Could not find 'Address' in $wg_conf.${NC}"
        usage
    fi

    echo "$wg_ip"
}

setup_linux() {
    local wg_ip="$1"
    echo -e "${GREEN}Setting up NAT and IP forwarding for Linux with IP range: $wg_ip...${NC}"

    sudo sysctl -w net.ipv4.ip_forward=1

    sudo iptables -A FORWARD -i wg0 -o eth0 -j ACCEPT
    sudo iptables -A FORWARD -i eth0 -o wg0 -j ACCEPT
    sudo iptables -t nat -A POSTROUTING -s "$wg_ip" -o eth0 -j MASQUERADE

    echo -e "${GREEN}NAT and IP forwarding setup complete for Linux.${NC}"
}

setup_macos() {
    local wg_ip="$1"
    echo -e "${GREEN}Setting up NAT and IP forwarding for macOS with IP range: $wg_ip...${NC}"

    sudo sysctl -w net.inet.ip.forwarding=1

    sudo cp /etc/pf.conf /etc/pf.conf.backup

    if ! grep -q "nat on en0 from $wg_ip to any -> (en0)" /etc/pf.conf; then
        echo -e "${YELLOW}Adding NAT rule to /etc/pf.conf...${NC}"

        sudo awk '/# NAT rules go here/ { print "nat on en0 from '"$wg_ip"' to any -> (en0)"; }1' /etc/pf.conf > /tmp/pf.conf
        sudo mv /tmp/pf.conf /etc/pf.conf
    else
        echo -e "${GREEN}NAT rule already exists in /etc/pf.conf${NC}"
    fi

    sudo pfctl -f /etc/pf.conf
    sudo pfctl -e

    echo -e "${GREEN}NAT and IP forwarding setup complete for macOS.${NC}"
}

if [[ $# -ne 1 ]]; then
    usage
fi

wg_conf_path="$1"
wg_ip=$(get_wireguard_ip "$wg_conf_path")

echo -e "${GREEN}Extracted WireGuard IP: $wg_ip${NC}"

OS="$(uname -s)"
case "${OS}" in
    Linux*)     setup_linux "$wg_ip" ;;
    Darwin*)    setup_macos "$wg_ip" ;;
    *)          echo -e "${RED}Unsupported operating system: ${OS}${NC}" ;;
esac