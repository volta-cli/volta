#!/usr/bin/env bash

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

notion_exit() {
  notion_cleanup
  exit $1
}

notion_cleanup() {
  unset -f notion_get_latest_release notion_eprintf notion_info notion_error notion_warning notion_exit notion_cleanup
}

NOTION_LATEST_VERSION=$(notion_get_latest_release)

case $(uname) in
    Linux)
        NOTION_OS=linux
        NOTION_PRETTY_OS=Linux
        ;;
    Darwin)
        NOTION_OS=macos
        NOTION_PRETTY_OS=macOS
        ;;
    *)
        notion_error "The current operating system does not appear to be supported by Notion."
        notion_exit 1
esac

NOTION_INSTALLER="https://github.com/notion-cli/notion/releases/download/v${NOTION_LATEST_VERSION}/notion-${NOTION_LATEST_VERSION}-${NOTION_OS}.sh"

notion_info 'Fetching' "${NOTION_PRETTY_OS} installer"

curl -sSLf ${NOTION_INSTALLER} | bash
STATUS=$?

notion_exit $STATUS
