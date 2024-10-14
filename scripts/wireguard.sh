#!/bin/sh

export HOMEBREW_NO_AUTO_UPDATE=1

INTERFACE="wg0"
PORT=51820
OVERWRITE=false

usage() {
    echo "Usage: $0 [--overwrite | --uninstall | --interface <name>]"
    echo "  --overwrite  Overwrite existing configuration"
    echo "  --uninstall  Remove WireGuard and all configuration files"
    echo "  --interface <name>  Specify the interface name (default: wg0)"
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
        sudo rm -rf /opt/homebrew/etc/wireguard
    else
        sudo apt-get remove -y wireguard-tools
        sudo rm -rf /etc/wireguard
    fi

    echo "WireGuard uninstalled and configuration files removed."
    exit 0
}

if [ "$1" = "--uninstall" ]; then
    uninstall_wireguard
    exit 0
fi

while [ $# -gt 0 ]; do
    case $1 in
        --overwrite)
            OVERWRITE=true
            shift
            ;;
        --uninstall)
            uninstall_wireguard
            exit 0
            ;;
        --interface)
            if [ -z "$2" ]; then
                echo "Error: --interface requires an argument."
                usage
            fi
            INTERFACE="$2"
            shift 2
            ;;
        *)
            usage
            ;;
    esac
done

if [ "$(uname)" = "Darwin" ]; then
    WG_CONF_PATH="/opt/homebrew/etc/wireguard/${INTERFACE}.conf"
    WG_QUICK="/opt/homebrew/bin/bash /opt/homebrew/bin/wg-quick"
else
    WG_CONF_PATH="/etc/wireguard/${INTERFACE}.conf"
    WG_QUICK="wg-quick"
fi

if ! command -v wg >/dev/null 2>&1; then
    if install_prompt "WireGuard"; then
        echo "Installing WireGuard"
        if [ "$(uname)" = "Darwin" ]; then
            brew install wireguard-tools && brew install bash
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
            sudo $WG_QUICK up $INTERFACE && sudo wg show $INTERFACE
        fi
    fi
    exit 0
fi

if [ "$OVERWRITE" = true ] || [ ! -f "$WG_CONF_PATH" ]; then
    PRIVATE_KEY=$(wg genkey)
    PUBLIC_KEY=$(echo "$PRIVATE_KEY" | wg pubkey)
    CLIENT_IP=$(generate_client_ip "$PUBLIC_KEY")

    sudo mkdir -p "$(dirname "$WG_CONF_PATH")"
    sudo tee "$WG_CONF_PATH" > /dev/null <<EOF
[Interface]
# The private key of the WireGuard server (keep this secret)
PrivateKey = "$PRIVATE_KEY"

# The IP address and subnet of the WireGuard interface on the server
Address = "$CLIENT_IP/24"

# The UDP port on which WireGuard will listen
ListenPort = 51820

# Optional: Firewall settings to allow traffic
# PostUp and PostDown run commands after the interface is brought up or down
# PostUp = iptables -A FORWARD -i wg0 -j ACCEPT; iptables -A FORWARD -o wg0 -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
# PostDown = iptables -D FORWARD -i wg0 -j ACCEPT; iptables -D FORWARD -o wg0 -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE

EOF

    echo "WireGuard configuration created at $WG_CONF_PATH"
    echo "Your WireGuard public key is: $PUBLIC_KEY"

    if [ "$(uname)" = "Darwin" ]; then
        echo "Detected macOS, skipping iptables configuration."
    else
        if install_prompt "iptables rules for forwarding traffic"; then
            sudo iptables -A FORWARD -i $INTERFACE -j ACCEPT
            sudo iptables -A FORWARD -o $INTERFACE -j ACCEPT
            sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
            echo "iptables rules added for forwarding traffic."
        fi
    fi

    if install_prompt "WireGuard (wg-quick up ${INTERFACE})"; then
        echo "Bringing up WireGuard on interface ${INTERFACE}"
        sudo $WG_QUICK up $INTERFACE && sudo wg show $INTERFACE
    fi
fi
