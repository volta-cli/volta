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

CHECK=write-mode=diff
TOOLCHAIN_VERSION=1.26.0

if [ "$#" -gt 1 ] || [ "$#" -eq 1 ] && ! [[ "$1" =~ "--check" ]]; then
  usage
  exit 1
elif [ -n "$1" ]; then
  extra_arg=--${CHECK}
fi

IFS='%'
install_rust_output=$(rustup install ${TOOLCHAIN_VERSION} 2>&1)
if [ "$?" -ne 0 ]; then
  echo $install_rust_output 1>&2
  exit 1
fi

IFS='%'
install_rustfmt_output=$(rustup component add --toolchain ${TOOLCHAIN_VERSION} rustfmt-preview 2>&1)
if [ "$?" -ne 0 ]; then
  echo $install_rustfmt_output 1>&2
  exit 1
fi

cargo +${TOOLCHAIN_VERSION} fmt -- ${extra_arg}
