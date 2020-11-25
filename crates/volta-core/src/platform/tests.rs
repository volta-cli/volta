// Suppressing the redundant clone warning while the `pnpm` feature is active, as that makes it
// difficult to properly avoid redundant clones. This should be removed when the feature flag is
// disabled (#[cfg(feature = "pnpm")])
#![allow(clippy::redundant_clone)]

use super::*;
use crate::layout::volta_home;
#[cfg(windows)]
use crate::layout::volta_install;
use semver::Version;
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
    let starting_path = format!(
        "/usr/bin:/blah:{}:/doesnt/matter/bin",
        volta_home().unwrap().shim_dir().to_string_lossy()
    );
    std::env::set_var("PATH", &starting_path);

    let node_bin = volta_home().unwrap().node_image_bin_dir("1.2.3");
    let expected_node_bin = node_bin.to_str().unwrap();

    let npm_bin = volta_home().unwrap().npm_image_bin_dir("6.4.3");
    let expected_npm_bin = npm_bin.to_str().unwrap();

    #[cfg(feature = "pnpm")]
    let pnpm_bin = volta_home().unwrap().pnpm_image_bin_dir("5.1.3");
    #[cfg(feature = "pnpm")]
    let expected_pnpm_bin = pnpm_bin.to_str().unwrap();

    let yarn_bin = volta_home().unwrap().yarn_image_bin_dir("4.5.7");
    let expected_yarn_bin = yarn_bin.to_str().unwrap();

    let v123 = Version::parse("1.2.3").unwrap();
    let v457 = Version::parse("4.5.7").unwrap();
    #[cfg(feature = "pnpm")]
    let v513 = Version::parse("5.1.3").unwrap();
    let v643 = Version::parse("6.4.3").unwrap();

    let only_node = Image {
        node: Sourced::with_default(v123.clone()),
        npm: None,
        #[cfg(feature = "pnpm")]
        pnpm: None,
        yarn: None,
    };

    assert_eq!(
        only_node.path().unwrap().into_string().unwrap(),
        format!("{}:{}", expected_node_bin, starting_path)
    );

    let node_npm = Image {
        node: Sourced::with_default(v123.clone()),
        npm: Some(Sourced::with_default(v643.clone())),
        #[cfg(feature = "pnpm")]
        pnpm: None,
        yarn: None,
    };

    assert_eq!(
        node_npm.path().unwrap().into_string().unwrap(),
        format!(
            "{}:{}:{}",
            expected_npm_bin, expected_node_bin, starting_path
        )
    );

    #[cfg(feature = "pnpm")]
    {
        let node_pnpm = Image {
            node: Sourced::with_default(v123.clone()),
            npm: None,
            pnpm: Some(Sourced::with_default(v513.clone())),
            yarn: None,
        };

        assert_eq!(
            node_pnpm.path().unwrap().into_string().unwrap(),
            format!(
                "{}:{}:{}",
                expected_pnpm_bin, expected_node_bin, starting_path
            )
        );
    }

    let node_yarn = Image {
        node: Sourced::with_default(v123.clone()),
        npm: None,
        #[cfg(feature = "pnpm")]
        pnpm: None,
        yarn: Some(Sourced::with_default(v457.clone())),
    };

    assert_eq!(
        node_yarn.path().unwrap().into_string().unwrap(),
        format!(
            "{}:{}:{}",
            expected_yarn_bin, expected_node_bin, starting_path
        )
    );

    let node_npm_yarn = Image {
        node: Sourced::with_default(v123.clone()),
        npm: Some(Sourced::with_default(v643.clone())),
        #[cfg(feature = "pnpm")]
        pnpm: None,
        yarn: Some(Sourced::with_default(v457.clone())),
    };

    assert_eq!(
        node_npm_yarn.path().unwrap().into_string().unwrap(),
        format!(
            "{}:{}:{}:{}",
            expected_npm_bin, expected_yarn_bin, expected_node_bin, starting_path
        )
    );

    #[cfg(feature = "pnpm")]
    {
        let all = Image {
            node: Sourced::with_default(v123.clone()),
            npm: Some(Sourced::with_default(v643.clone())),
            pnpm: Some(Sourced::with_default(v513.clone())),
            yarn: Some(Sourced::with_default(v457.clone())),
        };

        assert_eq!(
            all.path().unwrap().into_string().unwrap(),
            format!(
                "{}:{}:{}:{}:{}",
                expected_npm_bin,
                expected_pnpm_bin,
                expected_yarn_bin,
                expected_node_bin,
                starting_path
            )
        );
    }
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

    std::env::set_var("PATH", &path_with_shims);

    let node_bin = volta_home().unwrap().node_image_bin_dir("1.2.3");
    let expected_node_bin = node_bin.to_str().unwrap();

    let npm_bin = volta_home().unwrap().npm_image_bin_dir("6.4.3");
    let expected_npm_bin = npm_bin.to_str().unwrap();

    #[cfg(feature = "pnpm")]
    let pnpm_bin = volta_home().unwrap().pnpm_image_bin_dir("5.1.3");
    #[cfg(feature = "pnpm")]
    let expected_pnpm_bin = pnpm_bin.to_str().unwrap();

    let yarn_bin = volta_home().unwrap().yarn_image_bin_dir("4.5.7");
    let expected_yarn_bin = yarn_bin.to_str().unwrap();

    let v123 = Version::parse("1.2.3").unwrap();
    let v457 = Version::parse("4.5.7").unwrap();
    #[cfg(feature = "pnpm")]
    let v513 = Version::parse("5.1.3").unwrap();
    let v643 = Version::parse("6.4.3").unwrap();

    let only_node = Image {
        node: Sourced::with_default(v123.clone()),
        npm: None,
        #[cfg(feature = "pnpm")]
        pnpm: None,
        yarn: None,
    };

    assert_eq!(
        only_node.path().unwrap().into_string().unwrap(),
        format!("{};{}", expected_node_bin, path_with_shims),
    );

    let node_npm = Image {
        node: Sourced::with_default(v123.clone()),
        npm: Some(Sourced::with_default(v643.clone())),
        #[cfg(feature = "pnpm")]
        pnpm: None,
        yarn: None,
    };

    assert_eq!(
        node_npm.path().unwrap().into_string().unwrap(),
        format!(
            "{};{};{}",
            expected_npm_bin, expected_node_bin, path_with_shims
        )
    );

    #[cfg(feature = "pnpm")]
    {
        let node_pnpm = Image {
            node: Sourced::with_default(v123.clone()),
            npm: None,
            pnpm: Some(Sourced::with_default(v513.clone())),
            yarn: None,
        };

        assert_eq!(
            node_pnpm.path().unwrap().into_string().unwrap(),
            format!(
                "{};{};{}",
                expected_pnpm_bin, expected_node_bin, path_with_shims
            )
        );
    }

    let node_yarn = Image {
        node: Sourced::with_default(v123.clone()),
        npm: None,
        #[cfg(feature = "pnpm")]
        pnpm: None,
        yarn: Some(Sourced::with_default(v457.clone())),
    };

    assert_eq!(
        node_yarn.path().unwrap().into_string().unwrap(),
        format!(
            "{};{};{}",
            expected_yarn_bin, expected_node_bin, path_with_shims
        )
    );

    let node_npm_yarn = Image {
        node: Sourced::with_default(v123.clone()),
        npm: Some(Sourced::with_default(v643.clone())),
        #[cfg(feature = "pnpm")]
        pnpm: None,
        yarn: Some(Sourced::with_default(v457.clone())),
    };

    assert_eq!(
        node_npm_yarn.path().unwrap().into_string().unwrap(),
        format!(
            "{};{};{};{}",
            expected_npm_bin, expected_yarn_bin, expected_node_bin, path_with_shims
        )
    );

    #[cfg(feature = "pnpm")]
    {
        let all = Image {
            node: Sourced::with_default(v123.clone()),
            npm: Some(Sourced::with_default(v643.clone())),
            pnpm: Some(Sourced::with_default(v513.clone())),
            yarn: Some(Sourced::with_default(v457.clone())),
        };

        assert_eq!(
            all.path().unwrap().into_string().unwrap(),
            format!(
                "{};{};{};{};{}",
                expected_npm_bin,
                expected_pnpm_bin,
                expected_yarn_bin,
                expected_node_bin,
                path_with_shims
            )
        );
    }
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

