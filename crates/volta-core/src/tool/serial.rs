use std::cmp::Ordering;
use std::str::FromStr;

use super::Spec;
use crate::error::ErrorDetails;
use crate::version::VersionSpec;
use lazy_static::lazy_static;
use regex::Regex;
use validate_npm_package_name::{validate, Validity};
use volta_fail::Fallible;

lazy_static! {
    static ref TOOL_SPEC_PATTERN: Regex =
        Regex::new("^(?P<name>(?:@([^/]+?)[/])?([^/]+?))(@(?P<version>.+))?$")
            .expect("regex is valid");
    static ref HAS_VERSION: Regex = Regex::new(r"^[^\s]+@").expect("regex is valid");
}

/// Methods for parsing a Spec out of string values
impl Spec {
    pub fn from_str_and_version(tool_name: &str, version: VersionSpec) -> Self {
        match tool_name {
            "node" => Spec::Node(version),
            "npm" => Spec::Npm(version),
            "yarn" => Spec::Yarn(version),
            package => Spec::Package(package.to_string(), version),
        }
    }

    /// Try to parse a tool and version from a string like `<tool>[@<version>].
    pub fn try_from_str(tool_spec: &str) -> Fallible<Self> {
        let captures =
            TOOL_SPEC_PATTERN
                .captures(tool_spec)
                .ok_or(ErrorDetails::ParseToolSpecError {
                    tool_spec: tool_spec.into(),
                })?;

        // Validate that the captured name is a valid NPM package name.
        let name = &captures["name"];
        if let Validity::Invalid { errors, .. } = validate(name) {
            return Err(ErrorDetails::InvalidToolName {
                name: name.into(),
                errors,
            }
            .into());
        }

        let version = captures
            .name("version")
            .map(|version| VersionSpec::parse(version.as_str()))
            .transpose()?
            .unwrap_or_default();

        Ok(match name {
            "node" => Spec::Node(version),
            "npm" => Spec::Npm(version),
            "yarn" => Spec::Yarn(version),
            package => Spec::Package(package.into(), version),
        })
    }

    /// Get a valid, sorted `Vec<Spec>` given a `Vec<String>`.
    ///
    /// Accounts for the following error conditions:
    ///
    /// - `volta install node 12`, where the user intended to install `node@12`
    ///   but used syntax like in nodenv or nvm
    /// - invalid version specs
    ///
    /// Returns a listed sorted so that if `node` is included in the list, it is
    /// always first.
    pub fn from_strings<T>(tool_strs: &[T], action: &str) -> Fallible<Vec<Spec>>
    where
        T: AsRef<str>,
    {
        Self::check_args(tool_strs, action)?;

        let mut tools = tool_strs
            .iter()
            .map(|arg| Self::try_from_str(arg.as_ref()))
            .collect::<Fallible<Vec<Spec>>>()?;

        tools.sort_by(Self::sort_comparator);
        Ok(tools)
    }

    /// Check the args for the bad pattern of `volta install <tool> <number>`.
    fn check_args<T>(args: &[T], action: &str) -> Fallible<()>
    where
        T: AsRef<str>,
    {
        let mut args = args.iter();

        // The case we are concerned with is where we have `<tool> <number>`.
        // This is only interesting if there are exactly two args. Then we care
        // whether the two items are a bare name (with no `@version`), followed
        // by a valid version specifier. That is:
        //
        // - `volta install node@lts latest` is allowed.
        // - `volta install node latest` is an error.
        // - `volta install node latest yarn` is allowed.
        if let (Some(name), Some(maybe_version), None) = (args.next(), args.next(), args.next()) {
            if !HAS_VERSION.is_match(name.as_ref())
                && VersionSpec::from_str(maybe_version.as_ref()).is_ok()
            {
                return Err(ErrorDetails::InvalidInvocation {
                    action: action.to_string(),
                    name: name.as_ref().to_string(),
                    version: maybe_version.as_ref().to_string(),
                }
                .into());
            }
        }

        Ok(())
    }

