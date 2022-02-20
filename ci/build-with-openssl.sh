#!/bin/bash

set -e

OPENSSL_DIR=/app/src/openssl-dist ./ci/build-and-package.sh "$1"
