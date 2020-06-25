#!/bin/bash

set -eux

apt install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
        wget \
        libssl-dev \
        pkg-config \
        build-essential

wget "https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init"
chmod +x rustup-init

./rustup-init -y --no-modify-path --default-toolchain stable
chmod -R a+w $RUSTUP_HOME $CARGO_HOME

rustup --version
cargo --version
rustc --version