    /// Compare `Spec`s for sorting when converting from strings
    ///
    /// We want to preserve the original order as much as possible, so we treat tools in
    /// the same tool category as equal. We still need to pull Node to the front of the
    /// list, followed by Npm / Yarn, and then Packages last.
    fn sort_comparator(left: &Spec, right: &Spec) -> Ordering {
        match (left, right) {
            (Spec::Node(_), Spec::Node(_)) => Ordering::Equal,
            (Spec::Node(_), _) => Ordering::Less,
            (_, Spec::Node(_)) => Ordering::Greater,
            (Spec::Npm(_), Spec::Npm(_)) => Ordering::Equal,
            (Spec::Npm(_), _) => Ordering::Less,
            (_, Spec::Npm(_)) => Ordering::Greater,
            (Spec::Yarn(_), Spec::Yarn(_)) => Ordering::Equal,
            (Spec::Yarn(_), _) => Ordering::Less,
            (_, Spec::Yarn(_)) => Ordering::Greater,
            (Spec::Package(_, _), Spec::Package(_, _)) => Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    mod try_from_str {
        use std::str::FromStr as _;

        use super::super::super::Spec;
        use crate::version::VersionSpec;

        const LTS: &str = "lts";
        const LATEST: &str = "latest";
        const MAJOR: &str = "3";
        const MINOR: &str = "3.0";
        const PATCH: &str = "3.0.0";

        /// Convenience macro for generating the <tool>@<version> string.
        macro_rules! versioned_tool {
            ($tool:expr, $version:expr) => {
                format!("{}@{}", $tool, $version)
            };
        }

        #[test]
        fn parses_bare_node() {
            assert_eq!(
                Spec::try_from_str("node").expect("succeeds"),
                Spec::Node(VersionSpec::default())
            );
        }

        #[test]
        fn parses_node_with_valid_versions() {
            let tool = "node";

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(tool, MAJOR)).expect("succeeds"),
                Spec::Node(VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests"))
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(tool, MINOR)).expect("succeeds"),
                Spec::Node(VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests"))
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(tool, PATCH)).expect("succeeds"),
                Spec::Node(VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests"))
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(tool, LATEST)).expect("succeeds"),
                Spec::Node(VersionSpec::Latest)
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(tool, LTS)).expect("succeeds"),
                Spec::Node(VersionSpec::Lts)
            );
        }

        #[test]
        fn parses_bare_yarn() {
            assert_eq!(
                Spec::try_from_str("yarn").expect("succeeds"),
                Spec::Yarn(VersionSpec::default())
            );
        }

