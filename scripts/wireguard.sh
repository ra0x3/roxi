#!/bin/sh

export HOMEBREW_NO_AUTO_UPDATE=1

INTERFACE="wg0"
PORT=51820
OVERWRITE=false

YELLOW='\033[1;33m'
GREEN='\033[1;32m'
RED='\033[1;31m'
NC='\033[0m' # No Color

usage() {
    echo -e "${YELLOW}Usage: $0 [--overwrite | --uninstall | --interface <name>]${NC}"
    echo -e "${YELLOW}  --overwrite  Overwrite existing configuration${NC}"
    echo -e "${YELLOW}  --uninstall  Remove WireGuard and all configuration files${NC}"
    echo -e "${YELLOW}  --interface <name>  Specify the interface name (default: wg0)${NC}"
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
            echo -e "${YELLOW}$package_name installation skipped.${NC}"
            return 1
            ;;
    esac
}

uninstall_wireguard() {
    echo -e "${GREEN}Uninstalling WireGuard and removing configuration files.${NC}"

    if [ "$(uname)" = "Darwin" ]; then
        brew uninstall wireguard-tools
        sudo rm -rf /opt/homebrew/etc/wireguard
    else
        sudo apt-get remove -y wireguard-tools
        sudo rm -rf /etc/wireguard
    fi

    echo -e "${GREEN}WireGuard uninstalled and configuration files removed.${NC}"
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
                echo -e "${RED}Error: --interface requires an argument.${NC}"
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
        echo -e "${GREEN}Installing WireGuard${NC}"
        if [ "$(uname)" = "Darwin" ]; then
            brew install wireguard-tools && brew install bash
        else
            sudo apt-get install -y wireguard-tools
        fi
    else
        echo -e "${RED}WireGuard is required to proceed. Exiting.${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}WireGuard is already installed${NC}"
fi

if [ -f "$WG_CONF_PATH" ] && [ "$OVERWRITE" = false ]; then
    echo -e "${GREEN}Existing configuration found at $WG_CONF_PATH${NC}"
    if sudo wg show "$INTERFACE" >/dev/null 2>&1; then
        echo -e "${GREEN}Interface $INTERFACE is already up.${NC}"
    else
        if install_prompt "WireGuard (wg-quick up ${INTERFACE})"; then
            echo -e "${GREEN}Bringing up WireGuard on interface ${INTERFACE}${NC}"
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

    echo -e "${GREEN}WireGuard configuration created at $WG_CONF_PATH${NC}"
    echo -e "${GREEN}Your WireGuard public key is: $PUBLIC_KEY${NC}"

    if [ "$(uname)" = "Darwin" ]; then
        echo -e "${YELLOW}Detected macOS, skipping iptables configuration.${NC}"
    else
        if install_prompt "iptables rules for forwarding traffic"; then
            sudo iptables -A FORWARD -i $INTERFACE -j ACCEPT
            sudo iptables -A FORWARD -o $INTERFACE -j ACCEPT
            sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
            echo -e "${GREEN}iptables rules added for forwarding traffic.${NC}"
        fi
    fi

    if install_prompt "WireGuard (wg-quick up ${INTERFACE})"; then
        echo -e "${GREEN}Bringing up WireGuard on interface ${INTERFACE}${NC}"
        sudo $WG_QUICK up $INTERFACE && sudo wg show $INTERFACE
    fi
fi