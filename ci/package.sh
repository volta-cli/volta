#!/bin/bash

set -e

echo "Packaging Binaries"

tar -zcvf "$2.tar.gz" $1/volta $1/volta-shim $1/volta-migrate