        #[test]
        fn parses_yarn_with_valid_versions() {
            let tool = "yarn";

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(tool, MAJOR)).expect("succeeds"),
                Spec::Yarn(VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests"))
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(tool, MINOR)).expect("succeeds"),
                Spec::Yarn(VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests"))
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(tool, PATCH)).expect("succeeds"),
                Spec::Yarn(VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests"))
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(tool, LATEST)).expect("succeeds"),
                Spec::Yarn(VersionSpec::Latest)
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(tool, LTS)).expect("succeeds"),
                Spec::Yarn(VersionSpec::Lts)
            );
        }

        #[test]
        fn parses_bare_packages() {
            let package = "ember-cli";
            assert_eq!(
                Spec::try_from_str(package).expect("succeeds"),
                Spec::Package(package.into(), VersionSpec::default())
            );
        }

        #[test]
        fn parses_namespaced_packages() {
            let package = "@types/lodash";
            assert_eq!(
                Spec::try_from_str(package).expect("succeeds"),
                Spec::Package(package.into(), VersionSpec::default())
            );
        }

        #[test]
        fn parses_bare_packages_with_valid_versions() {
            let package = "something-awesome";

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(package, MAJOR)).expect("succeeds"),
                Spec::Package(
                    package.into(),
                    VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(package, MINOR)).expect("succeeds"),
                Spec::Package(
                    package.into(),
                    VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(package, PATCH)).expect("succeeds"),
                Spec::Package(
                    package.into(),
                    VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(package, LATEST)).expect("succeeds"),
                Spec::Package(package.into(), VersionSpec::Latest)
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(package, LTS)).expect("succeeds"),
                Spec::Package(package.into(), VersionSpec::Lts)
            );
        }

        #[test]
        fn parses_namespaced_packages_with_valid_versions() {
            let package = "@something/awesome";

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(package, MAJOR)).expect("succeeds"),
                Spec::Package(
                    package.into(),
                    VersionSpec::from_str(MAJOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(package, MINOR)).expect("succeeds"),
                Spec::Package(
                    package.into(),
                    VersionSpec::from_str(MINOR).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(package, PATCH)).expect("succeeds"),
                Spec::Package(
                    package.into(),
                    VersionSpec::from_str(PATCH).expect("`VersionSpec` has its own tests")
                )
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(package, LATEST)).expect("succeeds"),
                Spec::Package(package.into(), VersionSpec::Latest)
            );

            assert_eq!(
                Spec::try_from_str(&versioned_tool!(package, LTS)).expect("succeeds"),
                Spec::Package(package.into(), VersionSpec::Lts)
            );
        }
    }

    mod from_strings {
        use super::super::*;

        static PIN: &'static str = "pin";

        #[test]
        fn special_cases_tool_space_number() {
            let name = "potato";
            let version = "1.2.3";
            let args: Vec<String> = vec![name.into(), version.into()];

            let err = Spec::from_strings(&args, PIN).unwrap_err();
            let inner_err = err
                .downcast_ref::<ErrorDetails>()
                .expect("should be an ErrorDetails");

            assert_eq!(
                inner_err,
                &ErrorDetails::InvalidInvocation {
                    action: PIN.into(),
                    name: name.into(),
                    version: version.into()
                },
                "`volta <action> tool number` results in the correct error"
            );
        }

        #[test]
        fn leaves_other_scenarios_alone() {
            let empty: Vec<&str> = Vec::new();
            assert_eq!(
                Spec::from_strings(&empty, PIN).expect("is ok").len(),
                empty.len(),
                "when there are no args"
            );

            let only_one = ["node".to_owned()];
            assert_eq!(
                Spec::from_strings(&only_one, PIN).expect("is ok").len(),
                only_one.len(),
                "when there is only one arg"
            );

            let two_but_unmistakable = ["12".to_owned(), "node".to_owned()];
            assert_eq!(
                Spec::from_strings(&two_but_unmistakable, PIN.into())
                    .expect("is ok")
                    .len(),
                two_but_unmistakable.len(),
                "when there are two args but the order is not likely to be a mistake"
            );

            let two_but_valid_first = ["node@lts".to_owned(), "12".to_owned()];
            assert_eq!(
                Spec::from_strings(&two_but_valid_first, PIN.into())
                    .expect("is ok")
                    .len(),
                two_but_valid_first.len(),
                "when there are two args but the first is a valid tool spec"
            );

            let more_than_two_tools = ["node".to_owned(), "12".to_owned(), "yarn".to_owned()];
            assert_eq!(
                Spec::from_strings(&more_than_two_tools, PIN.into())
                    .expect("is ok")
                    .len(),
                more_than_two_tools.len(),
                "when there are more than two args"
            );
        }

        #[test]
        fn sorts_node_npm_yarn_to_front() {
            let multiple = [
                "ember-cli@3".to_owned(),
                "yarn".to_owned(),
                "npm@5".to_owned(),
                "node@latest".to_owned(),
            ];
            let expected = [
                Spec::Node(VersionSpec::Latest),
                Spec::Npm(VersionSpec::from_str("5").expect("requirement is valid")),
                Spec::Yarn(VersionSpec::default()),
                Spec::Package(
                    "ember-cli".to_owned(),
                    VersionSpec::from_str("3").expect("requirement is valid"),
                ),
            ];
            assert_eq!(
                Spec::from_strings(&multiple, PIN.into()).expect("is ok"),
                expected
            );
        }

        #[test]
        fn keeps_package_order_unchanged() {
            let packages_with_node = ["typescript@latest", "ember-cli@3", "node@lts", "mocha"];
            let expected = [
                Spec::Node(VersionSpec::Lts),
                Spec::Package("typescript".to_owned(), VersionSpec::Latest),
                Spec::Package(
                    "ember-cli".to_owned(),
                    VersionSpec::from_str("3").expect("requirement is valid"),
                ),
                Spec::Package("mocha".to_owned(), VersionSpec::default()),
            ];

            assert_eq!(
                Spec::from_strings(&packages_with_node, PIN.into()).expect("is ok"),
                expected
            );
        }
    }
}
