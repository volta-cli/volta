#!/usr/bin/env bash

# This is the bootstrap Unix installer served by `https://get.notionjs.com`.
# Its responsibility is to query the system to determine what OS (and in the
# case of Linux, what OpenSSL version) the system has, and then proceed to
# fetch and install the appropriate build of Notion.

usage() {
    cat >&2 <<END_USAGE
notion-install
The installer for Notion

USAGE:
    notion-install [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                  Prints help information

OPTIONS:
        --debug                 Compile and install Notion locally, using the debug target
        --release               Compile and install Notion locally, using the release target
        --version <version>     Install a specific release version of Notion
END_USAGE
}

# TODO: go thru all these functions and make sure the places that call them are checking the return value
notion_get_latest_release() {
  # curl --silent https://www.notionjs.com/latest-version
  # TODO: change this back
  # TODO: make this configurable for Artifactory?
  # OR just have a separate internal script...
  echo "0.3.1" # for testing
}

notion_eprintf() {
  command printf "$1\n" 1>&2
}

notion_info() {
  local action="$1"
  local details="$2"
  command printf '\033[1;32m%12s\033[0m %s\n' "$action" "$details" 1>&2
}

notion_error() {
  command printf '\033[1;31mError\033[0m: ' 1>&2
  notion_eprintf "$1"
}

notion_warning() {
  command printf '\033[1;33mWarning\033[0m: ' 1>&2
  notion_eprintf "$1"
  notion_eprintf ''
}

notion_request() {
  command printf "\033[1m$1\033[0m" 1>&2
  notion_eprintf ''
}

# TODO: change description once this is finalized
# Check for an existing installation that needs to be removed.
notion_upgrade_is_ok() {
  local _will_install_version="$1"
  local _install_dir="$2"

  # TODO: check for downgrade? will probably have to wipe and install

  local _notion_bin="$_install_dir/notion"

  # TODO: don't exit, just return from this
  if [[ -n "$_install_dir" && -x "$_notion_bin" ]]; then
    # Some 0.1.* builds would eagerly validate package.json even for benign commands,
    # so just to be safe we'll ignore errors and consider those to be 0.1 as well.
    local _prev_notion_version="$( ($_notion_bin --version 2>/dev/null || echo 0.1) | sed -E 's/^.*([0-9]+\.[0-9]+\.[0-9]+).*$/\1/')"
    if [ "$_prev_notion_version" == "$_will_install_version" ]; then
      notion_eprintf ""
      notion_eprintf "Version $_will_install_version already installed"
      return 1
    fi
    if [[ "$_prev_notion_version" == 0.1* || "$_prev_notion_version" == 0.2* ]]; then
      notion_eprintf ""
      notion_error "Your Notion installation is out of date and can't be automatically upgraded."
      notion_request "       Please delete or move $_install_dir and try again."
      notion_eprintf ""
      notion_eprintf "(We plan to implement automatic upgrades in the future. Thanks for bearing with us!)"
      notion_eprintf ""
      return 1
    fi
  fi
  # should be ok to install
  return 0
}

# returns the os name to be used in the packaged release,
# including the openssl info if necessary
parse_os_info() {
  local uname_str="$1"
  local openssl_version="$2"

  # TODO: need to check for version 0.1* anymore?
  # case $(uname) in
  #   Linux)
  #     if [[ "$_version" == 0.1* ]]; then
  #       NOTION_OS=linux
  #     else
  #       NOTION_OS="linux-openssl-$(notion_get_openssl_version)"
  #     fi
  #     NOTION_PRETTY_OS=Linux
  #     ;;
  #   Darwin)
  #     NOTION_OS=macos
  #     NOTION_PRETTY_OS=macOS
  #     ;;
  #   *)
  #     notion_error "The current operating system does not appear to be supported by Notion."
  #     notion_eprintf ""
  #     exit 1
  # esac

  case "$uname_str" in
    Linux)
      local parsed_version="$(parse_openssl_version "$openssl_version")"
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
      notion_error "Releases for '$uname_str' are not yet supported. You will need to add another OS case to this script, and to the install script to support this OS."
      return 1
  esac
  return 0
}

# TODO: description
parse_os_pretty() {
  local uname_str="$1"

  case "$uname_str" in
    Linux)
      echo "Linux"
      ;;
    Darwin)
      echo "macOS"
      ;;
    *)
      # don't know which OS specificaly, just return the uname
      echo "$uname_str"
  esac
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
    notion_error "Releases for '$libname' not currently supported. Supported libraries are: ${SUPPORTED_SSL_LIBS[@]}."
    return 1
  else
    notion_error "Could not determine OpenSSL version for '$version_str'. You probably need to update the regex to handle this output."
    return 1
  fi
}

