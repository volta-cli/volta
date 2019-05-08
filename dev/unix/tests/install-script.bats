# test the notion-install.sh script

# load the functions from the script
source dev/unix/notion-install.sh


# test building the path string

@test "build_path_str for fish" {
  expected_output=$(cat <<END_FISH_STRING

set -gx NOTION_HOME "$HOME/.whatever"
test -s "\$NOTION_HOME/load.fish"; and source "\$NOTION_HOME/load.fish"

string match -r ".notion" "\$PATH" > /dev/null; or set -gx PATH "\$NOTION_HOME/bin" \$PATH
END_FISH_STRING
)

  run build_path_str "$HOME/.config/fish/config.fish" "$HOME/.whatever"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}


@test "notion_build_path_str for bash and zsh" {
  expected_output=$(cat <<END_BASH_STRING

export NOTION_HOME="$HOME/.whatever"
[ -s "\$NOTION_HOME/load.sh" ] && . "\$NOTION_HOME/load.sh"

export PATH="\$NOTION_HOME/bin:\$PATH"
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


# test NOTION_HOME settings

@test "notion_home_is_ok - true cases" {
  # unset is fine
  unset NOTION_HOME
  run notion_home_is_ok
  [ "$status" -eq 0 ]

  # empty is fine
  NOTION_HOME=""
  run notion_home_is_ok
  [ "$status" -eq 0 ]

  # non-existing dir is fine
  NOTION_HOME="/some/dir/that/does/not/exist/anywhere"
  run notion_home_is_ok
  [ "$status" -eq 0 ]

  # existing dir is fine
  NOTION_HOME="$HOME"
  run notion_home_is_ok
  [ "$status" -eq 0 ]
}

@test "notion_home_is_ok - not ok" {
  # file is not ok
  NOTION_HOME="$(mktemp)"
  run notion_home_is_ok
  [ "$status" -eq 1 ]
}


# TODO: test creating symlinks
