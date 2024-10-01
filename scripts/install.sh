#!/bin/sh

OS=$(uname)

install_mac() {
    echo "Detected macOS"

    export HOMEBREW_NO_AUTO_UPDATE=1

    if ! command -v brew > /dev/null 2>&1; then
        echo "Homebrew not found. Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    echo "Checking for Rust installation..."
    if command -v rustc > /dev/null 2>&1; then
        echo "Rust is installed. Setting to version 1.78..."
        rustup install 1.78.0
        rustup default 1.78.0
    else
        echo "Rust not found. Installing Rust 1.78..."
        brew install rustup-init
        rustup toolchain install 1.78.0
        rustup default 1.78.0
    fi

    echo "Installing boringtun 0.5..."
    brew install wireguard-tools
}

install_linux() {
    echo "Detected Linux"

    echo "Updating package list..."
    sudo apt-get update

    echo "Installing package dependencies..."
    sudo apt-get install -y libssl-dev pkg-config git curl

    echo "Checking for Rust installation..."
    if command -v rustc > /dev/null 2>&1; then
        echo "Rust is installed. Setting to version 1.78..."
        rustup install 1.78.0
        rustup default 1.78.0
    else
        echo "Rust not found. Installing Rust 1.78..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.78.0 -y
        . $HOME/.cargo/env
    fi

    echo "Installing WireGuard"
    sudo apt-get install -y wireguard

}

if [ "$OS" = "Darwin" ]; then
    install_mac
elif [ "$OS" = "Linux" ]; then
    install_linux
else
    echo "Unsupported platform: $OS"
    exit 1
fi

echo "All dependencies installed successfully!"

