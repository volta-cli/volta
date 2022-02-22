#!/bin/bash

set -e

echo "Building Volta"

SDKROOT=$(xcrun -sdk macosx11.1 --show-sdk-path) MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release --target=$2-apple-darwin
cd "target/$2-apple-darwin/release"

echo "Packaging Binaries"

tar -zcvf "$1.tar.gz" volta volta-shim volta-migrate
