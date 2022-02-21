#!/bin/bash

set -e

echo "Checking out OpenSSL $1"
git clone --branch OpenSSL_"$1"-stable https://github.com/openssl/openssl

echo "Building OpenSSL"
cd openssl
./config shared --prefix=/app/src/openssl-dist
make
make install_sw
cd -
