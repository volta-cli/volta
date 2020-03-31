use super::*;
use crate::layout::volta_home;
#[cfg(windows)]
use crate::layout::volta_install;
use semver::Version;
use std;
#[cfg(windows)]
use std::path::PathBuf;

// Since unit tests are run in parallel, tests that modify the PATH environment variable are subject to race conditions
// To prevent that, ensure that all tests that rely on PATH are run in serial by adding them to this meta-test
#[test]
fn test_paths() {
    test_image_path();
    test_system_path();
}

#[cfg(unix)]
fn test_image_path() {
    std::env::set_var(
        "PATH",
        format!(
            "/usr/bin:/blah:{}:/doesnt/matter/bin",
            volta_home().unwrap().shim_dir().to_string_lossy()
        ),
    );

    let node_bin = volta_home()
        .unwrap()
        .root()
        .join("tools")
        .join("image")
        .join("node")
        .join("1.2.3")
        .join("6.4.3")
        .join("bin");
    let expected_node_bin = node_bin.as_path().to_str().unwrap();

    let yarn_bin = volta_home()
        .unwrap()
        .root()
        .join("tools")
        .join("image")
        .join("yarn")
        .join("4.5.7")
        .join("bin");
    let expected_yarn_bin = yarn_bin.as_path().to_str().unwrap();

    let v123 = Version::parse("1.2.3").unwrap();
    let v457 = Version::parse("4.5.7").unwrap();
    let v643 = Version::parse("6.4.3").unwrap();

    let no_yarn_image = Image {
        node: Sourced::with_default(v123.clone()),
        npm: Sourced::with_default(v643.clone()),
        yarn: None,
    };

    assert_eq!(
        no_yarn_image.path().unwrap().into_string().unwrap(),
        format!("{}:/usr/bin:/blah:/doesnt/matter/bin", expected_node_bin),
    );

    let with_yarn_image = Image {
        node: Sourced::with_default(v123),
        npm: Sourced::with_default(v643),
        yarn: Some(Sourced::with_default(v457)),
    };

    assert_eq!(
        with_yarn_image.path().unwrap().into_string().unwrap(),
        format!(
            "{}:{}:/usr/bin:/blah:/doesnt/matter/bin",
            expected_node_bin, expected_yarn_bin
        ),
    );
}

#[cfg(windows)]
fn test_image_path() {
    let mut pathbufs: Vec<PathBuf> = Vec::new();
    pathbufs.push(volta_home().unwrap().shim_dir().to_owned());
    pathbufs.push(PathBuf::from("C:\\\\somebin"));
    pathbufs.push(volta_install().unwrap().root().to_owned());
    pathbufs.push(PathBuf::from("D:\\\\ProbramFlies"));

    let path_with_shims = std::env::join_paths(pathbufs.iter())
        .unwrap()
        .into_string()
        .expect("Could not create path containing shim dir");

    std::env::set_var("PATH", path_with_shims);

    let node_bin = volta_home()
        .unwrap()
        .root()
        .join("tools")
        .join("image")
        .join("node")
        .join("1.2.3")
        .join("6.4.3");
    let expected_node_bin = node_bin.as_path().to_str().unwrap();

    let yarn_bin = volta_home()
        .unwrap()
        .root()
        .join("tools")
        .join("image")
        .join("yarn")
        .join("4.5.7")
        .join("bin");
    let expected_yarn_bin = yarn_bin.as_path().to_str().unwrap();

    let v123 = Version::parse("1.2.3").unwrap();
    let v457 = Version::parse("4.5.7").unwrap();
    let v643 = Version::parse("6.4.3").unwrap();

    let no_yarn_image = Image {
        node: Sourced::with_default(v123.clone()),
        npm: Sourced::with_default(v643.clone()),
        yarn: None,
    };

    assert_eq!(
        no_yarn_image.path().unwrap().into_string().unwrap(),
        format!("{};C:\\\\somebin;D:\\\\ProbramFlies", expected_node_bin),
    );

    let with_yarn_image = Image {
        node: Sourced::with_default(v123),
        npm: Sourced::with_default(v643),
        yarn: Some(Sourced::with_default(v457)),
    };

    assert_eq!(
        with_yarn_image.path().unwrap().into_string().unwrap(),
        format!(
            "{};{};C:\\\\somebin;D:\\\\ProbramFlies",
            expected_node_bin, expected_yarn_bin
        ),
    );
}

#[cfg(unix)]
fn test_system_path() {
    std::env::set_var(
        "PATH",
        format!(
            "{}:/usr/bin:/bin",
            volta_home().unwrap().shim_dir().to_string_lossy()
        ),
    );

    let expected_path = String::from("/usr/bin:/bin");

    assert_eq!(
        System::path().unwrap().into_string().unwrap(),
        expected_path
    );
}

#[cfg(windows)]
fn test_system_path() {
    let mut pathbufs: Vec<PathBuf> = Vec::new();
    pathbufs.push(volta_home().unwrap().shim_dir().to_owned());
    pathbufs.push(PathBuf::from("C:\\\\somebin"));
    pathbufs.push(volta_install().unwrap().root().to_owned());
    pathbufs.push(PathBuf::from("D:\\\\ProbramFlies"));

    let path_with_shims = std::env::join_paths(pathbufs.iter())
        .unwrap()
        .into_string()
        .expect("Could not create path containing shim dir");

    std::env::set_var("PATH", path_with_shims);

    let expected_path = String::from("C:\\\\somebin;D:\\\\ProbramFlies");

    assert_eq!(
        System::path().unwrap().into_string().unwrap(),
        expected_path
    );
}
