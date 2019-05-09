#!/usr/bin/env bash

# Script to build the binaries and package them up for release.
# This should be run from the top-level directory.

# get shared functions from the volta-install.sh file
# TODO: do this as a relative path
source dev/unix/volta-install.sh

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

# parse the 'version = "X.Y.Z"' line from the input Cargo.toml contents
# and return the version string
parse_cargo_version() {
  local contents="$1"

  while read -r line
  do
    if [[ "$line" =~ ^version\ =\ \"(.*)\" ]]
    then
      echo "${BASH_REMATCH[1]}"
      return 0
    fi
  done <<< "$contents"

  error "Could not determine the current version"
  return 1
}

# return if sourced (for testing the functions above without running the commands below)
return 0 2>/dev/null


# exit on error
set -e

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
VOLTA_VERSION="$(parse_cargo_version "$cargo_toml_contents")"

# figure out the OS details
os="$(uname -s)"
openssl_version="$(openssl version)"
VOLTA_OS="$(parse_os_info "$os" "$openssl_version")"

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
fi

# then package the binaries and shell scripts together
shell_script_dir="shell/unix"
info 'Packaging' "the compiled binaries and shell scripts"
# copy the load.* shell scripts to the target dir, to include them as well
cp "$shell_script_dir"/load.* "$target_dir/"
cd "$target_dir"
# using COPYFILE_DISABLE to avoid storing extended attribute files when run on OSX
# (see https://superuser.com/q/61185)
COPYFILE_DISABLE=1 tar -czvf "$release_filename.tar.gz" volta shim load.*

info 'Completed' "release in file $(bold "$target_dir/$release_filename.tar.gz")"
