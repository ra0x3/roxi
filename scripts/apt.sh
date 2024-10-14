#!/bin/sh

sudo apt-get update -y && sudo apt-get install -y \
    build-essential \
    curl \
    git \
    jq \
    libffi-dev \
    libreadline-dev \
    libsqlite3-dev \
    libssl-dev \
    llvm \
    make \
    tree \
    unzip \
    wget

curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install
rm -f awscliv2.zip

/usr/local/bin/aws --version
