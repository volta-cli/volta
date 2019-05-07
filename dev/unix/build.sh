#!/usr/bin/env bash

script_dir="$(dirname "$0")"

usage() {
  cat <<END_USAGE
build.sh: generate volta's generic unix installation script

usage: build.sh [target]
  [target]   build artifacts to use ('release' or 'debug', defaults to 'release')

The output file is saved as $script_dir/install.sh.
END_USAGE
}

if [ -z "$1" ]; then
  target_dir='release'
elif [[ "$1" =~ (debug|release) ]]; then
  target_dir="$1"
else
  usage
  exit 1
fi

encode_base64_sed_command() {
  command printf "s|<PLACEHOLDER_$2_PAYLOAD>|" > $1.base64.txt
  cat $3 | base64 - | tr -d '\n' >> $1.base64.txt
  command printf "|\n" >> $1.base64.txt
}

encode_expand_sed_command() {
  # This atrocity is a combination of:
  # - https://unix.stackexchange.com/questions/141387/sed-replace-string-with-file-contents
  # - https://serverfault.com/questions/391360/remove-line-break-using-awk
  # - https://stackoverflow.com/questions/1421478/how-do-i-use-a-new-line-replacement-in-a-bsd-sed
  command printf "s|<PLACEHOLDER_$2_PAYLOAD>|$(sed 's/|/\\|/g' $3 | awk '{printf "%s\\\n",$0} END {print ""}' )\\\n|\n" > $1.expand.txt
}

build_dir="$script_dir/../../target/$target_dir"
shell_dir="$script_dir/../../shell"

encode_base64_sed_command volta VOLTA "$build_dir/volta"
encode_base64_sed_command shim SHIM "$build_dir/shim"
encode_expand_sed_command bash_launcher BASH_LAUNCHER "$shell_dir/unix/load.sh"
encode_expand_sed_command fish_launcher FISH_LAUNCHER "$shell_dir/unix/load.fish"

sed -f volta.base64.txt \
    -f shim.base64.txt \
    -f bash_launcher.expand.txt \
    -f fish_launcher.expand.txt \
    < "$script_dir/install.sh.in" > "$script_dir/install.sh"

chmod 755 "$script_dir/install.sh"

rm volta.base64.txt \
   shim.base64.txt \
   bash_launcher.expand.txt \
   fish_launcher.expand.txt
