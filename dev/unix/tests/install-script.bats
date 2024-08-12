# test the volta-install.sh script

# load the functions from the script
source dev/unix/volta-install.sh


# happy path test to parse the version from Cargo.toml
@test "parse_cargo_version - normal Cargo.toml" {
  input=$(cat <<'END_CARGO_TOML'
[package]
name = "volta"
version = "0.7.38"
authors = ["David Herman <david.herman@gmail.com>"]
license = "BSD-2-Clause"
END_CARGO_TOML
)

  expected_output="0.7.38"

  run parse_cargo_version "$input"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# it doesn't parse the version from other dependencies
@test "parse_cargo_version - error" {
  input=$(cat <<'END_CARGO_TOML'
[dependencies]
volta-core = { path = "crates/volta-core" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0.37"
console = "0.6.1"
END_CARGO_TOML
)

  expected_output=$(echo -e "\033[1;31mError\033[0m: Could not determine the current version from Cargo.toml")

  run parse_cargo_version "$input"
  [ "$status" -eq 1 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# linux
@test "parse_os_info - linux" {
  expected_output="linux"

  run parse_os_info "Linux"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# macos
@test "parse_os_info - macos" {
  expected_output="macos"

  run parse_os_info "Darwin"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# unsupported OS
@test "parse_os_info - unsupported OS" {
  expected_output=""

  run parse_os_info "DOS"
  [ "$status" -eq 1 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# test element_in helper function
@test "element_in works correctly" {
  run element_in "foo" "foo" "bar" "baz"
  [ "$status" -eq 0 ]

  array=( "foo" "bar" "baz" )
  run element_in "foo" "${array[@]}"
  [ "$status" -eq 0 ]
  run element_in "bar" "${array[@]}"
  [ "$status" -eq 0 ]
  run element_in "baz" "${array[@]}"
  [ "$status" -eq 0 ]

  run element_in "fob" "${array[@]}"
  [ "$status" -eq 1 ]
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
