#!/bin/bash

set -e

echo "Building Volta"

MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release --target=aarch64-apple-darwin
MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release --target=x86_64-apple-darwin

echo "Packaging Binaries"

mkdir -p target/universal-apple-darwin/release

for exe in volta volta-shim volta-migrate
do
    lipo -create -output target/universal-apple-darwin/release/$exe target/x86_64-apple-darwin/release/$exe target/aarch64-apple-darwin/release/$exe
done

cd target/universal-apple-darwin/release

tar -zcvf "$1.tar.gz" volta volta-shim volta-migrate
