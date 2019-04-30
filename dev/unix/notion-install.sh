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
  local ACTION
  local DETAILS
  ACTION="$1"
  DETAILS="$2"
  command printf '\033[1;32m%12s\033[0m %s\n' "${ACTION}" "${DETAILS}" 1>&2
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

# Check for an existing installation that needs to be removed.
notion_check_existing_installation() {
  local LATEST_VERSION="$1"
  local INSTALL_DIR="$(notion_install_dir)"
  local NOTION_BIN="${INSTALL_DIR}/notion"

  if [[ -n "$INSTALL_DIR" && -x "$NOTION_BIN" ]]; then
    local PREV_NOTION_VERSION
    # Some 0.1.* builds would eagerly validate package.json even for benign commands,
    # so just to be safe we'll ignore errors and consider those to be 0.1 as well.
    PREV_NOTION_VERSION="$( ($NOTION_BIN --version 2>/dev/null || echo 0.1) | sed -E 's/^.*([0-9]+\.[0-9]+\.[0-9]+).*$/\1/')"
    if [ "$PREV_NOTION_VERSION" == "$LATEST_VERSION" ]; then
      notion_eprintf ""
      notion_eprintf "Latest version $LATEST_VERSION already installed"
      exit 0
    fi
    if [[ "$PREV_NOTION_VERSION" == 0.1* || "$PREV_NOTION_VERSION" == 0.2* ]]; then
      notion_eprintf ""
      notion_error "Your Notion installation is out of date and can't be automatically upgraded."
      notion_request "       Please delete or move $(notion_install_dir) and try again."
      notion_eprintf ""
      notion_eprintf "(We plan to implement automatic upgrades in the future. Thanks for bearing with us!)"
      notion_eprintf ""
      exit 1
    fi
  fi
}

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



### END FUNCTIONS

# parse command line options
while [ $# -gt 0 ]
do
  arg="$1"

  case "$arg" in
    -h|--help)
      echo "--help"
      echo "(usage)"
      exit 0
      ;;
    --debug)
      shift # shift off the argument
      # TODO: compile and install locally, going through the whole build process
      # (delegate the compile and packaging to the release script)
      echo "--debug"
      ;;
    --release)
      shift # shift off the argument
      # TODO: compile and install locally, going through the whole build process
      # (delegate the compile and packaging to the release script)
      echo "--release"
      ;;
    --version)
      shift # shift off the argument
      version_to_install="$1"
      shift # shift off the value
      echo "--version $version_to_install"
      ;;
    *)
      # TODO: anything else is whatever at this point
      echo "unknown option: '$arg'"
      echo "(usage)"
      exit 1
      ;;
  esac
done

# TODO: for now
exit

# TODO: this should be the latest version, or the version specified with --version
NOTION_LATEST_VERSION=$(notion_get_latest_release)

notion_info 'Checking' "for existing Notion installation"
notion_check_existing_installation "$NOTION_LATEST_VERSION"

case $(uname) in
    Linux)
        if [[ "$NOTION_LATEST_VERSION" == 0.1* ]]; then
          NOTION_OS=linux
        else
          NOTION_OS="linux-openssl-$(notion_get_openssl_version)"
        fi
        NOTION_PRETTY_OS=Linux
        ;;
    Darwin)
        NOTION_OS=macos
        NOTION_PRETTY_OS=macOS
        ;;
    *)
        notion_error "The current operating system does not appear to be supported by Notion."
        notion_eprintf ""
        exit 1
esac

# TODO: mktemp and store the download zip file there - for now:
_download_dir="$HOME/.test"
_filename="notion-$NOTION_LATEST_VERSION-$NOTION_OS.tar.gz"
_download_file="$_download_dir/$_filename"

# TODO: for now, download the test files from my desktop
NOTION_BINS="http://mistewar-ld2.linkedin.biz:8080/$_filename"
# TODO: this will be
# NOTION_BINS="https://github.com/notion-cli/notion/releases/download/v${NOTION_LATEST_VERSION}/$_filename"


notion_info 'Fetching' "binaries/archive/?? for $NOTION_PRETTY_OS, version $NOTION_LATEST_VERSION"

curl --progress-bar --show-error --location --fail "$NOTION_BINS" --output "$_download_file"

# TODO: set up directory layout
notion_info 'Creating' "directory layout"
notion_create_tree

# unzip the files
notion_info 'Extracting' "files"
# TODO: check for error
# TODO: all of these should be extracted to ~/.notion (or NOTION_HOME)
echo "will run: 'tar -xzvf "$_download_file"'"

exit

# TODO: shouldn't need to pipe into bash anymore
# curl -#SLf ${NOTION_BINS} | bash
# STATUS=$?
# exit $STATUS
