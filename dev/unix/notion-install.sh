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

notion_install_dir() {
  printf %s "${NOTION_HOME:-"$HOME/.notion"}"
}

# TODO: change description once this is finalized
# Check for an existing installation that needs to be removed.
notion_upgrade_is_ok() {
  local _will_install_version="$1"
  # TODO: check for downgrade? will probably have to wipe and install

  local _install_dir="$(notion_install_dir)"
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
      notion_request "       Please delete or move $(notion_install_dir) and try again."
      notion_eprintf ""
      notion_eprintf "(We plan to implement automatic upgrades in the future. Thanks for bearing with us!)"
      notion_eprintf ""
      return 1
    fi
  fi
  # should be ok to install
  return 0
}

# TODO: get this from the release script (along with the other functions)
# determines the major and minor version of OpenSSL on the system
notion_get_openssl_version() {
  local LIB
  local LIBNAME
  local FULLVERSION
  local MAJOR
  local MINOR

  # By default, we'll guess OpenSSL 1.0.1.
  LIB="$(openssl version 2>/dev/null || echo 'OpenSSL 1.0.1')"

  LIBNAME="$(echo $LIB | awk '{print $1;}')"

  if [[ "$LIBNAME" != "OpenSSL" ]]; then
    notion_error "Your system SSL library ($LIBNAME) is not currently supported on this OS."
    notion_eprintf ""
    exit 1
  fi

  FULLVERSION="$(echo $LIB | awk '{print $2;}')"
  MAJOR="$(echo ${FULLVERSION} | cut -d. -f1)"
  MINOR="$(echo ${FULLVERSION} | cut -d. -f2)"
  echo "${MAJOR}.${MINOR}"
}

notion_install_dir() {
  printf %s "${NOTION_HOME:-"$HOME/.notion"}"
}

notion_create_tree() {
  local _install_dir="$(notion_install_dir)"

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
  mkdir -p "$_install_dir/bin"
  mkdir -p "$_install_dir/cache/node"
  mkdir -p "$_install_dir/log"
  mkdir -p "$_install_dir/tmp"
  mkdir -p "$_install_dir/tools/image/node"
  mkdir -p "$_install_dir/tools/image/packages"
  mkdir -p "$_install_dir/tools/image/yarn"
  mkdir -p "$_install_dir/tools/inventory/node"
  mkdir -p "$_install_dir/tools/inventory/packages"
  mkdir -p "$_install_dir/tools/inventory/yarn"
  mkdir -p "$_install_dir/tools/user"
}

notion_install_version() {
  local version_to_install="$1"

  case "$version_to_install" in
    latest)
      notion_info 'Installing' "latest version of Notion"
      notion_install_release "$(notion_get_latest_release)"
      ;;
    local-debug)
      notion_info 'Installing' "Notion locally after compiling with '--debug'"
      notion_install_local "debug"
      ;;
    local-release)
      notion_info 'Installing' "Notion locally after compiling with '--release'"
      notion_install_local "release"
      ;;
    *)
      # assume anything else is a specific version
      notion_info 'Installing' "Notion version $version_to_install"
      notion_install_release "$version_to_install"
      ;;
  esac
}

notion_install_release() {
  local version="$1"

  notion_info 'Checking' "for existing Notion installation"
  if notion_upgrade_is_ok "$version"
  then
    local _download_archive="$(notion_download_release "$version")"
    notion_install_from_file "$_download_archive"
  fi

  exit
}

notion_install_local() {
  local debug_or_release="$1"

  # TODO: run compile
  local _compiled_archive="$(notion_compile_and_package "$debug_or_release")"
  notion_install_from_file "$_compiled_archive"

  exit
}

notion_compile_and_package() {
  local _debug_or_release="$1"
  # TODO: call the release script to do this, and return the filename that was written
  exit
}

notion_download_release() {
  local _version="$1"
  exit

  # TODO:
  # case $(uname) in
  #     Linux)
  #         if [[ "$NOTION_LATEST_VERSION" == 0.1* ]]; then
  #           NOTION_OS=linux
  #         else
  #           NOTION_OS="linux-openssl-$(notion_get_openssl_version)"
  #         fi
  #         NOTION_PRETTY_OS=Linux
  #         ;;
  #     Darwin)
  #         NOTION_OS=macos
  #         NOTION_PRETTY_OS=macOS
  #         ;;
  #     *)
  #         notion_error "The current operating system does not appear to be supported by Notion."
  #         notion_eprintf ""
  #         exit 1
  # esac

  # # TODO: mktemp and store the download zip file there - for now:
  # _download_dir="$HOME/.test"
  # _filename="notion-$NOTION_LATEST_VERSION-$NOTION_OS.tar.gz"
  # _download_file="$_download_dir/$_filename"

  # # TODO: for now, download the test files from my desktop
  # NOTION_BINS="http://mistewar-ld2.linkedin.biz:8080/$_filename"
  # # TODO: this will be
  # # NOTION_BINS="https://github.com/notion-cli/notion/releases/download/v${NOTION_LATEST_VERSION}/$_filename"


  # notion_info 'Fetching' "binaries/archive/?? for $NOTION_PRETTY_OS, version $NOTION_LATEST_VERSION"

  # curl --progress-bar --show-error --location --fail "$NOTION_BINS" --output "$_download_file"

  # TODO: echo the downloaded file name
}

notion_install_from_file() {
  local _archive="$1"
  exit
  # # TODO: set up directory layout
  # notion_info 'Creating' "directory layout"
  # notion_create_tree

  # # unzip the files
  # notion_info 'Extracting' "files"
  # # TODO: check for error
  # # TODO: all of these should be extracted to ~/.notion (or NOTION_HOME)
  # echo "will run: 'tar -xzvf "$_download_file"'"
}

# TODO: use the return hijinks from the release script here to return if sourced
### END FUNCTIONS

# exit on error
set -e

# default to installing the latest available Notion version
install_version="latest"

# parse command line options
while [ $# -gt 0 ]
do
  arg="$1"

  case "$arg" in
    -h|--help)
      usage
      exit 0
      ;;
    --debug)
      shift # shift off the argument
      # TODO: compile and install locally, going through the whole build process
      # (delegate the compile and packaging to the release script)
      install_version="local-debug"
      ;;
    --release)
      shift # shift off the argument
      # TODO: compile and install locally, going through the whole build process
      # (delegate the compile and packaging to the release script)
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

notion_install_version "$install_version"

