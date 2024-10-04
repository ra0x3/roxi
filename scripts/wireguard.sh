#!/bin/sh

export HOMEBREW_NO_AUTO_UPDATE=1

INTERFACE="wg0"

usage() {
    echo "Usage: $0 [--server | --node]"
    echo "  --server  Set up as a server (generates its own public key)"
    echo "  --node    Set up as a client node (asks for server public key)"
    exit 1
}

generate_client_ip() {
    local public_key="$1"
    local hash=$(echo -n "$public_key" | sha256sum | cut -c1-4)
    local first_octet=$(printf "%d\n" "0x$(echo "$hash" | cut -c1-2)")
    local second_octet=$(printf "%d\n" "0x$(echo "$hash" | cut -c3-4)")
    local client_ip="10.0.$first_octet.$second_octet"
    echo "$client_ip"
}

install_prompt() {
    local package_name="$1"
    read -p "Do you want to install $package_name? [Y/n]: " response
    case "$response" in
        [yY][eE][sS]|[yY]|"")
            return 0
            ;;
        *)
            echo "$package_name installation skipped."
            return 1
            ;;
    esac
}

if [ $# -eq 0 ]; then
    usage
fi

if ! command -v wg >/dev/null 2>&1; then
    if install_prompt "WireGuard"; then
	echo "Installing WireGuard"
        if [ "$(uname)" = "Darwin" ]; then
            brew install wireguard-tools
        else
            sudo apt-get install -y wireguard-tools
        fi
    else
        exit 1
    fi
else
    echo "WireGuard already installed"
fi

case "$1" in
    --server)
        PRIVATE_KEY=$(wg genkey)
        PUBLIC_KEY=$(echo "$PRIVATE_KEY" | wg pubkey)
        ;;
    --node)
        PRIVATE_KEY=$(wg genkey)
        PUBLIC_KEY=$(echo "$PRIVATE_KEY" | wg pubkey)
        read SERVER_PUBLIC_KEY
        ;;
    *)
        usage
        ;;
esac

echo "Enter the VPN server endpoint (e.g., 123.45.67.89:51820):"
read SERVER_ENDPOINT

CLIENT_IP=$(generate_client_ip "$PUBLIC_KEY")

WG_CONF_PATH="/etc/wireguard/${INTERFACE}.conf"

sudo mkdir -p /etc/wireguard

sudo tee "$WG_CONF_PATH" > /dev/null <<EOF
[Interface]
PrivateKey = $PRIVATE_KEY
Address = $CLIENT_IP/24
DNS = 1.1.1.1

[Peer]
PublicKey = ${SERVER_PUBLIC_KEY:-$PUBLIC_KEY}
Endpoint = $SERVER_ENDPOINT
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
EOF

echo "Your WireGuard public key is: $PUBLIC_KEY"

if [ "$(uname)" = "Darwin" ]; then
    echo "Detected macOS, skipping iptables configuration."
else
    if install_prompt "Add iptables rules for forwarding traffic"; then
        sudo iptables -A FORWARD -i $INTERFACE -j ACCEPT
        sudo iptables -A FORWARD -o $INTERFACE -j ACCEPT
        sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
    fi
fi


if install_prompt "Enable and start WireGuard (wg-quick up ${INTERFACE})"; then
    echo "Bringing up WireGuard on interface ${INTERFACE}"
    sudo wg-quick up $INTERFACE && sudo wg show $INTERFACE
fi

