#!/usr/bin/env bash
#
# Copyright 2020 Brian Smith.
#
# Permission to use, copy, modify, and/or distribute this software for any
# purpose with or without fee is hereby granted, provided that the above
# copyright notice and this permission notice appear in all copies.
#
# THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
# WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
# MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
# SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
# WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
# OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
# CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

set -eux -o pipefail
IFS=$'\n\t'

rustflags_self_contained="-Clink-self-contained=yes -Clinker=rust-lld"
qemu_aarch64="qemu-aarch64 -L /usr/aarch64-linux-gnu"

for arg in $*; do
  case $arg in
    --target=*)
      target=${arg#*=}
      ;;
    *)
      ;;
  esac
done

# See comments in install-build-tools.sh.
llvm_version=14

case $target in
  aarch64-unknown-linux-gnu)
    export CC_aarch64_unknown_linux_gnu=clang-$llvm_version
    export AR_aarch64_unknown_linux_gnu=llvm-ar-$llvm_version
    export CFLAGS_aarch64_unknown_linux_gnu="--sysroot=/usr/aarch64-linux-gnu"
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER="$qemu_aarch64"
    ;;
  aarch64-unknown-linux-musl)
    export CC_aarch64_unknown_linux_musl=clang-$llvm_version
    export AR_aarch64_unknown_linux_musl=llvm-ar-$llvm_version
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="$rustflags_self_contained"
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUNNER="$qemu_aarch64"
    ;;
  x86_64-unknown-linux-musl)
    export CC_x86_64_unknown_linux_musl=clang-$llvm_version
    export AR_x86_64_unknown_linux_musl=llvm-ar-$llvm_version
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="$rustflags_self_contained"
    ;;
  *)
    ;;
esac

cargo "$@"
