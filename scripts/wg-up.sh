#!/bin/sh

GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
NC="\033[0m"

ROXI_DIR="$HOME/.config/roxi"

usage() {
    echo "${YELLOW}Usage: $0 [interface]${NC}"
    echo "  interface (optional): Name of the WireGuard interface (default: wg0)."
}

validate_args() {
    if [ "$#" -gt 1 ]; then
        echo "${RED}Error: Invalid number of arguments.${NC}"
        usage
        exit 1
    fi
}

set_interface() {
    INTERFACE=${1:-wg0}
}

configure_paths() {
    if [ "$(uname)" = "Darwin" ]; then
        CONFIG_DIR="/opt/homebrew/etc/wireguard"
    else
        CONFIG_DIR="/etc/wireguard"
    fi
    CONFIG_FILE="$ROXI_DIR/$INTERFACE.conf"
    PRIVATE_KEY_PATH="$CONFIG_DIR/privatekey"
    PUBLIC_KEY_PATH="$CONFIG_DIR/publickey"
}

check_config_file() {
    if [ ! -f "$CONFIG_FILE" ]; then
        echo "${RED}Error: Config file '$CONFIG_FILE' does not exist in $ROXI_DIR.${NC}"
        exit 1
    fi
}

check_keys() {
    if [ ! -f "$PRIVATE_KEY_PATH" ] || [ ! -f "$PUBLIC_KEY_PATH" ]; then
        echo "${RED}Error: Private or public key file not found in $CONFIG_DIR.${NC}"
        exit 1
    fi
}

bring_up_interface() {
    echo "${GREEN}Bringing up WireGuard interface '$INTERFACE'...${NC}"
    sudo wg-quick up "$CONFIG_FILE"

    if [ $? -eq 0 ]; then
        echo "${GREEN}Successfully brought up interface '$INTERFACE'.${NC}"
    else
        echo "${RED}Error: Failed to bring up interface '$INTERFACE'.${NC}"
        exit 1
    fi
}

main() {
    validate_args "$@"
    set_interface "$1"
    configure_paths
    check_config_file
    check_keys
    bring_up_interface
}

main "$@"
