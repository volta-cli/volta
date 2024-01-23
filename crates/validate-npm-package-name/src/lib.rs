//! A Rust implementation of the validation rules from the core JS package
//! [`validate-npm-package-name`](https://github.com/npm/validate-npm-package-name/).

use once_cell::sync::Lazy;
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use regex::Regex;

/// The set of characters to encode, matching the characters encoded by
/// [`encodeURIComponent`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURIComponent#description)
static ENCODE_URI_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

static SCOPED_PACKAGE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:@([^/]+?)[/])?([^/]+?)$").expect("regex is valid"));
static SPECIAL_CHARS: Lazy<Regex> = Lazy::new(|| Regex::new(r"[~'!()*]").expect("regex is valid"));
const BLACKLIST: [&str; 2] = ["node_modules", "favicon.ico"];

// Borrowed from https://github.com/juliangruber/builtins
const BUILTINS: [&str; 39] = [
    "assert",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "https",
    "module",
    "net",
    "os",
    "path",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "stream",
    "string_decoder",
    "sys",
    "timers",
    "tls",
    "tty",
    "url",
    "util",
    "vm",
    "zlib",
    "freelist",
    // excluded only in some versions
    "freelist",
    "v8",
    "process",
    "async_hooks",
    "http2",
    "perf_hooks",
];

#[derive(Debug, PartialEq, Eq)]
pub enum Validity {
    /// Valid for new and old packages
    Valid,

    /// Valid only for old packages
    ValidForOldPackages { warnings: Vec<String> },

    /// Not valid for new or old packages
    Invalid {
        warnings: Vec<String>,
        errors: Vec<String>,
    },
}

impl Validity {
    pub fn valid_for_old_packages(&self) -> bool {
        matches!(self, Validity::Valid | Validity::ValidForOldPackages { .. })
    }

    pub fn valid_for_new_packages(&self) -> bool {
        matches!(self, Validity::Valid)
    }
}