mod inherit_option {
    mod map {
        use super::super::super::*;

        #[test]
        fn converts_some_value() {
            let opt = InheritOption::Some(1);

            assert_eq!(opt.map(|n| n + 1), InheritOption::Some(2));
        }

        #[test]
        fn leaves_none() {
            let opt: InheritOption<i32> = InheritOption::None;

            assert_eq!(opt.map(|n| n + 1), InheritOption::None);
        }

        #[test]
        fn leaves_inherit() {
            let opt: InheritOption<i32> = InheritOption::Inherit;

            assert_eq!(opt.map(|n| n + 1), InheritOption::Inherit);
        }
    }

    mod inherit {
        use super::super::super::*;

        #[test]
        fn keeps_some_value() {
            let opt = InheritOption::Some(1);

            assert_eq!(opt.inherit(Some(2)), Some(1));
        }

        #[test]
        fn leaves_none() {
            let opt = InheritOption::None;

            assert_eq!(opt.inherit(Some(2)), None);
        }

        #[test]
        fn inherits_from_base() {
            let opt = InheritOption::Inherit;

            assert_eq!(opt.inherit(Some(2)), Some(2));
        }
    }
}

mod cli_platform {
    use lazy_static::lazy_static;
    use semver::Version;

    lazy_static! {
        static ref NODE_VERSION: Version = Version::from((12, 14, 1));
        static ref NPM_VERSION: Version = Version::from((6, 13, 2));
        static ref PNPM_VERSION: Version = Version::from((5, 2, 15));
        static ref YARN_VERSION: Version = Version::from((1, 17, 0));
    }

