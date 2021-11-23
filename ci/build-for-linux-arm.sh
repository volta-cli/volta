#!/bin/bash

set -e

echo "Building Volta"

cargo build --release --target=aarch64-unknown-linux-gnu

echo "Packaging Binaries"

cd target/release
tar -zcvf "$1.tar.gz" volta volta-shim volta-migrate
