#!/bin/bash

install_prompt() {
    local package_name="$1"
    read -p "Do you want to install $package_name? [Y/n]: " response
    case "$response" in
        [yY][eE][sS]|[yY]|"")
            return 0
            ;;
        *)
            echo "$package_name installation skipped."
            return 1  # User chose no
            ;;
    esac
}

if ! command -v wg &> /dev/null; then
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

CLIENT_IP="10.0.0.$((RANDOM % 255))"

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

# Provide the client public key for the server to configure
echo "Your WireGuard public key is: $PUBLIC_KEY"
echo "Provide this public key to the VPN server administrator."

# Ask to enable and start the WireGuard interface
if install_prompt "Enable and start WireGuard (wg-quick up wg0)"; then
    sudo wg-quick up wg0
fi
