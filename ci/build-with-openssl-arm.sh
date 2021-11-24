#!/bin/bash

set -e

echo "Building OpenSSL"
cd openssl
./config linux-aarch64 shared --prefix=/root/workspace/openssl-arm-dist 
make
make install_sw
cd -

OPENSSL_DIR=/root/workspace/openssl-arm-dist ./ci/build-for-linux-arm.sh "$1"
