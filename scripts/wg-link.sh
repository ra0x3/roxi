#!/bin/sh

GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
NC="\033[0m"

CONFIG_DIR="/etc/wireguard"  # Default to Linux path
[ "$(uname)" = "Darwin" ] && CONFIG_DIR="/opt/homebrew/etc/wireguard"

usage() {
    echo "${YELLOW}Usage: $0 <destination> [interface]${NC}"
    echo "  destination: Path where the symlink should be created."
    echo "  interface (optional): Name of the WireGuard interface (default: wg0)."
}

validate_args() {
    if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
        echo "${RED}Error: Invalid number of arguments.${NC}"
        usage
        exit 1
    fi
}

set_paths() {
    DESTINATION=$1
    INTERFACE=${2:-wg0}
    SOURCE_FILE="$CONFIG_DIR/$INTERFACE.conf"
}

check_dir() {
    if [ ! -d "$CONFIG_DIR" ]; then
        echo "${RED}Error: Configuration directory '$CONFIG_DIR' does not exist.${NC}"
        exit 1
    else
        echo "${GREEN}Configuration directory '$CONFIG_DIR' exists.${NC}"
    fi
}

check_source_file() {
    if [ ! -f "$SOURCE_FILE" ]; then
        echo "${RED}Error: Source file '$SOURCE_FILE' does not exist.${NC}"
        exit 1
    else
        echo "${GREEN}Source file '$SOURCE_FILE' exists.${NC}"
    fi
}

create_symlink() {
    if [ -e "$DESTINATION" ]; then
        echo "${YELLOW}Warning: Destination '$DESTINATION' already exists. It will be overwritten.${NC}"
    fi

    ln -sf "$SOURCE_FILE" "$DESTINATION"
    if [ $? -eq 0 ]; then
        echo "${GREEN}Symlink created from '$SOURCE_FILE' to '$DESTINATION'.${NC}"
    else
        echo "${RED}Error: Failed to create symlink.${NC}"
        exit 1
    fi
}

main() {
    validate_args "$@"
    set_paths "$@"
    check_dir
    check_source_file
    create_symlink
}

main "$@"
