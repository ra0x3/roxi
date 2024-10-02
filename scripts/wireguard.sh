#!/bin/sh

export HOMEBREW_NO_AUTO_UPDATE=1

generate_client_ip() {
    public_key="$1"

    hash=$(echo -n "$public_key" | sha256sum | cut -c1-4)
    first_octet=$(printf "%d\n" 0x${hash:0:2})
    second_octet=$(printf "%d\n" 0x${hash:2:2})
    client_ip="10.0.$first_octet.$second_octet"
    echo "$client_ip"
}

install_prompt() {
    package_name="$1"
    echo "Do you want to install $package_name? [Y/n]: \c"
    read response
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

if ! command -v wg >/dev/null 2>&1; then
    if install_prompt "WireGuard"; then
        echo "Installing WireGuard via Homebrew..."
        brew install wireguard-tools
    else
        echo "WireGuard is required for this script. Exiting."
        exit 1
    fi
fi

echo "Generating WireGuard keys..."
PRIVATE_KEY=$(wg genkey)
PUBLIC_KEY=$(echo "$PRIVATE_KEY" | wg pubkey)

echo "Enter the VPN server public key:"
read SERVER_PUBLIC_KEY

echo "Enter the VPN server endpoint (e.g., 123.45.67.89:51820):"
read SERVER_ENDPOINT

CLIENT_IP=$(generate_client_ip "$PUBLIC_KEY")

WG_CONF_PATH="/etc/wireguard/wg0.conf"
echo "Creating wg0.conf at $WG_CONF_PATH..."

sudo mkdir -p /etc/wireguard

sudo tee "$WG_CONF_PATH" > /dev/null <<EOF
[Interface]
PrivateKey = $PRIVATE_KEY
Address = $CLIENT_IP/24
DNS = 1.1.1.1

[Peer]
PublicKey = $SERVER_PUBLIC_KEY
Endpoint = $SERVER_ENDPOINT
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
EOF

echo "wg0.conf created successfully at $WG_CONF_PATH"

echo "Your WireGuard public key is: $PUBLIC_KEY"
echo "Provide this public key to the VPN server administrator."

if [ "$(uname)" = "Darwin" ]; then
    echo "Detected MacOS, skipping iptables configuration."
else
    if install_prompt "Add iptables rules for forwarding traffic"; then
        sudo iptables -A FORWARD -i wg0 -j ACCEPT
        sudo iptables -A FORWARD -o wg0 -j ACCEPT
        sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
    fi
fi

if install_prompt "Enable and start WireGuard (wg-quick up wg0)"; then
    sudo wg-quick up wg0
fi

