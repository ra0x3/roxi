#!/bin/sh

sudo apt-get update && \
    sudo apt-get install -y gcc-aarch64-linux-gnu \
    binutils-aarch64-linux-gnu \
    qemu-user \
    libssl-dev \
    pkg-config
