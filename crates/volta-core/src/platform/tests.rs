use super::*;
use crate::layout::volta_home;
#[cfg(windows)]
use crate::layout::volta_install;
use node_semver::Version;
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
fn build_test_path() -> String {
    format!(
        "{}:/usr/bin:/bin",
        volta_home().unwrap().shim_dir().to_string_lossy()
    )
}

#[cfg(windows)]
fn build_test_path() -> String {
    let pathbufs = vec![
        volta_home().unwrap().shim_dir().to_owned(),
        PathBuf::from("C:\\\\somebin"),
        volta_install().unwrap().root().to_owned(),
        PathBuf::from("D:\\\\ProbramFlies"),
    ];
    std::env::join_paths(pathbufs.iter())
        .unwrap()
        .into_string()
        .expect("Could not create path containing shim dir")
}

fn test_image_path() {
    #[cfg(unix)]
    let path_delimiter = ":";
    #[cfg(windows)]
    let path_delimiter = ";";
    let path = build_test_path();
    std::env::set_var("PATH", &path);

    let node_bin = volta_home().unwrap().node_image_bin_dir("1.2.3");
    let expected_node_bin = node_bin.to_str().unwrap();

    let npm_bin = volta_home().unwrap().npm_image_bin_dir("6.4.3");
    let expected_npm_bin = npm_bin.to_str().unwrap();

    let pnpm_bin = volta_home().unwrap().pnpm_image_bin_dir("7.7.1");
    let expected_pnpm_bin = pnpm_bin.to_str().unwrap();

    let yarn_bin = volta_home().unwrap().yarn_image_bin_dir("4.5.7");
    let expected_yarn_bin = yarn_bin.to_str().unwrap();

    let v123 = Version::parse("1.2.3").unwrap();
    let v457 = Version::parse("4.5.7").unwrap();
    let v643 = Version::parse("6.4.3").unwrap();
    let v771 = Version::parse("7.7.1").unwrap();

    let only_node = Image {
        node: Sourced::with_default(v123.clone()),
        npm: None,
        pnpm: None,
        yarn: None,
    };

    assert_eq!(
        only_node.path().unwrap().into_string().unwrap(),
        [expected_node_bin, &path].join(path_delimiter)
    );

    let node_npm = Image {
        node: Sourced::with_default(v123.clone()),
        npm: Some(Sourced::with_default(v643.clone())),
        pnpm: None,
        yarn: None,
    };

    assert_eq!(
        node_npm.path().unwrap().into_string().unwrap(),
        [expected_npm_bin, expected_node_bin, &path].join(path_delimiter)
    );

    let node_pnpm = Image {
        node: Sourced::with_default(v123.clone()),
        npm: None,
        pnpm: Some(Sourced::with_default(v771.clone())),
        yarn: None,
    };

    assert_eq!(
        node_pnpm.path().unwrap().into_string().unwrap(),
        [expected_pnpm_bin, expected_node_bin, &path].join(path_delimiter)
    );

    let node_yarn = Image {
        node: Sourced::with_default(v123.clone()),
        npm: None,
        pnpm: None,
        yarn: Some(Sourced::with_default(v457.clone())),
    };

    assert_eq!(
        node_yarn.path().unwrap().into_string().unwrap(),
        [expected_yarn_bin, expected_node_bin, &path].join(path_delimiter)
    );

    let node_npm_pnpm = Image {
        node: Sourced::with_default(v123.clone()),
        npm: Some(Sourced::with_default(v643.clone())),
        pnpm: Some(Sourced::with_default(v771)),
        yarn: None,
    };

    assert_eq!(
        node_npm_pnpm.path().unwrap().into_string().unwrap(),
        [
            expected_npm_bin,
            expected_pnpm_bin,
            expected_node_bin,
            &path
        ]
        .join(path_delimiter)
    );

    let node_npm_yarn = Image {
        node: Sourced::with_default(v123),
        npm: Some(Sourced::with_default(v643)),
        pnpm: None,
        yarn: Some(Sourced::with_default(v457)),
    };

    assert_eq!(
        node_npm_yarn.path().unwrap().into_string().unwrap(),
        [
            expected_npm_bin,
            expected_yarn_bin,
            expected_node_bin,
            &path
        ]
        .join(path_delimiter)
    );
}

