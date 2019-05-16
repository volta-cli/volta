#!/usr/bin/env bash

# This is the bootstrap Unix installer served by `https://get.volta.sh`.
# Its responsibility is to query the system to determine what OS (and in the
# case of Linux, what OpenSSL version) the system has, and then proceed to
# fetch and install the appropriate build of Volta.

volta_get_latest_release() {
  curl --silent https://volta.sh/latest-version
}

volta_eprintf() {
  command printf "$1\n" 1>&2
}

volta_info() {
  local ACTION
  local DETAILS
  ACTION="$1"
  DETAILS="$2"
  command printf '\033[1;32m%12s\033[0m %s\n' "${ACTION}" "${DETAILS}" 1>&2
}

volta_error() {
  command printf '\033[1;31mError\033[0m: ' 1>&2
  volta_eprintf "$1"
}

volta_warning() {
  command printf '\033[1;33mWarning\033[0m: ' 1>&2
  volta_eprintf "$1"
  volta_eprintf ''
}

volta_request() {
  command printf "\033[1m$1\033[0m" 1>&2
  volta_eprintf ''
}

legacy_install_dir() {
  printf "%s" "${NOTION_HOME:-"$HOME/.notion"}"
}

# Check for a legacy installation from when the tool was named Notion.
volta_check_legacy_installation() {
  local LEGACY_INSTALL_DIR="$(legacy_install_dir)"
  if [[ -d "$LEGACY_INSTALL_DIR" ]]; then
      volta_eprintf ""
      volta_error "You have an existing Notion install, which can't be automatically upgraded to Volta."
      volta_request "       Please delete $LEGACY_INSTALL_DIR and try again."
      volta_eprintf ""
      volta_eprintf "(We plan to implement automatic upgrades in the future. Thanks for bearing with us!)"
      volta_eprintf ""
      exit 1
  fi
}

volta_install_dir() {
  printf %s "${VOLTA_HOME:-"$HOME/.volta"}"
}

# Check for an existing installation that needs to be removed.
volta_check_existing_installation() {
  local LATEST_VERSION="$1"
  local INSTALL_DIR="$(volta_install_dir)"
  local VOLTA_BIN="${INSTALL_DIR}/volta"

  if [[ -n "$INSTALL_DIR" && -x "$VOLTA_BIN" ]]; then
    local PREV_VOLTA_VERSION
    # Some 0.1.* builds would eagerly validate package.json even for benign commands,
    # so just to be safe we'll ignore errors and consider those to be 0.1 as well.
    PREV_VOLTA_VERSION="$( ($VOLTA_BIN --version 2>/dev/null || echo 0.1) | sed -E 's/^.*([0-9]+\.[0-9]+\.[0-9]+).*$/\1/')"
    if [ "$PREV_VOLTA_VERSION" == "$LATEST_VERSION" ]; then
      volta_eprintf ""
      volta_eprintf "Latest version $LATEST_VERSION already installed"
      exit 0
    fi
    if [[ "$PREV_VOLTA_VERSION" == 0.1* || "$PREV_VOLTA_VERSION" == 0.2* || "$PREV_VOLTA_VERSION" == 0.3* ]]; then
      volta_eprintf ""
      volta_error "Your Volta installation is out of date and can't be automatically upgraded."
      volta_request "       Please delete or move $INSTALL_DIR and try again."
      volta_eprintf ""
      volta_eprintf "(We plan to implement automatic upgrades in the future. Thanks for bearing with us!)"
      volta_eprintf ""
      exit 1
    fi
  fi
}

# determines the major and minor version of OpenSSL on the system
volta_get_openssl_version() {
  local LIB
  local LIBNAME
  local FULLVERSION
  local MAJOR
  local MINOR

  # By default, we'll guess OpenSSL 1.0.1.
  LIB="$(openssl version 2>/dev/null || echo 'OpenSSL 1.0.1')"

  LIBNAME="$(echo $LIB | awk '{print $1;}')"

  if [[ "$LIBNAME" != "OpenSSL" ]]; then
    volta_error "Your system SSL library ($LIBNAME) is not currently supported on this OS."
    volta_eprintf ""
    exit 1
  fi

  FULLVERSION="$(echo $LIB | awk '{print $2;}')"
  MAJOR="$(echo ${FULLVERSION} | cut -d. -f1)"
  MINOR="$(echo ${FULLVERSION} | cut -d. -f2)"

  # If we have version 1.0.x, check for RHEL / CentOS style OpenSSL SONAME (.so.10)
  if [[ "${MAJOR}.${MINOR}" == "1.0" && -f "/usr/lib64/libcrypto.so.10" ]]; then
    echo "rhel"
  else
    echo "${MAJOR}.${MINOR}"
  fi
}

VOLTA_LATEST_VERSION=$(volta_get_latest_release)

volta_info 'Checking' "for existing Volta installation"
volta_check_legacy_installation
volta_check_existing_installation "$VOLTA_LATEST_VERSION"


case $(uname) in
    Linux)
        VOLTA_OS="linux-openssl-$(volta_get_openssl_version)"
        VOLTA_PRETTY_OS=Linux
        ;;
    Darwin)
        VOLTA_OS=macos
        VOLTA_PRETTY_OS=macOS
        ;;
    *)
        volta_error "The current operating system does not appear to be supported by Volta."
        volta_eprintf ""
        exit 1
esac

VOLTA_INSTALLER="https://github.com/volta-cli/volta/releases/download/v${VOLTA_LATEST_VERSION}/volta-${VOLTA_LATEST_VERSION}-${VOLTA_OS}.sh"

volta_info 'Fetching' "${VOLTA_PRETTY_OS} installer"

curl -#SLf ${VOLTA_INSTALLER} | bash
STATUS=$?

exit $STATUS
