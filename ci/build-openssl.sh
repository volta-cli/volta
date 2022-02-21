#!/bin/bash

set -e

echo "Building OpenSSL"
cd openssl
./config shared --prefix=/app/src/openssl-dist
make
make install_sw
cd -
