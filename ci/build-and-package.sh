#!/bin/bash

set -e

echo "Building Volta"

if [ -z "$2" ]; then 
    cargo build --release
    cd target/release
else 
    cargo build --release --target=$2
    cd target/$2/release
fi

echo "Packaging Binaries"

tar -zcvf "$1.tar.gz" volta volta-shim volta-migrate
