#!/usr/bin/env bash

# This is the bootstrap Unix installer served by `https://get.jetson.sh`.
# Its responsibility is to query the system to determine what OS (and in the
# case of Linux, what OpenSSL version) the system has, and then proceed to
# fetch and install the appropriate build of Jetson.

jetson_get_latest_release() {
  curl --silent https://www.jetson.sh/latest-version
}

jetson_eprintf() {
  command printf "$1\n" 1>&2
}

jetson_info() {
  local ACTION
  local DETAILS
  ACTION="$1"
  DETAILS="$2"
  command printf '\033[1;32m%12s\033[0m %s\n' "${ACTION}" "${DETAILS}" 1>&2
}

jetson_error() {
  command printf '\033[1;31mError\033[0m: ' 1>&2
  jetson_eprintf "$1"
}

jetson_warning() {
  command printf '\033[1;33mWarning\033[0m: ' 1>&2
  jetson_eprintf "$1"
  jetson_eprintf ''
}

jetson_request() {
  command printf "\033[1m$1\033[0m" 1>&2
  jetson_eprintf ''
}

jetson_install_dir() {
  printf %s "${JETSON_HOME:-"$HOME/.jetson"}"
}

# Check for an existing installation that needs to be removed.
jetson_check_existing_installation() {
  local INSTALL_DIR
  INSTALL_DIR="$(jetson_install_dir)"

  local JETSON_BIN
  JETSON_BIN="${INSTALL_DIR}/jetson"

  if [[ -n "$INSTALL_DIR" && -x "$JETSON_BIN" ]]; then
    local PREV_JETSON_VERSION    
    # Some 0.1.* builds would eagerly validate package.json even for benign commands,
    # so just to be safe we'll ignore errors and consider those to be 0.1 as well.
    PREV_JETSON_VERSION="$(($JETSON_BIN --version 2>/dev/null || echo 0.1) | sed -E 's/^.*([0-9]+\.[0-9]+\.[0-9]+).*$/\1/')"
    if [[ "$PREV_JETSON_VERSION" == 0.1* || "$PREV_JETSON_VERSION" == 0.2* ]]; then
      jetson_eprintf ""
      jetson_error "Your Jetson installation is out of date and can't be automatically upgraded."
      jetson_request "       Please delete or move $(jetson_install_dir) and try again."
      jetson_eprintf ""
      jetson_eprintf "(We plan to implement automatic upgrades in the future. Thanks for bearing with us!)"
      jetson_eprintf ""
      exit 1
    fi
  fi
}

# determines the major and minor version of OpenSSL on the system
jetson_get_openssl_version() {
  local LIB
  local LIBNAME
  local FULLVERSION
  local MAJOR
  local MINOR

  # By default, we'll guess OpenSSL 1.0.1.
  LIB="$(openssl version 2>/dev/null || echo 'OpenSSL 1.0.1')"

  LIBNAME="$(echo $LIB | awk '{print $1;}')"

  if [[ "$LIBNAME" != "OpenSSL" ]]; then
    jetson_error "Your system SSL library ($LIBNAME) is not currently supported on this OS."
    jetson_eprintf ""
    exit 1
  fi

  FULLVERSION="$(echo $LIB | awk '{print $2;}')"
  MAJOR="$(echo ${FULLVERSION} | cut -d. -f1)"
  MINOR="$(echo ${FULLVERSION} | cut -d. -f2)"
  echo "${MAJOR}.${MINOR}"
}

jetson_info 'Checking' "for existing Jetson installation"
jetson_check_existing_installation

JETSON_LATEST_VERSION=$(jetson_get_latest_release)

case $(uname) in
    Linux)
        if [[ "$JETSON_LATEST_VERSION" == 0.1* ]]; then
          JETSON_OS=linux
        else
          JETSON_OS="linux-openssl-$(jetson_get_openssl_version)"
        fi
        JETSON_PRETTY_OS=Linux
        ;;
    Darwin)
        JETSON_OS=macos
        JETSON_PRETTY_OS=macOS
        ;;
    *)
        jetson_error "The current operating system does not appear to be supported by Jetson."
        jetson_eprintf ""
        exit 1
esac

JETSON_INSTALLER="https://github.com/jetson-cli/jetson/releases/download/v${JETSON_LATEST_VERSION}/jetson-${JETSON_LATEST_VERSION}-${JETSON_OS}.sh"

jetson_info 'Fetching' "${JETSON_PRETTY_OS} installer"

curl -#SLf ${JETSON_INSTALLER} | bash
STATUS=$?

exit $STATUS