fn test_system_path() {
    let path = build_test_path();
    std::env::set_var("PATH", path);

    #[cfg(unix)]
    let expected_path = String::from("/usr/bin:/bin");
    #[cfg(windows)]
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
    use node_semver::Version;

    const NODE_VERSION: Version = Version {
        major: 12,
        minor: 14,
        patch: 1,
        build: Vec::new(),
        pre_release: Vec::new(),
    };
    const NPM_VERSION: Version = Version {
        major: 6,
        minor: 13,
        patch: 2,
        build: Vec::new(),
        pre_release: Vec::new(),
    };
    const YARN_VERSION: Version = Version {
        major: 1,
        minor: 17,
        patch: 0,
        build: Vec::new(),
        pre_release: Vec::new(),
    };

    mod merge {
        use super::super::super::*;
        use super::*;

        #[test]
        fn uses_node() {
            let test = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::default(),
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: None,
                pnpm: None,
                yarn: None,
            };

            let merged = test.merge(base);

            assert_eq!(merged.node.value, NODE_VERSION);
            assert_eq!(merged.node.source, Source::CommandLine);
        }

        #[test]
        fn inherits_node() {
            let test = CliPlatform {
                node: None,
                npm: InheritOption::default(),
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(NODE_VERSION),
                npm: None,
                pnpm: None,
                yarn: None,
            };

            let merged = test.merge(base);

            assert_eq!(merged.node.value, NODE_VERSION);
            assert_eq!(merged.node.source, Source::Default);
        }

        #[test]
        fn uses_npm() {
            let test = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::Some(NPM_VERSION),
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: Some(Sourced::with_default(Version::from((5, 6, 3)))),
                pnpm: None,
                yarn: None,
            };

            let merged = test.merge(base);

            let merged_npm = merged.npm.unwrap();
            assert_eq!(merged_npm.value, NPM_VERSION);
            assert_eq!(merged_npm.source, Source::CommandLine);
        }

        #[test]
        fn inherits_npm() {
            let test = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::Inherit,
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: Some(Sourced::with_default(NPM_VERSION)),
                pnpm: None,
                yarn: None,
            };

            let merged = test.merge(base);

            let merged_npm = merged.npm.unwrap();
            assert_eq!(merged_npm.value, NPM_VERSION);
            assert_eq!(merged_npm.source, Source::Default);
        }

        #[test]
        fn none_does_not_inherit_npm() {
            let test = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::None,
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: Some(Sourced::with_default(NPM_VERSION)),
                pnpm: None,
                yarn: None,
            };

            let merged = test.merge(base);

            assert!(merged.npm.is_none());
        }

        #[test]
        fn uses_yarn() {
            let test = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::default(),
                pnpm: InheritOption::default(),
                yarn: InheritOption::Some(YARN_VERSION),
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: None,
                pnpm: None,
                yarn: Some(Sourced::with_default(Version::from((1, 10, 3)))),
            };

            let merged = test.merge(base);

            let merged_yarn = merged.yarn.unwrap();
            assert_eq!(merged_yarn.value, YARN_VERSION);
            assert_eq!(merged_yarn.source, Source::CommandLine);
        }

        #[test]
        fn inherits_yarn() {
            let test = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::default(),
                pnpm: InheritOption::default(),
                yarn: InheritOption::Inherit,
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: None,
                pnpm: None,
                yarn: Some(Sourced::with_default(YARN_VERSION)),
            };

            let merged = test.merge(base);

            let merged_yarn = merged.yarn.unwrap();
            assert_eq!(merged_yarn.value, YARN_VERSION);
            assert_eq!(merged_yarn.source, Source::Default);
        }

        #[test]
        fn none_does_not_inherit_yarn() {
            let test = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::default(),
                pnpm: InheritOption::default(),
                yarn: InheritOption::None,
            };

            let base = Platform {
                node: Sourced::with_default(Version::from((10, 10, 10))),
                npm: None,
                pnpm: None,
                yarn: Some(Sourced::with_default(YARN_VERSION)),
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
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();

            assert!(transformed.is_none());
        }

        #[test]
        fn uses_cli_node() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::default(),
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();

            let node = transformed.unwrap().node;
            assert_eq!(node.value, NODE_VERSION);
            assert_eq!(node.source, Source::CommandLine);
        }

        #[test]
        fn uses_cli_npm() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::Some(NPM_VERSION),
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();

            let npm = transformed.unwrap().npm.unwrap();
            assert_eq!(npm.value, NPM_VERSION);
            assert_eq!(npm.source, Source::CommandLine);
        }

        #[test]
        fn no_npm() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::None,
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();

            assert!(transformed.unwrap().npm.is_none());
        }

        #[test]
        fn inherit_npm_becomes_none() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::Inherit,
                pnpm: InheritOption::default(),
                yarn: InheritOption::default(),
            };

            let transformed: Option<Platform> = cli.into();

            assert!(transformed.unwrap().npm.is_none());
        }

        #[test]
        fn uses_cli_yarn() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::default(),
                pnpm: InheritOption::default(),
                yarn: InheritOption::Some(YARN_VERSION),
            };

            let transformed: Option<Platform> = cli.into();

            let yarn = transformed.unwrap().yarn.unwrap();
            assert_eq!(yarn.value, YARN_VERSION);
            assert_eq!(yarn.source, Source::CommandLine);
        }

        #[test]
        fn no_yarn() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::default(),
                pnpm: InheritOption::default(),
                yarn: InheritOption::None,
            };

            let transformed: Option<Platform> = cli.into();

            assert!(transformed.unwrap().yarn.is_none());
        }

        #[test]
        fn inherit_yarn_becomes_none() {
            let cli = CliPlatform {
                node: Some(NODE_VERSION),
                npm: InheritOption::default(),
                pnpm: InheritOption::default(),
                yarn: InheritOption::Inherit,
            };

            let transformed: Option<Platform> = cli.into();

            assert!(transformed.unwrap().yarn.is_none());
        }
    }
}
