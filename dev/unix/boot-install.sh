#!/usr/bin/env bash

# This is the bootstrap Unix installer served by `https://get.notionjs.com`.
# Its responsibility is to query the system to determine what OS (and in the
# case of Linux, what OpenSSL version) the system has, and then proceed to
# fetch and install the appropriate build of Notion.

notion_get_latest_release() {
  curl --silent https://www.notionjs.com/latest-version
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
  local INSTALL_DIR
  INSTALL_DIR="$(notion_install_dir)"

  local NOTION_BIN
  NOTION_BIN="${INSTALL_DIR}/notion"

  if [[ -n "$INSTALL_DIR" && -x "$NOTION_BIN" ]]; then
    local PREV_NOTION_VERSION    
    # Some 0.1.* builds would eagerly validate package.json even for benign commands,
    # so just to be safe we'll ignore errors and consider those to be 0.1 as well.
    PREV_NOTION_VERSION="$(($NOTION_BIN --version 2>/dev/null || echo 0.1) | sed -E 's/^.*([0-9]+\.[0-9]+\.[0-9]+).*$/\1/')"
    if [[ "$PREV_NOTION_VERSION" == 0.1* ]]; then
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

notion_info 'Checking' "for existing Notion installation"
notion_check_existing_installation

NOTION_LATEST_VERSION=$(notion_get_latest_release)

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

NOTION_INSTALLER="https://github.com/notion-cli/notion/releases/download/v${NOTION_LATEST_VERSION}/notion-${NOTION_LATEST_VERSION}-${NOTION_OS}.sh"

notion_info 'Fetching' "${NOTION_PRETTY_OS} installer"

curl -#SLf ${NOTION_INSTALLER} | bash
STATUS=$?

exit $STATUS
