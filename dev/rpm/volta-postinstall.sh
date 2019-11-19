#!/usr/bin/env bash
# Post-install setup for Volta:
#  * create the home directory layout
#  * create symlinks to the binaries in /usr/bin/volta/*
#  * copy over the shell integration files (load.*)
#  * create symlinks and shims
#  * update the user's profile

# exit on error
set -e

# where Volta will be installed
VOLTA_HOME="$HOME/.volta"
# where the RPM installed the compiled binaries
BIN_DIR="/usr/bin/volta-lib"


# symlink bins in home dir to /usr/bin/volta/{volta,shim}
create_bin_symlinks() {
  local install_dir="$1"

  info 'Creating' "symlinks to installed binaries"

  local bin_shims=( shim )

  # remove these symlinks or binaries if they exist, and re-link them
  # (using -f so there is no error if the files don't exist)
  for shim in "${bin_shims[@]}"; do
    rm -f "$install_dir/$shim"
    ln -s "$BIN_DIR/$shim" "$install_dir/$shim"
  done
}

# copy over the shell integration files
copy_shell_integration() {
  local install_dir="$1"
  info 'Copying' "Shell integration scripts"
  cp "$BIN_DIR"/load.* "$install_dir/"
}

# (the rest of these functions were taken from dev/unix/volta-install.sh)

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

eprintf() {
  command printf '%s\n' "$1" 1>&2
}

create_tree() {
  local install_dir="$1"

  info 'Creating' "directory layout"

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
  #         default/

  mkdir -p "$install_dir"
  mkdir -p "$install_dir"/bin
  mkdir -p "$install_dir"/cache/node
  mkdir -p "$install_dir"/log
  mkdir -p "$install_dir"/tmp
  mkdir -p "$install_dir"/tools/image/{node,packages,yarn}
  mkdir -p "$install_dir"/tools/inventory/{node,packages,yarn}
  mkdir -p "$install_dir"/tools/default
}

# NOTE: had to comment out the `chmod` lines here, because the binaries are installed by root
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
      #chmod 755 "$file"
    fi
  done

  # re-link the non-user shims
  for shim in "${main_shims[@]}"; do
    ln -s "$shim_exec" "$install_dir/bin/$shim"
    #chmod 755 "$install_dir/bin/$shim"
  done

  # and make sure these are executable
  #chmod 755 "$shim_exec" "$main_exec"
}

# If file exists, echo it
echo_fexists() {
  [ -f "$1" ] && echo "$1"
}

detect_profile() {
  local shellname="$1"
  local uname="$2"

  if [ -f "$PROFILE" ]; then
    echo "$PROFILE"
    return
  fi

  # try to detect the current shell
  case "$shellname" in
    bash)
      # Shells on macOS default to opening with a login shell, while Linuxes
      # default to a *non*-login shell, so if this is macOS we look for
      # `.bash_profile` first; if it's Linux, we look for `.bashrc` first. The
      # `*` fallthrough covers more than just Linux: it's everything that is not
      # macOS (Darwin). It can be made narrower later if need be.
      case $uname in
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
      case $uname in
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

update_profile() {
  local install_dir="$1"

  local profile_install_dir=$(echo "$install_dir" | sed "s:^$HOME:\$HOME:")
  local detected_profile="$(detect_profile $(basename "/$SHELL") $(uname -s) )"
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
    if ! command grep -qc 'VOLTA_HOME' "$detected_profile"; then
      command printf "$path_str" >> "$detected_profile"
    else
      warning "Your profile ($detected_profile) already mentions Volta and has not been changed."
    fi
  fi

  if command grep -qc 'NOTION_HOME' "$detected_profile"; then
    eprintf ''
    warning "Your profile ($detected_profile) mentions Notion."
    eprintf "         You probably want to remove that."
    eprintf ''
  fi
}


# setup the home directory layout
create_tree "$VOLTA_HOME"

# create symlinks to the installed binaries in /usr/bin/volta/*
create_bin_symlinks "$VOLTA_HOME"

# copy over the shell integration files
copy_shell_integration "$VOLTA_HOME"

# create symlinks for the shims
create_symlinks "$VOLTA_HOME"

# update the user's profile
update_profile "$VOLTA_HOME"

info "Finished" 'installation. Open a new terminal to start using Volta!'