    mod merge {
        use super::super::super::*;
        use super::*;

        #[test]
        fn uses_node() {
            let test = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: None,
                #[cfg(feature = "pnpm")]
                pnpm: None,
                yarn: None,
            };

            let merged = test.merge(base);

            assert_eq!(merged.node.value, NODE_VERSION.clone());
            assert_eq!(merged.node.source, Source::CommandLine);
        }

        #[test]
        fn inherits_node() {
            let test = CliPlatform {
                node: None,
                npm: InheritOption::default(),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(NODE_VERSION.clone()),
                npm: None,
                #[cfg(feature = "pnpm")]
                pnpm: None,
                yarn: None,
            };

            let merged = test.merge(base);

            assert_eq!(merged.node.value, NODE_VERSION.clone());
            assert_eq!(merged.node.source, Source::Default);
        }

        #[test]
        fn uses_npm() {
            let test = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::Some(NPM_VERSION.clone()),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: Some(Sourced::with_default(Version::from((5, 6, 3)))),
                #[cfg(feature = "pnpm")]
                pnpm: None,
                yarn: None,
            };

            let merged = test.merge(base);

            let merged_npm = merged.npm.unwrap();
            assert_eq!(merged_npm.value, NPM_VERSION.clone());
            assert_eq!(merged_npm.source, Source::CommandLine);
        }

        #[test]
        fn inherits_npm() {
            let test = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::Inherit,
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: Some(Sourced::with_default(NPM_VERSION.clone())),
                #[cfg(feature = "pnpm")]
                pnpm: None,
                yarn: None,
            };

            let merged = test.merge(base);

            let merged_npm = merged.npm.unwrap();
            assert_eq!(merged_npm.value, NPM_VERSION.clone());
            assert_eq!(merged_npm.source, Source::Default);
        }

        #[test]
        fn none_does_not_inherit_npm() {
            let test = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::None,
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: Some(Sourced::with_default(NPM_VERSION.clone())),
                #[cfg(feature = "pnpm")]
                pnpm: None,
                yarn: None,
            };

            let merged = test.merge(base);

            assert!(merged.npm.is_none());
        }

        #[test]
        #[cfg(feature = "pnpm")]
        fn uses_pnpm() {
            let test = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                pnpm: InheritOption::Some(PNPM_VERSION.clone()),
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: None,
                pnpm: Some(Sourced::with_default(Version::from((1, 10, 3)))),
                yarn: None,
            };

            let merged = test.merge(base);

            let merged_pnpm = merged.pnpm.unwrap();
            assert_eq!(merged_pnpm.value, PNPM_VERSION.clone());
            assert_eq!(merged_pnpm.source, Source::CommandLine);
        }

        #[test]
        #[cfg(feature = "pnpm")]
        fn inherits_pnpm() {
            let test = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                pnpm: InheritOption::Inherit,
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: None,
                pnpm: Some(Sourced::with_default(PNPM_VERSION.clone())),
                yarn: None,
            };

            let merged = test.merge(base);

            let merged_pnpm = merged.pnpm.unwrap();
            assert_eq!(merged_pnpm.value, PNPM_VERSION.clone());
            assert_eq!(merged_pnpm.source, Source::Default);
        }

        #[test]
        #[cfg(feature = "pnpm")]
        fn none_does_not_inherit_pnpm() {
            let test = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                pnpm: InheritOption::None,
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: None,
                pnpm: Some(Sourced::with_default(PNPM_VERSION.clone())),
                yarn: None,
            };

            let merged = test.merge(base);

            assert!(merged.pnpm.is_none());
        }

        #[test]
        fn uses_yarn() {
            let test = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::Some(YARN_VERSION.clone()),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: None,
                #[cfg(feature = "pnpm")]
                pnpm: None,
                yarn: Some(Sourced::with_default(Version::from((1, 10, 3)))),
            };

            let merged = test.merge(base);

            let merged_yarn = merged.yarn.unwrap();
            assert_eq!(merged_yarn.value, YARN_VERSION.clone());
            assert_eq!(merged_yarn.source, Source::CommandLine);
        }

        #[test]
        fn inherits_yarn() {
            let test = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::Inherit,
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: None,
                #[cfg(feature = "pnpm")]
                pnpm: None,
                yarn: Some(Sourced::with_default(YARN_VERSION.clone())),
            };

            let merged = test.merge(base);

            let merged_yarn = merged.yarn.unwrap();
            assert_eq!(merged_yarn.value, YARN_VERSION.clone());
            assert_eq!(merged_yarn.source, Source::Default);
        }

        #[test]
        fn none_does_not_inherit_yarn() {
            let test = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::None,
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: None,
                #[cfg(feature = "pnpm")]
                pnpm: None,
                yarn: Some(Sourced::with_default(YARN_VERSION.clone())),
            };

            let merged = test.merge(base);

            assert!(merged.yarn.is_none());
        }
    }

    mod into_platform {
        use super::super::super::*;
        use super::*;

        #[test]
        fn none_if_no_node() {
            let cli = CliPlatform {
                node: None,
                npm: InheritOption::default(),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();

            assert!(transformed.is_none());
        }

        #[test]
        fn uses_cli_node() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();

            let node = transformed.unwrap().node;
            assert_eq!(node.value, NODE_VERSION.clone());
            assert_eq!(node.source, Source::CommandLine);
        }

        #[test]
        fn uses_cli_npm() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::Some(NPM_VERSION.clone()),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();

            let npm = transformed.unwrap().npm.unwrap();
            assert_eq!(npm.value, NPM_VERSION.clone());
            assert_eq!(npm.source, Source::CommandLine);
        }

        #[test]
        fn no_npm() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::None,
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();

            assert!(transformed.unwrap().npm.is_none());
        }

        #[test]
        fn inherit_npm_becomes_none() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::Inherit,
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();

            assert!(transformed.unwrap().npm.is_none());
        }

        #[test]
        #[cfg(feature = "pnpm")]
        fn uses_cli_pnpm() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                pnpm: InheritOption::Some(PNPM_VERSION.clone()),
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();

            let pnpm = transformed.unwrap().pnpm.unwrap();
            assert_eq!(pnpm.value, PNPM_VERSION.clone());
            assert_eq!(pnpm.source, Source::CommandLine);
        }

        #[test]
        #[cfg(feature = "pnpm")]
        fn no_pnpm() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                pnpm: InheritOption::None,
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();
            assert!(transformed.unwrap().pnpm.is_none());
        }

        #[test]
        #[cfg(feature = "pnpm")]
        fn inherit_pnpm_becomes_none() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                pnpm: InheritOption::Inherit,
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();
            assert!(transformed.unwrap().pnpm.is_none());
        }

        #[test]
        fn uses_cli_yarn() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::Some(YARN_VERSION.clone()),
            };

            let transformed: Option<Platform> = cli.into();

            let yarn = transformed.unwrap().yarn.unwrap();
            assert_eq!(yarn.value, YARN_VERSION.clone());
            assert_eq!(yarn.source, Source::CommandLine);
        }

        #[test]
        fn no_yarn() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::None,
            };

            let transformed: Option<Platform> = cli.into();

            assert!(transformed.unwrap().yarn.is_none());
        }

        #[test]
        fn inherit_yarn_becomes_none() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION.clone()),
                npm: InheritOption::default(),
                #[cfg(feature = "pnpm")]
                pnpm: InheritOption::default(),
                yarn: InheritOption::Inherit,
            };

            let transformed: Option<Platform> = cli.into();

            assert!(transformed.unwrap().yarn.is_none());
        }
    }
}
