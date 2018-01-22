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

instantiate_template() {
  local notion
  local launchbin
  local launchscript

  notion=$(cat $1 | base64 -)
  launchbin=$(cat $2 | base64 -)
  launchscript=$(cat $3 | base64 -)

  sed -e "s|%%PLACEHOLDER_NOTION_PAYLOAD%%|$notion|" -e "s|%%PLACEHOLDER_LAUNCHBIN_PAYLOAD%%|$launchbin|" -e "s|%%PLACEHOLDER_LAUNCHSCRIPT_PAYLOAD%%|$launchscript|"
}

cat install.sh.in | instantiate_template $1 $2 $3

# # base64-encode the `notion` executable
# encoded=$(cat $1 | base64 -)

# # find the line in the install script template containing the placeholder string
# placeholder_line=$(grep -n "^%%PLACEHOLDER_NOTION_PAYLOAD%%" install.sh.in | cut -d ':' -f 1)
# before_placeholder=$[${placeholder_line} - 1]
# after_placeholder=$[${placeholder_line} + 1]

# # generate the install script by replacing the placeholder with the base64-encoded binary
# head -n ${before_placeholder} install.sh.in > install.sh
# echo $encoded >> install.sh
# tail -n +${after_placeholder} install.sh.in >> install.sh

# # make the script executable
# chmod 755 ./install.sh

#
# echo "Installation script generated at ./install.sh." >&2
