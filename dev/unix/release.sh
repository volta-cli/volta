#!/usr/bin/env bash

# Script to build the binaries and package them up for release.
# This should be run from the top-level directory.

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

volta_info() {
  local ACTION="$1"
  local DETAILS="$2"
  command printf '\033[1;32m%12s\033[0m %s\n' "${ACTION}" "${DETAILS}" 1>&2
}

volta_error() {
  command printf '\033[1;31mError\033[0m: %s\n' "$1" 1>&2
}

bold() {
  command printf '\033[1m%s\033[0m' "$1"
}

# parse the 'version = "0.3.0"' line from the input text
parse_version() {
  local contents="$1"

  while read -r line
  do
    if [[ "$line" =~ ^version\ =\ \"(.*)\" ]]
    then
      echo "${BASH_REMATCH[1]}"
      return 0
    fi
  done <<< "$contents"

  volta_error "Could not determine the current version"
  return 1
}

# returns the os name to be used in the packaged release,
# including the openssl info if necessary
parse_os_info() {
  local uname_str="$1"
  local openssl_version="$2"

  case "$uname_str" in
    Linux)
      parsed_version="$(parse_openssl_version "$openssl_version")"
      # if there was an error, return
      exit_code="$?"
      if [ "$exit_code" != 0 ]
      then
        return "$exit_code"
      fi

      echo "linux-openssl-$parsed_version"
      ;;
    Darwin)
      echo "macos"
      ;;
    *)
      volta_error "Releases for '$uname_str' are not yet supported. You will need to add another OS case to this script, and to the install script to support this OS."
      return 1
  esac
  return 0
}

# return true(0) if the element is contained in the input arguments
# called like:
#  if element_in "foo" "${array[@]}"; then ...
element_in() {
  local match="$1";
  shift

  local element;
  # loop over the input arguments and return when a match is found
  for element in "$@"
  do
    [ "$element" == "$match" ] && return 0
  done
  return 1
}

# parse the OpenSSL version from the input text
parse_openssl_version() {
  local version_str="$1"

  # array containing the SSL libraries that are supported
  # would be nice to use a bash 4.x associative array, but bash 3.x is the default on OSX
  SUPPORTED_SSL_LIBS=( 'OpenSSL' )

  # use regex to get the library name and version
  # typical version string looks like 'OpenSSL 1.0.1e-fips 11 Feb 2013'
  if [[ "$version_str" =~ ^([^\ ]*)\ ([0-9]+\.[0-9]+\.[0-9]+) ]]
  then
    # check that the lib is supported
    libname="${BASH_REMATCH[1]}"
    if element_in "$libname" "${SUPPORTED_SSL_LIBS[@]}"
    then
      # lib is supported, return the version
      echo "${BASH_REMATCH[2]}"
      return 0
    fi
    volta_error "Releases for '$libname' not currently supported. Supported libraries are: ${SUPPORTED_SSL_LIBS[@]}."
    return 1
  else
    volta_error "Could not determine OpenSSL version for '$version_str'. You probably need to update the regex to handle this output."
    return 1
  fi
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
    volta_error "Unknown argument '$1'"
    usage
    exit1
    ;;
esac

# read the current version from Cargo.toml
cargo_toml_contents="$(<Cargo.toml)"
NOTION_VERSION="$(parse_version "$cargo_toml_contents")"

# figure out the OS details
os="$(uname -s)"
openssl_version="$(openssl version)"
NOTION_OS="$(parse_os_info "$os" "$openssl_version")"

release_filename="volta-$NOTION_VERSION-$NOTION_OS"

# first make sure the release binaries have been built
volta_info 'Building' "Volta for $(bold "$release_filename")"
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
volta_info 'Packaging' "the compiled binaries and shell scripts"
# copy the load.* shell scripts to the target dir, to include them as well
cp "$shell_script_dir"/load.* "$target_dir/"
cd "$target_dir"
# using COPYFILE_DISABLE to avoid storing extended attribute files when run on OSX
# (see https://superuser.com/q/61185)
COPYFILE_DISABLE=1 tar -czvf "$release_filename.tar.gz" volta shim load.*

volta_info 'Completed' "release in file $(bold "$target_dir/$release_filename.tar.gz")"
