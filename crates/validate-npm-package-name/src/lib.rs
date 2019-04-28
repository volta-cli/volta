//! A Rust implementation of the validation rules from the core JS package
//! [`validate-npm-package-name`](https://github.com/npm/validate-npm-package-name/).

use lazy_static::lazy_static;
use regex::Regex;
use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};

lazy_static! {
    static ref SCOPED_PACKAGE: Regex =
        Regex::new(r"^(?:@([^/]+?)[/])?([^/]+?)$").expect("regex is valid");
    static ref SPECIAL_CHARS: Regex = Regex::new(r"[~'!()*]").expect("regex is valid");
    static ref BLACKLIST: Vec<&'static str> = vec!["node_modules", "favicon.ico"];

    // Borrowed from https://github.com/juliangruber/builtins
    static ref BUILTINS: Vec<&'static str> = vec![
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
}

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
        match self {
            Validity::Invalid { .. } => false,
            _ => true,
        }
    }

    pub fn valid_for_new_packages(&self) -> bool {
        match self {
            Validity::Valid => true,
            _ => false,
        }
    }
}

pub fn validate(name: &str) -> Validity {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if name.len() == 0 {
        errors.push("name length must be greater than zero".into());
    }

    if name.starts_with(".") {
        errors.push("name cannot start with a period".into());
    }

    if name.starts_with("_") {
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

    if percent_encode(name.as_bytes(), DEFAULT_ENCODE_SET).to_string() != name {
        // Maybe it's a scoped package name, like @user/package
        if let Some(captures) = SCOPED_PACKAGE.captures(name) {
            let valid_scope_name = captures
                .get(1)
                .map(|scope| scope.as_str())
                .map(|scope| {
                    percent_encode(scope.as_bytes(), DEFAULT_ENCODE_SET).to_string() == scope
                })
                .unwrap_or(true);

            let valid_package_name = captures
                .get(2)
                .map(|package| package.as_str())
                .map(|package| {
                    percent_encode(package.as_bytes(), DEFAULT_ENCODE_SET).to_string() == package
                })
                .unwrap_or(true);

            if !valid_scope_name || !valid_package_name {
                errors.push("name can only contain URL-friendly characters".into())
            }
        }
    }

    match (warnings.len(), errors.len()) {
        (0, 0) => Validity::Valid,
        (_, 0) => Validity::ValidForOldPackages { warnings },
        (_, _) => Validity::Invalid { warnings, errors },
    }
}
