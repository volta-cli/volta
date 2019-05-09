# test the volta-install.sh script

# load the functions from the script
source dev/unix/volta-install.sh


# test building the path string

@test "build_path_str for fish" {
  expected_output=$(cat <<END_FISH_STRING

set -gx VOLTA_HOME "$HOME/.whatever"
test -s "\$VOLTA_HOME/load.fish"; and source "\$VOLTA_HOME/load.fish"

string match -r ".volta" "\$PATH" > /dev/null; or set -gx PATH "\$VOLTA_HOME/bin" \$PATH
END_FISH_STRING
)

  run build_path_str "$HOME/.config/fish/config.fish" "$HOME/.whatever"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}


@test "volta_build_path_str for bash and zsh" {
  expected_output=$(cat <<END_BASH_STRING

export VOLTA_HOME="$HOME/.whatever"
[ -s "\$VOLTA_HOME/load.sh" ] && . "\$VOLTA_HOME/load.sh"

export PATH="\$VOLTA_HOME/bin:\$PATH"
END_BASH_STRING
)

  run build_path_str "$HOME/.bashrc" "$HOME/.whatever"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")

  run build_path_str "$HOME/.bash_profile" "$HOME/.whatever"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")

  run build_path_str "$HOME/.zshrc" "$HOME/.whatever"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")

  run build_path_str "$HOME/.profile" "$HOME/.whatever"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}


# test VOLTA_HOME settings

@test "volta_home_is_ok - true cases" {
  # unset is fine
  unset VOLTA_HOME
  run volta_home_is_ok
  [ "$status" -eq 0 ]

  # empty is fine
  VOLTA_HOME=""
  run volta_home_is_ok
  [ "$status" -eq 0 ]

  # non-existing dir is fine
  VOLTA_HOME="/some/dir/that/does/not/exist/anywhere"
  run volta_home_is_ok
  [ "$status" -eq 0 ]

  # existing dir is fine
  VOLTA_HOME="$HOME"
  run volta_home_is_ok
  [ "$status" -eq 0 ]
}

@test "volta_home_is_ok - not ok" {
  # file is not ok
  VOLTA_HOME="$(mktemp)"
  run volta_home_is_ok
  [ "$status" -eq 1 ]
}


# TODO: test creating symlinks
