#!/usr/bin/env bash

usage() {
  cat <<'END_USAGE'
build.sh: generate notion's generic unix installation script

usage: build.sh <notion> <launchbin> <launchscript>
  <notion>        path to the binary `notion` executable
  <launchbin>     path to the binary `launchbin` executable
  <launchscript>  path to the binary `launchscript` executable

The output file is saved as ./install.sh.
END_USAGE
}

if [ "$#" -ne 3 ]; then
  usage >&2
  exit 1
fi

encode_base64_sed_command() {
  command printf "s|<PLACEHOLDER_$2_PAYLOAD>|" > $1.base64.txt
  cat $3 | base64 - | tr -d '\n' >> $1.base64.txt
  command printf "|\n" >> $1.base64.txt
}

encode_base64_sed_command notion NOTION $1
encode_base64_sed_command launchbin LAUNCHBIN $2
encode_base64_sed_command launchscript LAUNCHSCRIPT $3

sed -f notion.base64.txt -f launchbin.base64.txt -f launchscript.base64.txt < install.sh.in > install.sh

chmod 755 install.sh

rm *.base64.txt
