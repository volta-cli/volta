use std::path::PathBuf;

use super::*;

fn fixture_path(fixture_dirs: &[&str]) -> PathBuf {
    let mut cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    cargo_manifest_dir.push("fixtures");

    for fixture_dir in fixture_dirs.iter() {
        cargo_manifest_dir.push(fixture_dir);
    }

    cargo_manifest_dir
}

mod find_closest_root {
    use super::*;

    #[test]
    fn test_find_closest_root_direct() {
        let base_dir = fixture_path(&["basic"]);
        let project_dir =
            find_closest_root(base_dir.clone()).expect("Failed to find project directory");

        assert_eq!(project_dir, base_dir);
    }

    #[test]
    fn test_find_closest_root_ancestor() {
        let base_dir = fixture_path(&["basic", "subdir"]);
        let project_dir = find_closest_root(base_dir).expect("Failed to find project directory");

        assert_eq!(project_dir, fixture_path(&["basic"]));
    }

    #[test]
    fn test_find_closest_root_dependency() {
        let base_dir = fixture_path(&["basic", "node_modules", "eslint"]);
        let project_dir = find_closest_root(base_dir).expect("Failed to find project directory");

        assert_eq!(project_dir, fixture_path(&["basic"]));
    }
}

mod project {
    use super::*;

