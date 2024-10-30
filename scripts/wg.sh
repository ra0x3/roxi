#!/bin/sh

export HOMEBREW_NO_AUTO_UPDATE=1

INTERFACE="wg0"
PORT=51820
OVERWRITE=false
ROXI_DIR="$HOME/.config/roxi"

YELLOW='\033[1;33m'
GREEN='\033[1;32m'
RED='\033[1;31m'
NC='\033[0m'

usage() {
    echo  "${YELLOW}Usage: $0 [--overwrite | --uninstall | --interface <name>]${NC}"
    echo  "${YELLOW}  --overwrite  Overwrite existing configuration${NC}"
    echo  "${YELLOW}  --uninstall  Remove WireGuard and all configuration files${NC}"
    echo  "${YELLOW}  --interface <name>  Specify the interface name (default: wg0)${NC}"
    exit 1
}

generate_client_ip() {
    local public_key="$1"
    local hash=$(echo -n "$public_key" | sha256sum | cut -c1-4)
    local first_octet=$(printf "%d\n" "0x$(echo "$hash" | cut -c1-2)")
    local second_octet=$(printf "%d\n" "0x$(echo "$hash" | cut -c3-4)")
    echo "10.0.$first_octet.$second_octet"
}

install_prompt() {
    local package_name="$1"
    read -p "Do you want to install/add/enable: $package_name? [Y/n]: " response
    case "$response" in
        [yY][eE][sS]|[yY]|"") return 0 ;;
        *) echo  "${YELLOW}$package_name installation skipped.${NC}" ;;
    esac
    return 1
}

uninstall_wireguard() {
    echo  "${GREEN}Uninstalling WireGuard and removing configuration files.${NC}"
    if [ "$(uname)" = "Darwin" ]; then
        brew uninstall wireguard-tools
    else
        sudo apt-get remove -y wireguard-tools
    fi
    rm -rf "$ROXI_DIR"
    echo  "${GREEN}WireGuard uninstalled and configuration files removed.${NC}"
    exit 0
}

configure_paths() {
    WG_CONFIG_FILE="$ROXI_DIR/${INTERFACE}.conf"

    if [ "$(uname)" = "Darwin" ]; then
        CONFIG_DIR="/opt/homebrew/etc/wireguard"
        WG_QUICK="/opt/homebrew/bin/bash /opt/homebrew/bin/wg-quick"
    else
        CONFIG_DIR="/etc/wireguard"
        WG_QUICK="wg-quick"
    fi

    PRIVATE_KEY_PATH="$CONFIG_DIR/privatekey"
    PUBLIC_KEY_PATH="$CONFIG_DIR/publickey"
}

install_wireguard() {
    if ! command -v wg >/dev/null 2>&1; then
        if install_prompt "WireGuard"; then
            echo  "${GREEN}Installing WireGuard${NC}"
            if [ "$(uname)" = "Darwin" ]; then
                brew install wireguard-tools && brew install bash
            else
                sudo apt-get install -y wireguard-tools
            fi
        else
            echo  "${RED}WireGuard is required to proceed. Exiting.${NC}"
            exit 1
        fi
    else
        echo  "${GREEN}WireGuard is already installed${NC}"
    fi
}

check_existing_config() {
    if [ -f "$WG_CONFIG_FILE" ] && [ "$OVERWRITE" = false ]; then
        echo  "${GREEN}Existing configuration found at $WG_CONFIG_FILE${NC}"
        if sudo wg show "$INTERFACE" >/dev/null 2>&1; then
            echo  "${GREEN}Interface $INTERFACE is already up.${NC}"
        else
            if install_prompt "WireGuard (wg-quick up ${INTERFACE})"; then
                echo  "${GREEN}Bringing up WireGuard on interface ${INTERFACE}${NC}"
                sudo $WG_QUICK up $INTERFACE && sudo wg show $INTERFACE
            fi
        fi
        exit 0
    fi
}

generate_keys() {
    sudo mkdir -p "$(dirname "$PRIVATE_KEY_PATH")"

    if [ "$OVERWRITE" = true ] || [ ! -f "$PRIVATE_KEY_PATH" ] || [ ! -f "$PUBLIC_KEY_PATH" ]; then
        sudo wg genkey | tee "$PRIVATE_KEY_PATH" | wg pubkey | tee "$PUBLIC_KEY_PATH" > /dev/null
        PUBLIC_KEY=$(sudo cat "$PUBLIC_KEY_PATH")
        echo  "${GREEN}Keys generated and saved:${NC}"
        echo  "${GREEN}Private Key: $PRIVATE_KEY_PATH${NC}"
        echo  "${GREEN}Public Key: $PUBLIC_KEY_PATH${NC}"
    else
        echo  "${GREEN}Using existing keys:${NC}"
        echo  "${GREEN}Private Key: $PRIVATE_KEY_PATH${NC}"
        echo  "${GREEN}Public Key: $PUBLIC_KEY_PATH${NC}"
    fi
}

create_config_file() {
    CLIENT_IP=$(generate_client_ip "$PUBLIC_KEY")
    sudo tee "$WG_CONFIG_FILE" > /dev/null <<EOF
[Interface]
PrivateKey = $PRIVATE_KEY
Address = $CLIENT_IP/24
ListenPort = $PORT
# PostUp = iptables -A FORWARD -i $INTERFACE -j ACCEPT; iptables -A FORWARD -o $INTERFACE -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
# PostDown = iptables -D FORWARD -i $INTERFACE -j ACCEPT; iptables -D FORWARD -o $INTERFACE -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE
EOF
    echo  "${GREEN}WireGuard configuration created at $WG_CONFIG_FILE${NC}"
    echo  "${GREEN}Your WireGuard public key is: $PUBLIC_KEY${NC}"
}

update_firewall() {
    if [ "$(uname)" != "Darwin" ]; then
        if install_prompt "iptables rules for forwarding traffic"; then
            sudo iptables -A FORWARD -i $INTERFACE -j ACCEPT
            sudo iptables -A FORWARD -o $INTERFACE -j ACCEPT
            sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
            echo  "${GREEN}iptables rules added for forwarding traffic.${NC}"
        fi
    else
        echo  "${YELLOW}Detected macOS, skipping iptables configuration.${NC}"
    fi
}

start_wireguard() {
    if install_prompt "WireGuard (wg-quick up ${INTERFACE})"; then
        echo  "${GREEN}Bringing up WireGuard on interface ${INTERFACE}${NC}"
        sudo sh scripts/wg-link.sh "$WG_CONFIG_FILE" "$INTERFACE"
        sudo $WG_QUICK up $INTERFACE && sudo wg show $INTERFACE
    fi
}

check_dir() {
    if [ -d "$ROXI_DIR" ]; then
        echo  "${GREEN}Directory $ROXI_DIR already exists.${NC}"
    else
        mkdir -p "$ROXI_DIR"
        echo  "${YELLOW}Directory $ROXI_DIR created.${NC}"
    fi
}

if [ "$1" = "--uninstall" ]; then
    uninstall_wireguard
fi

while [ $# -gt 0 ]; do
    case $1 in
        --overwrite) OVERWRITE=true; shift ;;
        --uninstall) uninstall_wireguard ;;
        --interface)
            if [ -z "$2" ]; then
                echo  "${RED}Error: --interface requires an argument.${NC}"
                usage
            fi
            INTERFACE="$2"; shift 2 ;;
        *) usage ;;
    esac
done

configure_paths
check_dir
install_wireguard
check_existing_config
generate_keys
create_config_file
update_firewall
start_wireguard
