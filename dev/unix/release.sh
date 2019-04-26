#!/usr/bin/env bash
# Script to build the binaries and package them up for release.
# This should be run from the top-level directory.

notion_info() {
  local ACTION="$1"
  local DETAILS="$2"
  command printf '\033[1;32m%12s\033[0m %s\n' "${ACTION}" "${DETAILS}" 1>&2
}

notion_error() {
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

  notion_error "Could not determine the current version"
  return 1
}

# returns the os name to be used in the packaged release,
# including the openssl info if necessary
parse_os_info() {
  local uname_str="$1"
  local openssl_version="$2"

  case "$uname_str" in
    Linux)
      echo "linux-openssl-$(parse_openssl_version "$openssl_version")"
      ;;
    Darwin)
      echo "macos"
      ;;
    *)
      notion_error "Releases for '$uname_str' are not yet supported. Please modify this script and the install script to support this OS."
      return 1
  esac
  return 0
}

# parse the OpenSSL version from the input text
parse_openssl_version() {
  let version_str="$1"

  if [[ "$version_str" =~ ([^ ]*)\ (\d+).(\d+).(\d+) ]]
  then
    echo "matched $version_str" >&2
    return 0
  else
    echo "could NOT match $version_str" >&2
    return 1
  fi
}

### END FUNCTIONS

# read the current version from Cargo.toml
cargo_toml_contents="$(<Cargo.toml)"
NOTION_VERSION="$(parse_version "$cargo_toml_contents")"
# TODO: check exit code

# figure out the OS details
os="$(uname -s)"
openssl_version="$(openssl version)"
NOTION_OS="$(parse_os_info "$os" "$openssl_version")"
# TODO: check exit code

release_filename="notion-$NOTION_VERSION-$NOTION_OS"

# first make sure the release binaries have been built
notion_info 'Building' "Notion release $(bold "$release_filename")"
cargo build --release

# then package up the binaries together
target_dir="target/release"
notion_info 'Packaging' "the compiled binaries"
cd "$target_dir"
tar -czf "$release_filename.tar.gz" notion shim

notion_info 'Completed' "release in file $(bold "$target_dir/$release_filename.tar.gz")"