    #[test]
    fn manifest_file() {
        let project_path = fixture_path(&["basic"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();

        let expected = fixture_path(&["basic", "package.json"]);
        assert_eq!(test_project.manifest_file(), &expected);
    }

    #[test]
    fn workspace_roots() {
        let project_path = fixture_path(&["nested", "subproject", "inner_project"]);
        let expected_base = project_path.clone();
        let test_project = Project::for_dir(project_path).unwrap().unwrap();

        let expected = vec![
            &*expected_base,
            expected_base.parent().unwrap(),
            expected_base.parent().unwrap().parent().unwrap(),
        ];

        assert_eq!(test_project.workspace_roots().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn platform_simple() {
        let project_path = fixture_path(&["basic"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();
        let platform = test_project.platform().unwrap();

        assert_eq!(platform.node, "6.11.1".parse().unwrap());
        assert_eq!(platform.npm, Some("3.10.10".parse().unwrap()));
        assert_eq!(platform.yarn, Some("1.2.0".parse().unwrap()));
    }

    #[test]
    fn platform_workspace() {
        let project_path = fixture_path(&["nested", "subproject", "inner_project"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();
        let platform = test_project.platform().unwrap();

        // From the top level `nested/package.json`
        assert_eq!(platform.node, "12.14.0".parse().unwrap());
        // From the middle project `nested/subproject/package.json`
        assert_eq!(platform.npm, Some("6.9.0".parse().unwrap()));
        // From the innermost project `nested/subproject/inner_project/package.json`
        assert_eq!(platform.yarn, Some("1.22.4".parse().unwrap()));
    }

    #[test]
    fn direct_dependencies_single() {
        let project_path = fixture_path(&["basic"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();

        // eslint, rsvp, bin-1, and bin-2 are direct dependencies
        assert!(test_project.has_direct_dependency("eslint"));
        assert!(test_project.has_direct_dependency("rsvp"));
        assert!(test_project.has_direct_dependency("@namespace/some-dep"));
        assert!(test_project.has_direct_dependency("@namespaced/something-else"));

        // typescript is not a direct dependency
        assert!(!test_project.has_direct_dependency("typescript"));
    }

    #[test]
    fn direct_dependencies_workspace() {
        let project_path = fixture_path(&["nested", "subproject", "inner_project"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();

        // express and typescript are direct dependencies of the innermost project
        assert!(test_project.has_direct_dependency("express"));
        assert!(test_project.has_direct_dependency("typescript"));
        // rsvp and glob are direct dependencies of the middle project
        assert!(test_project.has_direct_dependency("rsvp"));
        assert!(test_project.has_direct_dependency("glob"));
        // lodash and eslint are direct dependencies of the top-level workspace
        assert!(test_project.has_direct_dependency("lodash"));
        assert!(test_project.has_direct_dependency("eslint"));

        // react is not a direct dependency of any project
        assert!(!test_project.has_direct_dependency("react"));
    }

    #[test]
    fn find_bin_single() {
        let project_path = fixture_path(&["basic"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();

        assert_eq!(
            test_project.find_bin("rsvp"),
            Some(fixture_path(&["basic", "node_modules", ".bin", "rsvp"]))
        );

        assert!(test_project.find_bin("eslint").is_none());
    }

    #[test]
    fn find_bin_workspace() {
        // eslint, rsvp, tsc
        let project_path = fixture_path(&["nested", "subproject", "inner_project"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();

        // eslint is a binary in the root workspace
        assert_eq!(
            test_project.find_bin("eslint"),
            Some(fixture_path(&["nested", "node_modules", ".bin", "eslint"]))
        );

        // rsvp is a binary in the middle project
        assert_eq!(
            test_project.find_bin("rsvp"),
            Some(fixture_path(&[
                "nested",
                "subproject",
                "node_modules",
                ".bin",
                "rsvp"
            ]))
        );

        // tsc is a binary in the inner project
        assert_eq!(
            test_project.find_bin("tsc"),
            Some(fixture_path(&[
                "nested",
                "subproject",
                "inner_project",
                "node_modules",
                ".bin",
                "tsc"
            ]))
        );

        assert!(test_project.find_bin("ember").is_none());
    }

    #[test]
    fn detects_workspace_cycles() {
        // cycle-1 has a cycle with the original package.json
        let cycle_path = fixture_path(&["cycle-1"]);
        let project_error = Project::for_dir(cycle_path).unwrap_err();

        match project_error.kind() {
            ErrorKind::ExtensionCycleError { paths, duplicate } => {
                let expected_paths = vec![
                    fixture_path(&["cycle-1", "package.json"]),
                    fixture_path(&["cycle-1", "volta.json"]),
                ];
                assert_eq!(&expected_paths, paths);
                assert_eq!(&expected_paths[0], duplicate);
            }
            kind => panic!("Wrong error kind: {:?}", kind),
        }

        // cycle-2 has a cycle with 2 separate extensions, not including the original package.json
        let cycle_path = fixture_path(&["cycle-2"]);
        let project_error = Project::for_dir(cycle_path).unwrap_err();

        match project_error.kind() {
            ErrorKind::ExtensionCycleError { paths, duplicate } => {
                let expected_paths = vec![
                    fixture_path(&["cycle-2", "package.json"]),
                    fixture_path(&["cycle-2", "workspace-1.json"]),
                    fixture_path(&["cycle-2", "workspace-2.json"]),
                ];
                assert_eq!(&expected_paths, paths);
                assert_eq!(&expected_paths[1], duplicate);
            }
            kind => panic!("Wrong error kind: {:?}", kind),
        }
    }
}

mod needs_yarn_run {
    use super::*;

    #[test]
    fn project_does_not_need_yarn_run() {
        let project_path = fixture_path(&["basic"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();
        assert!(!test_project.needs_yarn_run());
    }

    #[test]
    fn project_has_yarnrc_yml() {
        let project_path = fixture_path(&["yarn", "yarnrc-yml"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();
        assert!(test_project.needs_yarn_run());
    }

    #[test]
    fn project_has_pnp_js() {
        let project_path = fixture_path(&["yarn", "pnp-js"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();
        assert!(test_project.needs_yarn_run());
    }

    #[test]
    fn project_has_pnp_cjs() {
        let project_path = fixture_path(&["yarn", "pnp-cjs"]);
        let test_project = Project::for_dir(project_path).unwrap().unwrap();
        assert!(test_project.needs_yarn_run());
    }
}
