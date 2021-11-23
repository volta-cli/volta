#!/bin/bash

set -e

echo "Building OpenSSL"
cd openssl
./config shared --prefix=/root/workspace/openssl-dist
make
make install_sw
cd -

OPENSSL_DIR=/root/workspace/openssl-dist ./ci/build-for-linux-arm.sh "$1"
