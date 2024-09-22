#!/bin/sh

export HOMEBREW_NO_AUTO_UPDATE=1

install_mac() {
    echo "Detected macOS"

    if ! command -v brew &> /dev/null; then
        echo "Homebrew not found. Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    echo "Checking for Rust installation..."
    if command -v rustc &> /dev/null; then
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
    brew install boringtun
}

install_linux() {
    echo "Detected Linux"

    echo "Updating package list..."
    sudo apt-get update

    echo "Installing package dependencies..."
    sudo apt-get install -y libssl-dev pkg-config git curl

    echo "Checking for Rust installation..."
    if command -v rustc &> /dev/null; then
        echo "Rust is installed. Setting to version 1.78..."
        rustup install 1.78.0
        rustup default 1.78.0
    else
        echo "Rust not found. Installing Rust 1.78..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.78.0 -y
        source $HOME/.cargo/env
    fi

    echo "Installing boringtun 0.5..."
    git clone https://github.com/cloudflare/boringtun.git
    cd boringtun
    cargo build --release
    sudo cp target/release/boringtun /usr/local/bin/
    cd ..
    rm -rf boringtun
}

if [[ "$OSTYPE" == "darwin"* ]]; then
    install_mac
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    install_linux
else
    echo "Unsupported platform: $OSTYPE"
    exit 1
fi

echo "All dependencies installed successfully!"
