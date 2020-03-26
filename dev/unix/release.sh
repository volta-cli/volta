#!/usr/bin/env bash

# Script to build the binaries and package them up for release.
# This should be run from the top-level directory.

# get the directory of this script
# (from https://stackoverflow.com/a/246128)
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# get shared functions from the volta-install.sh file
source "$DIR/volta-install.sh"

usage() {
  cat >&2 <<END_OF_USAGE
release.sh

Compile and package a release for Volta

USAGE:
    ./dev/unix/release.sh [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints this help info

OPTIONS:
        --release       Build artifacts in release mode, with optimizations (default)
        --dev           Build artifacts in dev mode, without optimizations
END_OF_USAGE
}


# default to compiling with '--release'
build_with_release="true"

# parse input arguments
case "$1" in
  -h|--help)
    usage
    exit 0
    ;;
  --dev)
    build_with_release="false"
    ;;
  ''|--release)
    # not really necessary to set this again
    build_with_release="true"
    ;;
  *)
    error "Unknown argument '$1'"
    usage
    exit1
    ;;
esac

# read the current version from Cargo.toml
cargo_toml_contents="$(<Cargo.toml)"
VOLTA_VERSION="$(parse_cargo_version "$cargo_toml_contents")" || exit 1

# figure out the OS details
os="$(uname -s)"
openssl_version="$(openssl version)" || exit 1
VOLTA_OS="$(parse_os_info "$os" "$openssl_version")"
if [ "$?" != 0 ]; then
  error "Releases for '$os' are not yet supported."
  request "To support '$os', add another case to parse_os_info() in volta-install.sh."
  exit 1
fi

release_filename="volta-$VOLTA_VERSION-$VOLTA_OS"

# first make sure the release binaries have been built
info 'Building' "Volta for $(bold "$release_filename")"
if [ "$build_with_release" == "true" ]
then
  target_dir="target/release"
  cargo build --release
else
  target_dir="target/debug"
  cargo build
fi || exit 1

# then package the binaries and shell scripts together
info 'Packaging' "the compiled binaries"
cd "$target_dir"
# using COPYFILE_DISABLE to avoid storing extended attribute files when run on OSX
# (see https://superuser.com/q/61185)
COPYFILE_DISABLE=1 tar -czvf "$release_filename.tar.gz" volta volta-shim volta-migrate

info 'Completed' "release in file $target_dir/$release_filename.tar.gz"
