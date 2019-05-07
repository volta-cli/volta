# test the notion-install.sh script

# load the functions from the script
source dev/unix/notion-install.sh


# test building the path string

@test "notion_build_path_str for fish" {
  expected_output=$(cat <<END_FISH_STRING

set -gx NOTION_HOME "$HOME/.whatever"
test -s "\$NOTION_HOME/load.fish"; and source "\$NOTION_HOME/load.fish"

string match -r ".notion" "\$PATH" > /dev/null; or set -gx PATH "\$NOTION_HOME/bin" \$PATH
END_FISH_STRING
)

  run notion_build_path_str "$HOME/.config/fish/config.fish" "$HOME/.whatever"
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

  run notion_build_path_str "$HOME/.bashrc" "$HOME/.whatever"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")

  run notion_build_path_str "$HOME/.bash_profile" "$HOME/.whatever"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")

  run notion_build_path_str "$HOME/.zshrc" "$HOME/.whatever"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")

  run notion_build_path_str "$HOME/.profile" "$HOME/.whatever"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

