#!/usr/bin/env bash

# This is the bootstrap Unix installer served by `https://get.volta.sh`.
# Its responsibility is to query the system to determine what OS (and in the
# case of Linux, what OpenSSL version) the system has, fetch and install the
# appropriate build of Volta, and modify the user's profile.

# NOTE: to use an internal company repo, change how this determines the latest version
get_latest_release() {
  echo "0.5.0"
  # TODO:
  # curl --silent "https://volta.sh/latest-version"
}

release_url() {
  echo "https://github.com/volta-cli/volta/releases"
}

usage() {
    cat >&2 <<END_USAGE
volta-install: The installer for Volta

USAGE:
    volta-install [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                  Prints help information

OPTIONS:
        --dev                   Compile and install Volta locally, using the dev target
        --release               Compile and install Volta locally, using the release target
        --version <version>     Install a specific release version of Volta
END_USAGE
}

info() {
  local action="$1"
  local details="$2"
  command printf '\033[1;32m%12s\033[0m %s\n' "$action" "$details" 1>&2
}

error() {
  command printf '\033[1;31mError\033[0m: %s\n\n' "$1" 1>&2
}

warning() {
  command printf '\033[1;33mWarning\033[0m: %s\n\n' "$1" 1>&2
}

request() {
  command printf '\033[1m%s\033[0m\n' "$1" 1>&2
}

eprintf() {
  command printf '%s\n' "$1" 1>&2
}

bold() {
  command printf '\033[1m%s\033[0m' "$1"
}

# create symlinks for shims in the bin/ dir
create_symlinks() {
  local install_dir="$1"

  info 'Creating' "symlinks and shims"
  local main_shims=( node npm npx yarn )
  local shim_exec="$install_dir/shim"
  local main_exec="$install_dir/volta"

  # remove these symlinks or binaries if they exist, so that the symlinks can be created later
  # (using -f so there is no error if the files don't exist)
  for shim in "${main_shims[@]}"; do
    rm -f "$install_dir/bin/$shim"
  done

  # update symlinks for any shims created by the user
  for file in "$install_dir"/bin/*; do
    if [ -e "$file" ] && ! [ -d "$file" ]; then
      rm -f "$file"
      ln -s "$shim_exec" "$file"
      chmod 755 "$file"
    fi
  done

  # re-link the non-user shims
  for shim in "${main_shims[@]}"; do
    ln -s "$shim_exec" "$install_dir/bin/$shim"
    chmod 755 "$install_dir/bin/$shim"
  done

  # and make sure these are executable
  chmod 755 "$shim_exec" "$main_exec"
}

# If file exists, echo it
echo_fexists() {
  [ -f "$1" ] && echo "$1"
}

detect_profile() {
  if [ -f "$PROFILE" ]; then
    echo "$PROFILE"
    return
  fi

  # TODO: I think basename and uname will need to be args, so I can test this beast
  # try to detect the current shell
  case "$(basename "/$SHELL")" in
    bash)
      # Shells on macOS default to opening with a login shell, while Linuxes
      # default to a *non*-login shell, so if this is macOS we look for
      # `.bash_profile` first; if it's Linux, we look for `.bashrc` first. The
      # `*` fallthrough covers more than just Linux: it's everything that is not
      # macOS (Darwin). It can be made narrower later if need be.
      case $(uname) in
        Darwin)
          echo_fexists "$HOME/.bash_profile" || echo_fexists "$HOME/.bashrc"
        ;;
        *)
          echo_fexists "$HOME/.bashrc" || echo_fexists "$HOME/.bash_profile"
        ;;
      esac
      ;;
    zsh)
      echo "$HOME/.zshrc"
      ;;
    fish)
      echo "$HOME/.config/fish/config.fish"
      ;;
    *)
      # Fall back to checking for profile file existence. Once again, the order
      # differs between macOS and everything else.
      local profiles
      case $(uname) in
        Darwin)
          profiles=( .profile .bash_profile .bashrc .zshrc .config/fish/config.fish )
          ;;
        *)
          profiles=( .profile .bashrc .bash_profile .zshrc .config/fish/config.fish )
          ;;
      esac

      for profile in "${profiles[@]}"; do
        echo_fexists "$HOME/$profile" && break
      done
      ;;
  esac
}

# generate shell code to source the loading script and modify the path for the input profile
build_path_str() {
  local profile="$1"
  local profile_install_dir="$2"

  if [[ $profile =~ \.fish$ ]]; then
    # fish uses a little different syntax to load the shell integration script, and modify the PATH
    cat <<END_FISH_SCRIPT

set -gx VOLTA_HOME "$profile_install_dir"
test -s "\$VOLTA_HOME/load.fish"; and source "\$VOLTA_HOME/load.fish"

string match -r ".volta" "\$PATH" > /dev/null; or set -gx PATH "\$VOLTA_HOME/bin" \$PATH
END_FISH_SCRIPT
  else
    # bash and zsh
    cat <<END_BASH_SCRIPT

export VOLTA_HOME="$profile_install_dir"
[ -s "\$VOLTA_HOME/load.sh" ] && . "\$VOLTA_HOME/load.sh"

export PATH="\$VOLTA_HOME/bin:\$PATH"
END_BASH_SCRIPT
  fi
}

# check for issue with VOLTA_HOME
# if it is set, and exists, but is not a directory, the install will fail
volta_home_is_ok() {
  if [ -n "${VOLTA_HOME-}" ] && [ -e "$VOLTA_HOME" ] && ! [ -d "$VOLTA_HOME" ]; then
    error "\$VOLTA_HOME is set but is not a directory ($VOLTA_HOME)."
    eprintf "Please check your profile scripts and environment."
    return 1
  fi
  return 0
}

update_profile() {
  local install_dir="$1"

  local profile_install_dir=$(echo "$install_dir" | sed "s:^$HOME:\$HOME:")
  local detected_profile="$(detect_profile)"
  local path_str="$(build_path_str "$detected_profile" "$profile_install_dir")"
  info 'Editing' "user profile ($detected_profile)"

  if [ -z "${detected_profile-}" ] ; then
    error "No user profile found."
    eprintf "Tried \$PROFILE ($PROFILE), ~/.bashrc, ~/.bash_profile, ~/.zshrc, ~/.profile, and ~/.config/fish/config.fish."
    eprintf ''
    eprintf "You can either create one of these and try again or add this to the appropriate file:"
    eprintf "$path_str"
    return 1
  else
    # TODO: without using grep? just bash?
    if ! command grep -qc 'VOLTA_HOME' "$detected_profile"; then
      command printf "$path_str" >> "$detected_profile"
    else
      warning "Your profile ($detected_profile) already mentions Volta and has not been changed."
    fi
  fi
}

legacy_dir() {
  echo "${NOTION_HOME:-"$HOME/.notion"}"
}

# Check for a legacy installation from when the tool was named Notion.
no_legacy_install() {
  if [ -d "$(legacy_dir)" ]; then
    eprintf ""
    error "You have existing Notion install, which can't be automatically upgraded to Volta."
    request "       Please delete $(legacy_dir) and try again."
    eprintf ""
    eprintf "(We plan to implement automatic upgrades in the future. Thanks for bearing with us!)"
    eprintf ""
    return 1
  fi
  return 0
}

# TODO: change description once this is finalized
# Check for an existing installation that needs to be removed.
upgrade_is_ok() {
  local will_install_version="$1"
  local install_dir="$2"

  # TODO: this will not be able to install things prior to the Volta rename, so check for that (probably version 0.5.0)

  # TODO: check for downgrade? will probably have to wipe and install

  local volta_bin="$install_dir/volta"

  # TODO: don't exit, just return from this
  if [[ -n "$install_dir" && -x "$volta_bin" ]]; then
    # Some 0.1.* builds would eagerly validate package.json even for benign commands,
    # so just to be safe we'll ignore errors and consider those to be 0.1 as well.
    local prev_version="$( ($volta_bin --version 2>/dev/null || echo 0.1) | sed -E 's/^.*([0-9]+\.[0-9]+\.[0-9]+).*$/\1/')"
    if [ "$prev_version" == "$will_install_version" ]; then
      eprintf "Version $will_install_version already installed"
      return 1
    fi
    # TODO: have to do more version checks here
    if [[ "$prev_version" == 0.1* || "$prev_version" == 0.2* ]]; then
      eprintf ""
      error "Your Volta installation is out of date and can't be automatically upgraded."
      request "       Please delete or move $install_dir and try again."
      eprintf ""
      eprintf "(We plan to implement automatic upgrades in the future. Thanks for bearing with us!)"
      eprintf ""
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

  # TODO: this will not be able to install things prior to the Volta rename, so check for that (probably version 0.5.0)

  # TODO: need to check for version 0.1*, because those binaries are named differently
  # TODO: need to check for versions prior to the use of this script, because those will use the install.sh installer
  # TODO: will ALSO need to check for versions prior to the final rename, becuase those use Volta
  # case $(uname) in
  #   Linux)
  #     if [[ "$version" == 0.1* ]]; then
  #       VOLTA_OS=linux
  #     else
  #       VOLTA_OS="linux-openssl-$(volta_get_openssl_version)"
  #     fi
  #     VOLTA_PRETTY_OS=Linux
  #     ;;
  #   Darwin)
  #     VOLTA_OS=macos
  #     VOLTA_PRETTY_OS=macOS
  #     ;;
  #   *)
  #     error "The current operating system does not appear to be supported by Volta."
  #     eprintf ""
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
      error "Releases for '$uname_str' are not yet supported. You will need to add another OS case to this script, and to the install script to support this OS."
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
  for element in "$@"; do
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
    error "Releases for '$libname' not currently supported. Supported libraries are: ${SUPPORTED_SSL_LIBS[@]}."
    return 1
  else
    error "Could not determine OpenSSL version for '$version_str'. You probably need to update the regex to handle this output."
    return 1
  fi
}

create_tree() {
  local install_dir="$1"

  # .volta/
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

  mkdir -p "$install_dir"
  mkdir -p "$install_dir"/bin
  mkdir -p "$install_dir"/cache/node
  mkdir -p "$install_dir"/log
  mkdir -p "$install_dir"/tmp
  mkdir -p "$install_dir"/tools/image/{node,packages,yarn}
  mkdir -p "$install_dir"/tools/inventory/{node,packages,yarn}
  mkdir -p "$install_dir"/tools/user
}

install_version() {
  local version_to_install="$1"
  local install_dir="$2"

  if ! volta_home_is_ok; then
    exit 1
  fi

  case "$version_to_install" in
    latest)
      info 'Installing' "latest version of Volta"
      install_release "$(get_latest_release)" "$install_dir"
      ;;
    local-dev)
      info 'Installing' "Volta locally after compiling"
      install_local "dev" "$install_dir"
      ;;
    local-release)
      info 'Installing' "Volta locally after compiling with '--release'"
      install_local "release" "$install_dir"
      ;;
    *)
      # assume anything else is a specific version
      info 'Installing' "Volta version $version_to_install"
      install_release "$version_to_install" "$install_dir"
      ;;
  esac

  if [ "$?" == 0 ]
  then
    create_symlinks "$install_dir" &&
      update_profile "$install_dir" &&
      info "Finished" 'installation. Open a new terminal to start using Volta!'

    if command grep -qc 'NOTION_HOME' "$VOLTA_PROFILE"; then
      eprintf ''
      warning "Your profile ($VOLTA_PROFILE) mentions Notion."
      eprintf "         You probably want to remove that."
      eprintf ''
    fi
  fi
}

install_release() {
  local version="$1"
  local install_dir="$2"

  info 'Checking' "for existing Volta installation"
  if no_legacy_install && upgrade_is_ok "$version" "$install_dir"
  then
    download_archive="$(download_release "$version"; exit "$?")"
    exit_status="$?"
    if [ "$exit_status" != 0 ]
    then
      error "Could not download Volta version '$version'. See $(release_url) for a list of available releases"
      return "$exit_status"
    fi

    install_from_file "$download_archive" "$install_dir"
  fi
}

install_local() {
  local dev_or_release="$1"
  local install_dir="$2"

  # TODO: there's no version available here?
  info 'Checking' "for existing Volta installation"
  if upgrade_is_ok "$version" "$install_dir"
  then
    # compile and package the binaries, then install from that local archive
    local compiled_archive="$(compile_and_package "$dev_or_release")"
    install_from_file "$compiled_archive" "$install_dir"
  fi
}

compile_and_package() {
  local dev_or_release="$1"
  # TODO: call the release script to do this, and return the packaged archive file
  # TODO: parse the output to get the archive file name
  dev/unix/release.sh "--$dev_or_release"
  # TODO: check exit status
  echo "target/release/volta-0.4.0-macos.tar.gz"
}

download_release() {
  local version="$1"

  local uname_str="$(uname -s)"
  local openssl_version="$(openssl version)"
  local os_info="$(parse_os_info "$uname_str" "$openssl_version")"
  local pretty_os_name="$(parse_os_pretty "$uname_str")"

  info 'Fetching' "archive for $pretty_os_name, version $version"

  # TODO: refactor this to pull the repo-specific code out, for internal integration
  # using $version and $os_info, return the URL

  # store the downloaded archive in a temporary directory
  local download_dir="$(mktemp -d)"
  local filename="volta-$version-$os_info.tar.gz"
  local download_file="$download_dir/$filename"
  local archive_url="$(release_url)/download/v$version/$filename"

  curl --progress-bar --show-error --location --fail "$archive_url" --output "$download_file" && echo "$download_file"
}

install_from_file() {
  local archive="$1"
  local extract_to="$2"

  info 'Creating' "directory layout"
  create_tree "$extract_to"

  info 'Extracting' "Volta binaries and launchers"
  # extract the files to the specified directory
  tar -xzvf "$archive" -C "$extract_to"
}


# return if sourced (for testing the functions above)
return 0 2>/dev/null

# default to installing the latest available version
version_to_install="latest"

# install to VOLTA_HOME, defaulting to ~/.volta
install_dir="${VOLTA_HOME:-"$HOME/.volta"}"

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
      version_to_install="local-dev"
      ;;
    --release)
      shift # shift off the argument
      version_to_install="local-release"
      ;;
    --version)
      shift # shift off the argument
      version_to_install="$1"
      shift # shift off the value
      ;;
    *)
      error "unknown option: '$arg'"
      usage
      exit 1
      ;;
  esac
done

install_version "$version_to_install" "$install_dir"
