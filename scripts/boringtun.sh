#!/bin/sh

PORT=51820

usage() {
    echo "Usage: sudo $0 --up | --down | --reload --interface <interface> --address <address> --allowed-ips <allowed_ips>"
    exit 1
}

setup_nat_linux() {
    echo "Setting up NAT and IP forwarding for Linux..."
    sudo sysctl -w net.ipv4.ip_forward=1
    sudo iptables -A FORWARD -i "$interface" -o eth0 -j ACCEPT
    sudo iptables -A FORWARD -i eth0 -o "$interface" -j ACCEPT
    sudo iptables -t nat -A POSTROUTING -s "$address" -o eth0 -j MASQUERADE
    echo "NAT setup complete for Linux."
}

setup_nat_macos() {
    echo "Setting up NAT and IP forwarding for macOS..."
    sudo sysctl -w net.inet.ip.forwarding=1

    sudo cp /etc/pf.conf /etc/pf.conf.backup

    if ! grep -q "nat on en0 from $address to any -> (en0)" /etc/pf.conf; then
        echo "Adding NAT rule to /etc/pf.conf..."
        sudo awk '/# NAT rules go here/ { print "nat on en0 from '"$address"' to any -> (en0)"; }1' /etc/pf.conf > /tmp/pf.conf
        sudo mv /tmp/pf.conf /etc/pf.conf
    else
        echo "NAT rule already exists in /etc/pf.conf"
    fi

    sudo pfctl -f /etc/pf.conf
    sudo pfctl -e
    echo "NAT setup complete for macOS."
}

wg_up() {
    sudo boringtun "$interface"
    sudo wg set "$interface" private-key /etc/wireguard/privatekey listen-port $PORT peer "$allowed_ips"
    sudo ip addr add "$address" dev "$interface"
    sudo ip link set "$interface" up

    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        setup_nat_linux
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        setup_nat_macos
    else
        echo "Unsupported platform: $OSTYPE"
        exit 1
    fi
    echo "WireGuard interface $interface is up."
}

wg_down() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        sudo ip link set "$interface" down
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        sudo ifconfig "$interface" down
    else
        echo "Unsupported platform: $OSTYPE"
        exit 1
    fi
    sudo pkill boringtun
    echo "WireGuard interface $interface is down."
}

wg_reload() {
    wg_down
    wg_up
    echo "WireGuard interface $interface reloaded."
}

action=""
address=""
interface=""
allowed_ips=""

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --up) action="up" ;;
        --down) action="down" ;;
        --reload) action="reload" ;;
        --address) address="$2"; shift ;;
        --interface) interface="$2"; shift ;;
        --allowed-ips) allowed_ips="$2"; shift ;;
        *) echo "Unknown option: $1"; usage ;;
    esac
    shift
done

if [[ -z "$action" || -z "$interface" || -z "$address" || -z "$allowed_ips" ]]; then
    usage
fi

case $action in
    up) wg_up ;;
    down) wg_down ;;
    reload) wg_reload ;;
    *) usage ;;
esac
