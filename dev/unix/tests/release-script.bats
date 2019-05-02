# test the release.sh script

# load the functions from the script
source dev/unix/release.sh

# happy path test to parse the version from Cargo.toml
@test "parse_version - normal Cargo.toml" {
  input=$(cat <<'END_CARGO_TOML'
[package]
name = "notion"
version = "0.7.38"
authors = ["David Herman <david.herman@gmail.com>"]
license = "BSD-2-Clause"
END_CARGO_TOML
)

  expected_output="0.7.38"

  run parse_version "$input"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# it doesn't parse the version from other dependencies
@test "parse_version - error" {
  input=$(cat <<'END_CARGO_TOML'
[dependencies]
notion-core = { path = "crates/notion-core" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0.37"
console = "0.6.1"
END_CARGO_TOML
)

  expected_output=$(echo -e "\033[1;31mError\033[0m: Could not determine the current version")

  run parse_version "$input"
  [ "$status" -eq 1 ]
  diff <(echo "$output") <(echo "$expected_output")
}


# macos
@test "parse_os_info - macos" {
  expected_output="macos"

  run parse_os_info "Darwin" "this is ignored"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# linux - supported OpenSSL
@test "parse_os_info - linux with supported OpenSSL" {
  expected_output="linux-openssl-1.2.3"

  run parse_os_info "Linux" "OpenSSL 1.2.3a whatever else"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# linux - unsupported OpenSSL
@test "parse_os_info - linux with unsupported OpenSSL" {
  expected_output=$(echo -e "\033[1;31mError\033[0m: Releases for 'SomeSSL' not currently supported. Supported libraries are: OpenSSL.")

  run parse_os_info "Linux" "SomeSSL 1.2.3a whatever else"
  [ "$status" -eq 1 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# linux - unexpected OpenSSL version format
@test "parse_os_info - linux with unexpected OpenSSL format" {
  expected_output=$(echo -e "\033[1;31mError\033[0m: Could not determine OpenSSL version for 'Some SSL 1.2.4'. You probably need to update the regex to handle this output.")

  run parse_os_info "Linux" "Some SSL 1.2.4"
  [ "$status" -eq 1 ]
  diff <(echo "$output") <(echo "$expected_output")
}

# unsupported OS
@test "parse_os_info - unsupported OS" {
  expected_output=$(echo -e "\033[1;31mError\033[0m: Releases for 'DOS' are not yet supported. You will need to add another OS case to this script, and to the install script to support this OS.")

  run parse_os_info "DOS" "doesn't matter"
  [ "$status" -eq 1 ]
  diff <(echo "$output") <(echo "$expected_output")
}


# parsing valid OpenSSL version strings
@test "parse_openssl_version - valid versions" {
  expected_output="0.9.5"
  run parse_openssl_version "OpenSSL 0.9.5a 1 Apr 2000"
  [ "$status" -eq 0 ]
  diff <(echo "$output") <(echo "$expected_output")

  expected_output="1.0.1"
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
  expected_output=$(echo -e "\033[1;31mError\033[0m: Could not determine OpenSSL version for 'Some Weird Version 1.2.3'. You probably need to update the regex to handle this output.")
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
