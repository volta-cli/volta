#!/bin/bash

set -e

echo "Building Volta"

cargo rustc --print cfg
#cargo build --release

echo "Packaging Binaries"

cd target/release
tar -zcvf "$1.tar.gz" volta volta-shim volta-migrate
