#!/bin/bash

set -e

echo "Building Volta"

cargo build --release

echo "Packaging Binaries"

cd target/release
tar -zcvf "volta.tar.gz" volta volta-shim volta-migrate
