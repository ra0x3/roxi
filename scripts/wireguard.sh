#!/bin/sh

export HOMEBREW_NO_AUTO_UPDATE=1

INTERFACE="wg0"
PORT=51820
OVERWRITE=false

usage() {
    echo "Usage: $0 [--server | --node | --uninstall]"
    echo "  --server    Set up as a server (generates its own public key)"
    echo "  --node      Set up as a client node (asks for server public key)"
    echo "  --overwrite  Overwrite existing configuration"
    echo "  --uninstall Remove WireGuard and all configuration files"
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
    read -p "Do you want to install/add/enable: $package_name? [Y/n]: " response
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

uninstall_wireguard() {
    echo "Uninstalling WireGuard and removing configuration files."

    if [ "$(uname)" = "Darwin" ]; then
        brew uninstall wireguard-tools
    else
        sudo apt-get remove -y wireguard-tools
    fi

    echo "Removing WireGuard configuration files."
    sudo rm -rf /etc/wireguard
    echo "WireGuard uninstalled and configuration files removed."
    exit 0
}

if [ $# -eq 0 ]; then
    usage
fi

if [ "$1" = "--uninstall" ]; then
    uninstall_wireguard
    exit 0
fi

for arg in "$@"; do
    case $arg in
        --overwrite)
            OVERWRITE=true
            shift
            ;;
        --server|--node)
            MODE=$arg
            ;;
        *)
            usage
            ;;
    esac
done

WG_CONF_PATH="/etc/wireguard/${INTERFACE}.conf"

if ! command -v wg >/dev/null 2>&1; then
    if install_prompt "WireGuard"; then
        echo "Installing WireGuard"
        if [ "$(uname)" = "Darwin" ]; then
            brew install wireguard-tools
        else
            sudo apt-get install -y wireguard-tools
        fi
    else
        echo "WireGuard is required to proceed. Exiting."
        exit 1
    fi
else
    echo "WireGuard is already installed"
fi

if [ -f "$WG_CONF_PATH" ] && [ "$OVERWRITE" = false ]; then
    echo "Existing configuration found at $WG_CONF_PATH"
    if sudo wg show "$INTERFACE" >/dev/null 2>&1; then
        echo "Interface $INTERFACE is already up."
    else
        if install_prompt "WireGuard (wg-quick up ${INTERFACE})"; then
            echo "Bringing up WireGuard on interface ${INTERFACE}"
            sudo wg-quick up $INTERFACE && sudo wg show $INTERFACE
        fi
    fi
    exit 0
fi

if [ "$OVERWRITE" = true ] || [ ! -f "$WG_CONF_PATH" ]; then
    PRIVATE_KEY=$(wg genkey)
    PUBLIC_KEY=$(echo "$PRIVATE_KEY" | wg pubkey)
    CLIENT_IP=$(generate_client_ip "$PUBLIC_KEY")

    sudo tee "$WG_CONF_PATH" > /dev/null <<EOF
[Interface]
PrivateKey = $PRIVATE_KEY
Address = $CLIENT_IP/24
# DNS = 8.8.8.8
EOF

    echo "WireGuard configuration created at $WG_CONF_PATH"
    echo "Your WireGuard public key is: $PUBLIC_KEY"

    if [ "$(uname)" = "Darwin" ]; then
        echo "Detected macOS, skipping iptables configuration."
    else
        if [ "$MODE" = "--server" ] && install_prompt "iptables rules for forwarding traffic"; then
            sudo iptables -A FORWARD -i $INTERFACE -j ACCEPT
            sudo iptables -A FORWARD -o $INTERFACE -j ACCEPT
            sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
            echo "iptables rules added for forwarding traffic."
        fi
    fi

    if install_prompt "WireGuard (wg-quick up ${INTERFACE})"; then
        echo "Bringing up WireGuard on interface ${INTERFACE}"
        sudo wg-quick up $INTERFACE && sudo wg show $INTERFACE
    fi
fi
