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


# macos
@test "parse_os_info - macos intel" {
  expected_output="macos"

  run parse_os_info "Darwin" "arch is ignored" "openssl is ignored"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

@test "parse_os_info - macos m1" {
  expected_output="macos-aarch64"

  run parse_os_info "Darwin" "arm64" "openssl is ignored"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# linux - supported OpenSSL
@test "parse_os_info - linux-x86_64 with supported OpenSSL" {
  expected_output="linux-openssl-1.2-x86_64"

  run parse_os_info "Linux" "x86_64" "OpenSSL 1.2.3a whatever else"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

@test "parse_os_info - linux-arm64 with supported OpenSSL" {
  expected_output="linux-openssl-1.2-arm64"

  run parse_os_info "Linux" "arm64" "OpenSSL 1.2.3a whatever else"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# linux - unsupported OpenSSL
@test "parse_os_info - linux with unsupported OpenSSL" {
  expected_output=$(echo -e "\033[1;31mError\033[0m: Releases for 'SomeSSL' not currently supported. Supported libraries are: OpenSSL.")

  run parse_os_info "Linux" "x86_64" "SomeSSL 1.2.3a whatever else"
  [ "$status" -eq 1 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# linux - unexpected OpenSSL version format
@test "parse_os_info - linux with unexpected OpenSSL format" {
  expected_output=$(echo -e "\033[1;31mError\033[0m: Could not determine OpenSSL version for 'Some SSL 1.2.4'.")

  run parse_os_info "Linux" "x86_64" "Some SSL 1.2.4"
  [ "$status" -eq 1 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# unsupported OS
@test "parse_os_info - unsupported OS" {
  expected_output=""

  run parse_os_info "DOS" "doesn't matter"
  [ "$status" -eq 1 ]
  diff <(echo "$output") <(echo "$expected_output")
}


# parsing valid OpenSSL version strings
@test "parse_openssl_version - valid versions" {
  expected_output="0.9"
  run parse_openssl_version "OpenSSL 0.9.5a 1 Apr 2000"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")

  expected_output="1.0"
  run parse_openssl_version "OpenSSL 1.0.1e-fips 11 Feb 2013"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# unsupported OpenSSL library
@test "parse_openssl_version - unsupported library" {
  expected_output=$(echo -e "\033[1;31mError\033[0m: Releases for 'LibreSSL' not currently supported. Supported libraries are: OpenSSL.")
  run parse_openssl_version "LibreSSL 2.6.5"
  [ "$status" -eq 1 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# version string with unexpected format
@test "parse_openssl_version - unexpected format" {
  expected_output=$(echo -e "\033[1;31mError\033[0m: Could not determine OpenSSL version for 'Some Weird Version 1.2.3'.")
  run parse_openssl_version "Some Weird Version 1.2.3"
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

@test "check_architecture" {
  # Succeeds for local-release and supported arch
  run check_architecture "local-release" "x86_64"
  [ "$status" -eq 0 ]

  # Succeeds for local-release and unsupported arch
  run check_architecture "local-release" "i686"
  [ "$status" -eq 0 ]

  # Succeeds for local-dev and supported arch
  run check_architecture "local-dev" "x86_64"
  [ "$status" -eq 0 ]

  # Succeeds for local-dev and unsupported arch
  run check_architecture "local-dev" "i686"
  [ "$status" -eq 0 ]

  # Succeeds for latest and supported arch
  run check_architecture "latest" "x86_64"
  [ "$status" -eq 0 ]

  # Fails for latest and unsupported arch
  run check_architecture "latest" "i686"
  [ "$status" -ne 0 ]

  # Succeeds for version and supported arch
  run check_architecture "0.5.0" "x86_64"
  [ "$status" -eq 0 ]

  # Fails for version and unsupported arch
  run check_architecture "0.5.0" "i686"
  [ "$status" -ne 0 ]
}


# TODO: test creating symlinks
