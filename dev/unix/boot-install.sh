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
  notion_eprintf ''
}

notion_warning() {
  command printf '\033[1;33mWarning\033[0m: ' 1>&2
  notion_eprintf "$1"
  notion_eprintf ''
}

# determines the major and minor version of OpenSSL on the system
notion_get_openssl_version() {
  local LIB
  local FULLVERSION
  local MAJOR
  local MINOR
  # By default, we'll guess OpenSSL 1.0.1.
  LIB="$(openssl version 2>/dev/null || echo 'OpenSSL 1.0.1')"
  FULLVERSION="$(echo $LIB | awk '{print $2;}')"
  MAJOR="$(echo ${FULLVERSION} | cut -d. -f1)"
  MINOR="$(echo ${FULLVERSION} | cut -d. -f2)"
  echo "${MAJOR}.${MINOR}"
}

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
        exit 1
esac

NOTION_INSTALLER="https://github.com/notion-cli/notion/releases/download/v${NOTION_LATEST_VERSION}/notion-${NOTION_LATEST_VERSION}-${NOTION_OS}.sh"

notion_info 'Fetching' "${NOTION_PRETTY_OS} installer"

curl -sSLf ${NOTION_INSTALLER} | bash
STATUS=$?

exit $STATUS
