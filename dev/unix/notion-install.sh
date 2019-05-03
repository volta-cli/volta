#!/usr/bin/env bash

# This is the bootstrap Unix installer served by `https://get.notionjs.com`.
# Its responsibility is to query the system to determine what OS (and in the
# case of Linux, what OpenSSL version) the system has, and then proceed to
# fetch and install the appropriate build of Notion.

usage() {
    cat >&2 <<END_USAGE
notion-install
The installer for Notion

USAGE:
    notion-install [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                  Prints help information

OPTIONS:
        --debug                 Compile and install Notion locally, using the debug target
        --release               Compile and install Notion locally, using the release target
        --version <version>     Install a specific release version of Notion
END_USAGE
}

notion_info() {
  local action="$1"
  local details="$2"
  command printf '\033[1;32m%12s\033[0m %s\n' "$action" "$details" 1>&2
}

notion_error() {
  command printf '\033[1;31mError\033[0m: %s\n\n' "$1" 1>&2
}

notion_warning() {
  command printf '\033[1;33mWarning\033[0m: %s\n\n' "$1" 1>&2
}

notion_request() {
  command printf '\033[1m%s\033[0m\n' "$1" 1>&2
}

notion_eprintf() {
  command printf '%s\n' "$1" 1>&2
}

bold() {
  command printf '\033[1m%s\033[0m' "$1"
}

# TODO: clean these functions up
notion_create_binaries() {
  local INSTALL_DIR

  INSTALL_DIR="$(notion_install_dir)"

  notion_unpack_notion        > "${INSTALL_DIR}"/notion
  notion_unpack_shim          > "${INSTALL_DIR}"/shim
  notion_unpack_bash_launcher > "${INSTALL_DIR}"/load.sh
  notion_unpack_fish_launcher > "${INSTALL_DIR}"/load.fish

  # Remove any existing binaries for tools so that the symlinks can be installed
  # using -f so there is no error if the files don't exist
  rm -f "${INSTALL_DIR}"/bin/node
  rm -f "${INSTALL_DIR}"/bin/npm
  rm -f "${INSTALL_DIR}"/bin/npx
  rm -f "${INSTALL_DIR}"/bin/yarn

  for FILE_NAME in "${INSTALL_DIR}"/bin/*; do
    if [ -e "${FILE_NAME}" ] && ! [ -d "${FILE_NAME}" ]; then
      rm -f "${FILE_NAME}"
      ln -s "${INSTALL_DIR}"/shim "${FILE_NAME}"
    fi
  done

  ln -s "${INSTALL_DIR}"/shim "${INSTALL_DIR}"/bin/node
  ln -s "${INSTALL_DIR}"/shim "${INSTALL_DIR}"/bin/npm
  ln -s "${INSTALL_DIR}"/shim "${INSTALL_DIR}"/bin/npx
  ln -s "${INSTALL_DIR}"/shim "${INSTALL_DIR}"/bin/yarn

  chmod 755 "${INSTALL_DIR}/"/notion "${INSTALL_DIR}/bin"/* "${INSTALL_DIR}"/shim
}

notion_try_profile() {
  if [ -z "${1-}" ] || [ ! -f "${1}" ]; then
    return 1
  fi
  echo "${1}"
}

notion_detect_profile() {
  if [ -n "${PROFILE}" ] && [ -f "${PROFILE}" ]; then
    echo "${PROFILE}"
    return
  fi

  local DETECTED_PROFILE
  DETECTED_PROFILE=''
  local SHELLTYPE
  SHELLTYPE="$(basename "/$SHELL")"

  if [ "$SHELLTYPE" = "bash" ]; then
    if [ -f "$HOME/.bashrc" ]; then
      DETECTED_PROFILE="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
      DETECTED_PROFILE="$HOME/.bash_profile"
    fi
  elif [ "$SHELLTYPE" = "zsh" ]; then
    DETECTED_PROFILE="$HOME/.zshrc"
  elif [ "$SHELLTYPE" = "fish" ]; then
    DETECTED_PROFILE="$HOME/.config/fish/config.fish"
  fi

  if [ -z "$DETECTED_PROFILE" ]; then
    for EACH_PROFILE in ".profile" ".bashrc" ".bash_profile" ".zshrc" ".config/fish/config.fish"
    do
      if DETECTED_PROFILE="$(notion_try_profile "${HOME}/${EACH_PROFILE}")"; then
        break
      fi
    done
  fi

  if [ -n "$DETECTED_PROFILE" ]; then
    echo "$DETECTED_PROFILE"
  fi
}

notion_build_path_str() {
  local PROFILE
  PROFILE="$1"
  local PROFILE_INSTALL_DIR
  PROFILE_INSTALL_DIR="$2"

  local PATH_STR
  if [[ $PROFILE =~ \.fish$ ]]; then
    PATH_STR="\\nset -gx NOTION_HOME \"${PROFILE_INSTALL_DIR}\"\\ntest -s \"\$NOTION_HOME/load.fish\"; and source \"\$NOTION_HOME/load.fish\"\\n\\nstring match -r \".notion\" \"\$PATH\" > /dev/null; or set -gx PATH \"\$NOTION_HOME/bin\" \$PATH"
  else
    PATH_STR="\\nexport NOTION_HOME=\"${PROFILE_INSTALL_DIR}\"\\n[ -s \"\$NOTION_HOME/load.sh\" ] && \\. \"\$NOTION_HOME/load.sh\"\\n\\nexport PATH=\"\${NOTION_HOME}/bin:\$PATH\""
  fi

  echo "$PATH_STR"
}

notion_install() {
  if [ -n "${NOTION_HOME-}" ] && [ -e "${NOTION_HOME}" ] && ! [ -d "${NOTION_HOME}" ]; then
    notion_error "\$NOTION_HOME is set but is not a directory (${NOTION_HOME})."
    notion_eprintf "Please check your profile scripts and environment."
    exit 1
  fi

  notion_info 'Creating' "Notion directory tree ($(notion_install_dir))"
  notion_create_tree

  notion_info 'Unpacking' "\`notion\` executable and shims"
  notion_create_binaries

  notion_info 'Editing' "user profile"
  local NOTION_PROFILE
  NOTION_PROFILE="$(notion_detect_profile)"
  local PROFILE_INSTALL_DIR
  PROFILE_INSTALL_DIR=$(notion_install_dir | sed "s:^$HOME:\$HOME:")
  local PATH_STR
  PATH_STR="$(notion_build_path_str "$NOTION_PROFILE" "$PROFILE_INSTALL_DIR")"

  if [ -z "${NOTION_PROFILE-}" ] ; then
    local TRIED_PROFILE
    if [ -n "${PROFILE}" ]; then
      TRIED_PROFILE="${NOTION_PROFILE} (as defined in \$PROFILE), "
    fi
    notion_error "No user profile found."
    notion_eprintf "Tried ${TRIED_PROFILE-}~/.bashrc, ~/.bash_profile, ~/.zshrc, ~/.profile, and ~.config/fish/config.fish."
    notion_eprintf ''
    notion_eprintf "You can either create one of these and try again or add this to the appropriate file:"
    notion_eprintf "${PATH_STR}"
    exit 1
  else
    if ! command grep -qc 'NOTION_HOME' "$NOTION_PROFILE"; then
      command printf "${PATH_STR}" >> "$NOTION_PROFILE"
    else
      notion_eprintf ''
      notion_warning "Your profile (${NOTION_PROFILE}) already mentions Notion and has not been changed."
      notion_eprintf ''
    fi
  fi

  notion_info "Finished" 'installation. Open a new terminal to start using Notion!'
  exit 0
}


# TODO: go thru all these functions and make sure the places that call them are checking the return value
notion_get_latest_release() {
  # curl --silent https://www.notionjs.com/latest-version
  # TODO: change this back
  # TODO: make this configurable for Artifactory?
  # OR just have a separate internal script...
  echo "0.3.1" # for testing
}

# TODO: change description once this is finalized
# Check for an existing installation that needs to be removed.
notion_upgrade_is_ok() {
  local _will_install_version="$1"
  local _install_dir="$2"

  # TODO: check for downgrade? will probably have to wipe and install

  local _notion_bin="$_install_dir/notion"

  # TODO: don't exit, just return from this
  if [[ -n "$_install_dir" && -x "$_notion_bin" ]]; then
    # Some 0.1.* builds would eagerly validate package.json even for benign commands,
    # so just to be safe we'll ignore errors and consider those to be 0.1 as well.
    local _prev_notion_version="$( ($_notion_bin --version 2>/dev/null || echo 0.1) | sed -E 's/^.*([0-9]+\.[0-9]+\.[0-9]+).*$/\1/')"
    if [ "$_prev_notion_version" == "$_will_install_version" ]; then
      notion_eprintf ""
      notion_eprintf "Version $_will_install_version already installed"
      return 1
    fi
    if [[ "$_prev_notion_version" == 0.1* || "$_prev_notion_version" == 0.2* ]]; then
      notion_eprintf ""
      notion_error "Your Notion installation is out of date and can't be automatically upgraded."
      notion_request "       Please delete or move $_install_dir and try again."
      notion_eprintf ""
      notion_eprintf "(We plan to implement automatic upgrades in the future. Thanks for bearing with us!)"
      notion_eprintf ""
      return 1
    fi
  fi
  # should be ok to install
  return 0
}

# returns the os name to be used in the packaged release,
# including the openssl info if necessary
parse_os_info() {
  local uname_str="$1"
  local openssl_version="$2"

  # TODO: need to check for version 0.1* anymore?
  # case $(uname) in
  #   Linux)
  #     if [[ "$_version" == 0.1* ]]; then
  #       NOTION_OS=linux
  #     else
  #       NOTION_OS="linux-openssl-$(notion_get_openssl_version)"
  #     fi
  #     NOTION_PRETTY_OS=Linux
  #     ;;
  #   Darwin)
  #     NOTION_OS=macos
  #     NOTION_PRETTY_OS=macOS
  #     ;;
  #   *)
  #     notion_error "The current operating system does not appear to be supported by Notion."
  #     notion_eprintf ""
  #     exit 1
  # esac

  case "$uname_str" in
    Linux)
      parsed_version="$(parse_openssl_version "$openssl_version")"
      # if there was an error, return
      exit_code="$?"
      if [ "$exit_code" != 0 ]
      then
        return "$exit_code"
      fi

      echo "linux-openssl-$parsed_version"
      ;;
    Darwin)
      echo "macos"
      ;;
    *)
      notion_error "Releases for '$uname_str' are not yet supported. You will need to add another OS case to this script, and to the install script to support this OS."
      return 1
  esac
  return 0
}

# TODO: description
parse_os_pretty() {
  local uname_str="$1"

  case "$uname_str" in
    Linux)
      echo "Linux"
      ;;
    Darwin)
      echo "macOS"
      ;;
    *)
      # don't know which OS specificaly, just return the uname
      echo "$uname_str"
  esac
}

# return true(0) if the element is contained in the input arguments
# called like:
#  if element_in "foo" "${array[@]}"; then ...
element_in() {
  local match="$1";
  shift

  local element;
  # loop over the input arguments and return when a match is found
  for element in "$@"
  do
    [ "$element" == "$match" ] && return 0
  done
  return 1
}

# parse the OpenSSL version from the input text
parse_openssl_version() {
  local version_str="$1"

  # array containing the SSL libraries that are supported
  # would be nice to use a bash 4.x associative array, but bash 3.x is the default on OSX
  SUPPORTED_SSL_LIBS=( 'OpenSSL' )

  # use regex to get the library name and version
  # typical version string looks like 'OpenSSL 1.0.1e-fips 11 Feb 2013'
  if [[ "$version_str" =~ ^([^\ ]*)\ ([0-9]+\.[0-9]+\.[0-9]+) ]]
  then
    # check that the lib is supported
    libname="${BASH_REMATCH[1]}"
    if element_in "$libname" "${SUPPORTED_SSL_LIBS[@]}"
    then
      # lib is supported, return the version
      echo "${BASH_REMATCH[2]}"
      return 0
    fi
    notion_error "Releases for '$libname' not currently supported. Supported libraries are: ${SUPPORTED_SSL_LIBS[@]}."
    return 1
  else
    notion_error "Could not determine OpenSSL version for '$version_str'. You probably need to update the regex to handle this output."
    return 1
  fi
}

notion_create_tree() {
  local _install_dir="$1"

  # .notion/
  #     bin/
  #     cache/
  #         node/
  #     log/
  #     tmp/
  #     tools/
  #         image/
  #             node/
  #             packages/
  #             yarn/
  #         inventory/
  #             node/
  #             packages/
  #             yarn/
  #         user/

  mkdir -p "$_install_dir"
  mkdir -p "$_install_dir"/bin
  mkdir -p "$_install_dir"/cache/node
  mkdir -p "$_install_dir"/log
  mkdir -p "$_install_dir"/tmp
  mkdir -p "$_install_dir"/tools/image/{node,packages,yarn}
  mkdir -p "$_install_dir"/tools/inventory/{node,packages,yarn}
  mkdir -p "$_install_dir"/tools/user
}

notion_install_version() {
  local version_to_install="$1"
  local install_dir="$2"

  case "$version_to_install" in
    latest)
      notion_info 'Installing' "latest version of Notion"
      notion_install_release "$(notion_get_latest_release)" "$install_dir"
      ;;
    local-dev)
      notion_info 'Installing' "Notion locally after compiling"
      notion_install_local "dev" "$install_dir"
      ;;
    local-release)
      notion_info 'Installing' "Notion locally after compiling with '--release'"
      notion_install_local "release" "$install_dir"
      ;;
    *)
      # assume anything else is a specific version
      notion_info 'Installing' "Notion version $version_to_install"
      notion_install_release "$version_to_install" "$install_dir"
      ;;
  esac
}

notion_install_release() {
  local version="$1"
  local install_dir="$2"

  notion_info 'Checking' "for existing Notion installation"
  if notion_upgrade_is_ok "$version" "$install_dir"
  then
    download_archive="$(notion_download_release "$version"; exit "$?")"
    exit_status="$?"
    if [ "$exit_status" != 0 ]
    then
      notion_error "Could not download Notion version '$version'\n\nSee https://github.com/notion-cli/notion/releases for a list of available releases"
      return "$exit_status"
    fi

    notion_install_from_file "$download_archive" "$install_dir"
  fi
}

notion_install_local() {
  local dev_or_release="$1"
  local install_dir="$2"

  # compile and package the binaries, then install from that local archive
  local _compiled_archive="$(notion_compile_and_package "$dev_or_release")"
  notion_install_from_file "$_compiled_archive" "$install_dir"
}

notion_compile_and_package() {
  local dev_or_release="$1"
  # TODO: call the release script to do this, and return the packaged archive file
  # TODO: parse the output to get the archive file name
  dev/unix/release.sh "--$dev_or_release"
  # TODO: check exit status
  echo "target/release/notion-0.3.0-macos.tar.gz"
}

notion_download_release() {
  local _version="$1"

  local uname_str="$(uname -s)"
  local openssl_version="$(openssl version)"
  local os_info="$(parse_os_info "$uname_str" "$openssl_version")"
  local pretty_os_name="$(parse_os_pretty "$uname_str")"

  notion_info 'Fetching' "archive for $pretty_os_name, version $_version"

  # store the downloaded archive in a temporary directory
  local _download_dir="$(mktemp -d)"
  local _filename="notion-$_version-$os_info.tar.gz"
  local _download_file="$_download_dir/$_filename"

  # TODO: for now, download the test files from my desktop
  local notion_archive="http://mistewar-ld2.linkedin.biz:8080/$_filename"
  # this will eventually be
  # local notion_archive="https://github.com/notion-cli/notion/releases/download/v$_version/$_filename"

  curl --progress-bar --show-error --location --fail "$notion_archive" --output "$_download_file" && echo "$_download_file"
}

notion_install_from_file() {
  local _archive="$1"
  local _extract_to="$2"

  notion_info 'Creating' "directory layout"
  notion_create_tree "$_extract_to"

  notion_info 'Extracting' "Notion binaries and launchers"
  # extract the files to the specified directory
  echo "running: 'tar -xzvf "$_archive" -C "$_extract_to"'" >&2
  tar -xzvf "$_archive" -C "$_extract_to"
}

# return if sourced (for testing the functions)
return 0 2>/dev/null

# TODO: do I actually want this?
# exit on error
# set -e

# default to installing the latest available Notion version
install_version="latest"

# install to NOTION_HOME, defaulting to ~/.notion
install_dir="${NOTION_HOME:-"$HOME/.notion"}"

# parse command line options
while [ $# -gt 0 ]
do
  arg="$1"

  case "$arg" in
    -h|--help)
      usage
      exit 0
      ;;
    --dev)
      shift # shift off the argument
      install_version="local-dev"
      ;;
    --release)
      shift # shift off the argument
      install_version="local-release"
      ;;
    --version)
      shift # shift off the argument
      install_version="$1"
      shift # shift off the value
      ;;
    *)
      notion_error "unknown option: '$arg'"
      usage
      exit 1
      ;;
  esac
done

notion_install_version "$install_version" "$install_dir"
# TODO: use stuff from install.sh.in to modify profile and whatever else