pub fn validate(name: &str) -> Validity {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if name.is_empty() {
        errors.push("name length must be greater than zero".into());
    }

    if name.starts_with('.') {
        errors.push("name cannot start with a period".into());
    }

    if name.starts_with('_') {
        errors.push("name cannot start with an underscore".into());
    }

    if name.trim() != name {
        errors.push("name cannot contain leading or trailing spaces".into());
    }

    // No funny business
    for blacklisted_name in BLACKLIST.iter() {
        if &name.to_lowercase() == blacklisted_name {
            errors.push(format!("{} is a blacklisted name", blacklisted_name));
        }
    }

    // Generate warnings for stuff that used to be allowed

    for builtin in BUILTINS.iter() {
        if name.to_lowercase() == *builtin {
            warnings.push(format!("{} is a core module name", builtin));
        }
    }

    // really-long-package-names-------------------------------such--length-----many---wow
    // the thisisareallyreallylongpackagenameitshouldpublishdowenowhavealimittothelengthofpackagenames-poch.
    if name.len() > 214 {
        warnings.push("name can no longer contain more than 214 characters".into());
    }

    // mIxeD CaSe nAMEs
    if name.to_lowercase() != name {
        warnings.push("name can no longer contain capital letters".into());
    }

    if name
        .split('/')
        .last()
        .map(|final_part| SPECIAL_CHARS.is_match(final_part))
        .unwrap_or(false)
    {
        warnings.push(r#"name can no longer contain special characters ("~\'!()*")"#.into());
    }

    if utf8_percent_encode(name, ENCODE_URI_SET).to_string() != name {
        // Maybe it's a scoped package name, like @user/package
        if let Some(captures) = SCOPED_PACKAGE.captures(name) {
            let valid_scope_name = captures
                .get(1)
                .map(|scope| scope.as_str())
                .map(|scope| utf8_percent_encode(scope, ENCODE_URI_SET).to_string() == scope)
                .unwrap_or(true);

            let valid_package_name = captures
                .get(2)
                .map(|package| package.as_str())
                .map(|package| utf8_percent_encode(package, ENCODE_URI_SET).to_string() == package)
                .unwrap_or(true);

            if valid_scope_name && valid_package_name {
                return done(warnings, errors);
            }
        }

        errors.push("name can only contain URL-friendly characters".into());
    }

    done(warnings, errors)
}

fn done(warnings: Vec<String>, errors: Vec<String>) -> Validity {
    match (warnings.len(), errors.len()) {
        (0, 0) => Validity::Valid,
        (_, 0) => Validity::ValidForOldPackages { warnings },
        (_, _) => Validity::Invalid { warnings, errors },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traditional() {
        assert_eq!(validate("some-package"), Validity::Valid);
        assert_eq!(validate("example.com"), Validity::Valid);
        assert_eq!(validate("under_score"), Validity::Valid);
        assert_eq!(validate("period.js"), Validity::Valid);
        assert_eq!(validate("123numeric"), Validity::Valid);
        assert_eq!(
            validate("crazy!"),
            Validity::ValidForOldPackages {
                warnings: vec![
                    r#"name can no longer contain special characters ("~\'!()*")"#.into()
                ]
            }
        );
    }

    #[test]
    fn scoped() {
        assert_eq!(validate("@npm/thingy"), Validity::Valid);
        assert_eq!(
            validate("@npm-zors/money!time.js"),
            Validity::ValidForOldPackages {
                warnings: vec![
                    r#"name can no longer contain special characters ("~\'!()*")"#.into()
                ]
            }
        );
    }

    #[test]
    fn invalid() {
        assert_eq!(
            validate(""),
            Validity::Invalid {
                errors: vec!["name length must be greater than zero".into()],
                warnings: vec![]
            }
        );

        assert_eq!(
            validate(".start-with-period"),
            Validity::Invalid {
                errors: vec!["name cannot start with a period".into()],
                warnings: vec![]
            }
        );

        assert_eq!(
            validate("_start-with-underscore"),
            Validity::Invalid {
                errors: vec!["name cannot start with an underscore".into()],
                warnings: vec![]
            }
        );

        assert_eq!(
            validate("contain:colons"),
            Validity::Invalid {
                errors: vec!["name can only contain URL-friendly characters".into()],
                warnings: vec![]
            }
        );

        assert_eq!(
            validate(" leading-space"),
            Validity::Invalid {
                errors: vec![
                    "name cannot contain leading or trailing spaces".into(),
                    "name can only contain URL-friendly characters".into()
                ],
                warnings: vec![]
            }
        );

        assert_eq!(
            validate("trailing-space "),
            Validity::Invalid {
                errors: vec![
                    "name cannot contain leading or trailing spaces".into(),
                    "name can only contain URL-friendly characters".into()
                ],
                warnings: vec![]
            }
        );

        assert_eq!(
            validate("s/l/a/s/h/e/s"),
            Validity::Invalid {
                errors: vec!["name can only contain URL-friendly characters".into()],
                warnings: vec![]
            }
        );

        assert_eq!(
            validate("node_modules"),
            Validity::Invalid {
                errors: vec!["node_modules is a blacklisted name".into()],
                warnings: vec![]
            }
        );

        assert_eq!(
            validate("favicon.ico"),
            Validity::Invalid {
                errors: vec!["favicon.ico is a blacklisted name".into()],
                warnings: vec![]
            }
        );
    }

    #[test]
    fn node_io_core() {
        assert_eq!(
            validate("http"),
            Validity::ValidForOldPackages {
                warnings: vec!["http is a core module name".into()]
            }
        );
    }

    #[test]
    fn long_package_names() {
        let one_too_long = "ifyouwanttogetthesumoftwonumberswherethosetwonumbersarechosenbyfindingthelargestoftwooutofthreenumbersandsquaringthemwhichismultiplyingthembyitselfthenyoushouldinputthreenumbersintothisfunctionanditwilldothatforyou-";
        let short_enough = "ifyouwanttogetthesumoftwonumberswherethosetwonumbersarechosenbyfindingthelargestoftwooutofthreenumbersandsquaringthemwhichismultiplyingthembyitselfthenyoushouldinputthreenumbersintothisfunctionanditwilldothatforyou";

        assert_eq!(
            validate(one_too_long),
            Validity::ValidForOldPackages {
                warnings: vec!["name can no longer contain more than 214 characters".into()]
            }
        );

        assert_eq!(validate(short_enough), Validity::Valid);
    }

    #[test]
    fn legacy_mixed_case() {
        assert_eq!(
            validate("CAPITAL-LETTERS"),
            Validity::ValidForOldPackages {
                warnings: vec!["name can no longer contain capital letters".into()]
            }
        );
    }
}
