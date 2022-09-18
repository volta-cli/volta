#!/bin/bash

set -e

echo "Building Volta"

SDKROOT=$(xcrun -sdk macosx11.1 --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release --target=aarch64-apple-darwin

echo "Packaging Binaries"

cd target/aarch64-apple-darwin/release
tar -zcvf "$1.tar.gz" volta volta-shim volta-migrate
