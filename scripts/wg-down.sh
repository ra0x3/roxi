#!/bin/sh

GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
NC="\033[0m" # No Color

usage() {
    echo "${YELLOW}Usage: $0 [interface]${NC}"
    echo "  interface (optional): Name of the WireGuard interface (default: wg0)."
}

if [ "$#" -gt 1 ]; then
    echo "${RED}Error: Invalid number of arguments.${NC}"
    usage
    exit 1
fi

INTERFACE=${1:-wg0}

if [ "$(uname)" = "Darwin" ]; then
    CONFIG_DIR="/opt/homebrew/etc/wireguard"
else
    CONFIG_DIR="/etc/wireguard"
fi

CONFIG_FILE="$CONFIG_DIR/$INTERFACE.conf"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "${RED}Error: Config file '$CONFIG_FILE' does not exist.${NC}"
    exit 1
fi

echo "${GREEN}Bringing down WireGuard interface '$INTERFACE'...${NC}"

sudo wg-quick down "$INTERFACE"
if [ $? -eq 0 ]; then
    echo "${GREEN}Successfully brought down interface '$INTERFACE'.${NC}"
else
    echo "${RED}Error: Failed to bring down interface '$INTERFACE'.${NC}"
    exit 1
fi