notion_create_tree() {
  local _install_dir="$1"

  # .notion/
  #     bin/
  #     cache/
  #         node/
  #     log/
  #     tmp/
  #     tools/
  #         image/
  #             node/
  #             packages/
  #             yarn/
  #         inventory/
  #             node/
  #             packages/
  #             yarn/
  #         user/

  mkdir -p "$_install_dir"
  mkdir -p "$_install_dir"/bin
  mkdir -p "$_install_dir"/cache/node
  mkdir -p "$_install_dir"/log
  mkdir -p "$_install_dir"/tmp
  mkdir -p "$_install_dir"/tools/image/{node,packages,yarn}
  mkdir -p "$_install_dir"/tools/inventory/{node,packages,yarn}
  mkdir -p "$_install_dir"/tools/user
}

notion_install_version() {
  local version_to_install="$1"
  local install_dir="$2"

  case "$version_to_install" in
    latest)
      notion_info 'Installing' "latest version of Notion"
      notion_install_release "$(notion_get_latest_release)" "$install_dir"
      ;;
    local-dev)
      notion_info 'Installing' "Notion locally after compiling"
      notion_install_local "dev" "$install_dir"
      ;;
    local-release)
      notion_info 'Installing' "Notion locally after compiling with '--release'"
      notion_install_local "release" "$install_dir"
      ;;
    *)
      # assume anything else is a specific version
      notion_info 'Installing' "Notion version $version_to_install"
      notion_install_release "$version_to_install" "$install_dir"
      ;;
  esac
}

notion_install_release() {
  local version="$1"
  local install_dir="$2"

  notion_info 'Checking' "for existing Notion installation"
  if notion_upgrade_is_ok "$version" "$install_dir"
  then
    download_archive="$(notion_download_release "$version"; exit "$?")"
    exit_status="$?"
    if [ "$exit_status" != 0 ]
    then
      notion_error "Could not download Notion version '$version'\n\nSee https://github.com/notion-cli/notion/releases for a list of available releases"
      return "$exit_status"
    fi

    notion_install_from_file "$download_archive" "$install_dir"
  fi
}

notion_install_local() {
  local dev_or_release="$1"
  local install_dir="$2"

  # compile and package the binaries, then install from that local archive
  local _compiled_archive="$(notion_compile_and_package "$dev_or_release")"
  notion_install_from_file "$_compiled_archive" "$install_dir"
}

notion_compile_and_package() {
  local dev_or_release="$1"
  # TODO: call the release script to do this, and return the packaged archive file
  # TODO: parse the output to get the archive file name
  dev/unix/release.sh "--$dev_or_release"
  # TODO: check exit status
  echo "target/release/notion-0.3.0-macos.tar.gz"
}

notion_download_release() {
  local _version="$1"

  local uname_str="$(uname -s)"
  local openssl_version="$(openssl version)"
  local os_info="$(parse_os_info "$uname_str" "$openssl_version")"
  local pretty_os_name="$(parse_os_pretty "$uname_str")"

  notion_info 'Fetching' "archive for $pretty_os_name, version $_version"

  # store the downloaded archive in a temporary directory
  local _download_dir="$(mktemp -d)"
  local _filename="notion-$_version-$os_info.tar.gz"
  local _download_file="$_download_dir/$_filename"

  # TODO: for now, download the test files from my desktop
  local notion_archive="http://mistewar-ld2.linkedin.biz:8080/$_filename"
  # this will eventually be
  # local notion_archive="https://github.com/notion-cli/notion/releases/download/v$_version/$_filename"

  curl --progress-bar --show-error --location --fail "$notion_archive" --output "$_download_file" && echo "$_download_file"
}

notion_install_from_file() {
  local _archive="$1"
  local _extract_to="$2"

  notion_info 'Creating' "directory layout"
  notion_create_tree "$_extract_to"

  notion_info 'Extracting' "Notion binaries and launchers"
  # extract the files to the specified directory
  echo "running: 'tar -xzvf "$_archive" -C "$_extract_to"'" >&2
  tar -xzvf "$_archive" -C "$_extract_to"
}

# return if sourced (for testing the functions)
return 0 2>/dev/null

# TODO: do I actually want this?
# exit on error
# set -e

# default to installing the latest available Notion version
install_version="latest"

# install to NOTION_HOME, defaulting to ~/.notion
install_dir="${NOTION_HOME:-"$HOME/.notion"}"

# parse command line options
while [ $# -gt 0 ]
do
  arg="$1"

  case "$arg" in
    -h|--help)
      usage
      exit 0
      ;;
    --dev)
      shift # shift off the argument
      install_version="local-dev"
      ;;
    --release)
      shift # shift off the argument
      install_version="local-release"
      ;;
    --version)
      shift # shift off the argument
      install_version="$1"
      shift # shift off the value
      ;;
    *)
      notion_error "unknown option: '$arg'"
      usage
      exit 1
      ;;
  esac
done

notion_install_version "$install_version" "$install_dir"
# TODO: use stuff from install.sh.in to modify profile and whatever

