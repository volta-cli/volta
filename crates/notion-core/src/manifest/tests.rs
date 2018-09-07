use manifest::Manifest;
use semver::Version;
use std::collections::HashMap;
use std::path::PathBuf;

fn fixture_path(fixture_dir: &str) -> PathBuf {
    let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    cargo_manifest_dir.push("fixtures");
    cargo_manifest_dir.push(fixture_dir);
    cargo_manifest_dir
}

#[test]
fn gets_node_version() {
    let project_path = fixture_path("basic");
    let version = Manifest::for_dir(&project_path)
        .expect("Could not get manifest")
        .node()
        .unwrap();
    assert_eq!(version, Version::parse("6.11.1").unwrap());
}

#[test]
fn gets_yarn_version() {
    let project_path = fixture_path("basic");
    let version = Manifest::for_dir(&project_path)
        .expect("Could not get manifest")
        .yarn();
    assert_eq!(version.unwrap(), Version::parse("1.2.0").unwrap());
}

#[test]
fn gets_dependencies() {
    let project_path = fixture_path("basic");
    let dependencies = Manifest::for_dir(&project_path)
        .expect("Could not get manifest")
        .dependencies;
    let mut expected_deps = HashMap::new();
    expected_deps.insert("@namespace/some-dep".to_string(), "0.2.4".to_string());
    expected_deps.insert("rsvp".to_string(), "^3.5.0".to_string());
    assert_eq!(dependencies, expected_deps);
}

#[test]
fn gets_dev_dependencies() {
    let project_path = fixture_path("basic");
    let dev_dependencies = Manifest::for_dir(&project_path)
        .expect("Could not get manifest")
        .dev_dependencies;
    let mut expected_deps = HashMap::new();
    expected_deps.insert(
        "@namespaced/something-else".to_string(),
        "^6.3.7".to_string(),
    );
    expected_deps.insert("eslint".to_string(), "~4.8.0".to_string());
    assert_eq!(dev_dependencies, expected_deps);
}

#[test]
fn node_for_no_toolchain() {
    let project_path = fixture_path("no_toolchain");
    let manifest = Manifest::for_dir(&project_path).expect("Could not get manifest");
    assert_eq!(manifest.node(), None);
}

#[test]
fn yarn_for_no_toolchain() {
    let project_path = fixture_path("no_toolchain");
    let manifest = Manifest::for_dir(&project_path).expect("Could not get manifest");
    assert_eq!(manifest.yarn(), None);
}

#[test]
fn gets_bin_map_format() {
    let project_path = fixture_path("basic/node_modules/eslint");
    let bin = Manifest::for_dir(&project_path)
        .expect("Could not get manifest")
        .bin;
    let mut expected_bin = HashMap::new();
    expected_bin.insert("eslint".to_string(), "./bin/eslint.js".to_string());
    assert_eq!(bin, expected_bin);
}

#[test]
fn gets_multiple_bins() {
    let project_path = fixture_path("basic/node_modules/typescript");
    let bin = Manifest::for_dir(&project_path)
        .expect("Could not get manifest")
        .bin;
    let mut expected_bin = HashMap::new();
    expected_bin.insert("tsc".to_string(), "./bin/tsc".to_string());
    expected_bin.insert("tsserver".to_string(), "./bin/tsserver".to_string());
    assert_eq!(bin, expected_bin);
}

#[test]
fn gets_bin_string_format() {
    let project_path = fixture_path("basic/node_modules/rsvp");
    let bin = Manifest::for_dir(&project_path)
        .expect("Could not get manifest")
        .bin;
    let mut expected_bin = HashMap::new();
    expected_bin.insert("rsvp".to_string(), "./bin/rsvp.js".to_string());
    assert_eq!(bin, expected_bin);
}

#[test]
fn handles_dep_with_no_bin() {
    let project_path = fixture_path("basic/node_modules/@namespace/some-dep");
    let bin = Manifest::for_dir(&project_path)
        .expect("Could not get manifest")
        .bin;
    let expected_bin = HashMap::new();
    assert_eq!(bin, expected_bin);
}
