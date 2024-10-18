#!/bin/sh

RUST_VERSION=1.81.0
OS=$(uname)

YELLOW='\033[1;33m'
GREEN='\033[1;32m'
RED='\033[1;31m'
NC='\033[0m' # No Color

install_mac() {
    echo -e "${GREEN}Detected macOS${NC}"

    export HOMEBREW_NO_AUTO_UPDATE=1

    if ! command -v brew > /dev/null 2>&1; then
        echo -e "${YELLOW}Homebrew not found. Installing Homebrew...${NC}"
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    echo -e "${GREEN}Checking for Rust installation...${NC}"
    if command -v rustc > /dev/null 2>&1; then
        echo -e "${GREEN}Rust is installed. Setting to version ${RUST_VERSION}...${NC}"
        rustup install $RUST_VERSION
        rustup default $RUST_VERSION
    else
        echo -e "${YELLOW}Rust not found. Installing Rust ${RUST_VERSION}...${NC}"
        brew install rustup-init
        rustup toolchain install $RUST_VERSION
        rustup default $RUST_VERSION
    fi

    echo -e "${GREEN}Installing WireGuard tools...${NC}"
    brew install wireguard-tools
}

install_linux() {
    echo -e "${GREEN}Detected Linux${NC}"

    echo -e "${GREEN}Updating package list...${NC}"
    sudo apt-get update

    echo -e "${GREEN}Installing package dependencies...${NC}"
    sudo apt-get install -y libssl-dev pkg-config git curl

    echo -e "${GREEN}Checking for Rust installation...${NC}"
    if command -v rustc > /dev/null 2>&1; then
        echo -e "${GREEN}Rust is installed. Setting to version ${RUST_VERSION}...${NC}"
        rustup install $RUST_VERSION
        rustup default $RUST_VERSION
    else
        echo -e "${YELLOW}Rust not found. Installing Rust ${RUST_VERSION}...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain $RUST_VERSION -y
        . $HOME/.cargo/env
    fi

    echo -e "${GREEN}Installing WireGuard...${NC}"
    sudo apt-get install -y wireguard
}

if [ "$OS" = "Darwin" ]; then
    install_mac
elif [ "$OS" = "Linux" ]; then
    install_linux
else
    echo -e "${RED}Unsupported platform: $OS${NC}"
    exit 1
fi

echo -e "${GREEN}All dependencies installed successfully!${NC}"