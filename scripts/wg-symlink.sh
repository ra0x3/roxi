#!/bin/sh

GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
NC="\033[0m"

usage() {
    echo "${YELLOW}Usage: $0 <source> <destination> [interface]${NC}"
    echo "  source: Path to the local WireGuard config file."
    echo "  destination: Path where the symlink should be created."
    echo "  interface (optional): Name of the WireGuard interface (default: wg0)."
}

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    echo "${RED}Error: Invalid number of arguments.${NC}"
    usage
    exit 1
fi

SOURCE=$1
DESTINATION=$2
INTERFACE=${3:-wg0}

if [ "$(uname)" = "Darwin" ]; then
    DESTINATION="/opt/homebrew/etc/wireguard/$INTERFACE.conf"
fi

if [ ! -f "$SOURCE" ]; then
    echo "${RED}Error: Source file '$SOURCE' does not exist.${NC}"
    exit 1
fi

if [ -e "$DESTINATION" ]; then
    echo "${YELLOW}Warning: Destination '$DESTINATION' already exists. It will be overwritten.${NC}"
fi

ln -sf "$SOURCE" "$DESTINATION"
if [ $? -eq 0 ]; then
    echo "${GREEN}Symlink created from '$SOURCE' to '$DESTINATION'.${NC}"
else
    echo "${RED}Error: Failed to create symlink.${NC}"
    exit 1
fi
