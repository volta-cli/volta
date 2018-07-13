#!/usr/bin/env bash

usage() {
  cat <<END_USAGE
rustfmt.sh: check the formatting of the Notion project

usage: rustfmt [--check]
  --check   check for formatting issues

By default, the changes will be automatically applied. If the --check option is
supplied, no files are changed and a diff is printed if issues are found.

END_USAGE
}

run_quiet() {
  IFS='%' captured=$($* 2>&1)
  if [ "$?" -ne 0 ]; then
    echo $captured 1>&2
    exit 1
  fi
}

CHECK=write-mode=diff
TOOLCHAIN_VERSION=1.26.0

if [ "$#" -gt 1 ] || [ "$#" -eq 1 ] && ! [[ "$1" =~ "--check" ]]; then
  usage
  exit 1
elif [ -n "$1" ]; then
  extra_arg=--${CHECK}
fi

run_quiet rustup install ${TOOLCHAIN_VERSION}
run_quiet rustup component add --toolchain ${TOOLCHAIN_VERSION} rustfmt-preview

cargo +${TOOLCHAIN_VERSION} fmt -- ${extra_arg}
