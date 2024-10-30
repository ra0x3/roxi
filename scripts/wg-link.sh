#!/bin/sh

GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
NC="\033[0m"

CONFIG_DIR="/etc/wireguard"
DESTINATION="/etc/wireguard"
INTERFACE="wg0"
[ "$(uname)" = "Darwin" ] && CONFIG_DIR="/opt/homebrew/etc/wireguard" && DESTINATION="/opt/homebrew/etc/wireguard"

usage() {
    echo "${YELLOW}Usage: $0 [--interface INTERFACE]${NC}"
    echo "  --interface INTERFACE (optional): Name of the WireGuard interface (default: wg0)."
}

parse_args() {
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --interface)
                if [ -n "$2" ]; then
                    INTERFACE="$2"
                    shift
                else
                    echo "${RED}Error: --interface flag requires an argument.${NC}"
                    usage
                    exit 1
                fi
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                echo "${RED}Error: Invalid argument '$1'.${NC}"
                usage
                exit 1
                ;;
        esac
        shift
    done
}

set_paths() {
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
    parse_args "$@"
    set_paths
    check_dir
    check_source_file
    create_symlink
}

main "$@"
