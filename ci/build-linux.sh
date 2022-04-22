#!/bin/bash

set -e

# Activate the upgraded versions of GCC and binutils
source /opt/rh/devtoolset-2/enable

echo "Building Volta"

cargo build --release

echo "Packaging Binaries"

cd target/release
tar -zcvf "$1.tar.gz" volta volta-shim volta-migrate
