#!/usr/bin/env bash

script_dir="$(dirname "$0")"

usage() {
  cat <<END_USAGE
build.sh: generate notion's generic unix installation script

usage: build.sh [target]
  [target]   build artifacts to use ('release' or 'debug', defaults to 'release')

The output file is saved as $script_dir/install.sh.
END_USAGE
}

target_dir='release'
if [ "$#" -gt 1 ] || ! [[ "$1" =~ (debug|release) ]]; then
  usage
  exit 1
elif [ -n "$1" ]; then
  target_dir="$1"
fi

encode_base64_sed_command() {
  command printf "s|<PLACEHOLDER_$2_PAYLOAD>|" > $1.base64.txt
  cat $3 | base64 - | tr -d '\n' >> $1.base64.txt
  command printf "|\n" >> $1.base64.txt
}

build_dir="$script_dir/../../target/$target_dir"

encode_base64_sed_command notion NOTION "$build_dir/notion"
encode_base64_sed_command node NODE "$build_dir/node"
encode_base64_sed_command launchbin LAUNCHBIN "$build_dir/launchbin"
encode_base64_sed_command launchscript LAUNCHSCRIPT "$build_dir/launchscript"

sed -f notion.base64.txt \
    -f node.base64.txt \
    -f launchbin.base64.txt \
    -f launchscript.base64.txt \
    < "$script_dir/install.sh.in" > "$script_dir/install.sh"

chmod 755 "$script_dir/install.sh"

rm notion.base64.txt \
   node.base64.txt \
   launchbin.base64.txt \
   launchscript.base64.txt
