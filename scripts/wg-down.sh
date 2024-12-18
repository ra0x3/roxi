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
        WG_QUICK="/opt/homebrew/bin/bash /opt/homebrew/bin/wg-quick"
    else
        CONFIG_DIR="/etc/wireguard"
        WG_QUICK="wg-quick"
    fi
    CONFIG_FILE="$ROXI_DIR/$INTERFACE.conf"
    PRIVATE_KEY_PATH="$CONFIG_DIR/privatekey"
    PUBLIC_KEY_PATH="$CONFIG_DIR/publickey"
}

check_required_files() {
    if [ ! -f "$CONFIG_FILE" ] || [ ! -f "$PRIVATE_KEY_PATH" ] || [ ! -f "$PUBLIC_KEY_PATH" ]; then
        echo "${RED}Error: Required files not found. Ensure $CONFIG_FILE in $ROXI_DIR, and $PRIVATE_KEY_PATH and $PUBLIC_KEY_PATH in $CONFIG_DIR.${NC}"
        exit 1
    fi
}

bring_down_interface() {
    echo "${GREEN}Bringing down WireGuard interface '$INTERFACE'...${NC}"
    sudo $WG_QUICK down "$INTERFACE"
    if [ $? -eq 0 ]; then
        echo "${GREEN}Successfully brought down interface '$INTERFACE'.${NC}"
    else
        echo "${RED}Error: Failed to bring down interface '$INTERFACE'.${NC}"
        exit 1
    fi
}

main() {
    validate_args "$@"
    set_interface "$1"
    configure_paths
    check_required_files
    bring_down_interface
}

main "$@"
