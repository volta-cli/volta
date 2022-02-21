#!/bin/bash

set -e

echo "Building Volta"

if [[ "$1" == *linux-musl* ]]; then
    if [[ "$1" == *aarch64* ]]; then
        cross build --release --target aarch64-unknown-linux-musl
        cd target/aarch64-unknown-linux-musl/release
    elif [[ "$1" == *x86_64* ]]; then
        cross build --release --target x86_64-unknown-linux-musl
        cd target/x86_64-unknown-linux-musl/release
    fi
else
    cargo build --release
    cd target/release
fi

echo "Packaging Binaries"

tar -zcvf "$1.tar.gz" volta volta-shim volta-migrate