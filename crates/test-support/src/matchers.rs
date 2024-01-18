use std::fmt;
use std::process::Output;
use std::str;
use std::usize;

use crate::process::ProcessBuilder;

use hamcrest2::core::{MatchResult, Matcher};
use serde_json::{self, Value};

#[derive(Clone)]
pub struct Execs {
    expect_stdout: Option<String>,
    expect_stderr: Option<String>,
    expect_exit_code: Option<i32>,
    expect_stdout_contains: Vec<String>,
    expect_stderr_contains: Vec<String>,
    expect_either_contains: Vec<String>,
    expect_stdout_contains_n: Vec<(String, usize)>,
    expect_stdout_not_contains: Vec<String>,
    expect_stderr_not_contains: Vec<String>,
    expect_stderr_unordered: Vec<String>,
    expect_neither_contains: Vec<String>,
    expect_json: Option<Vec<Value>>,
}

impl Execs {
    /// Verify that stdout is equal to the given lines.
    /// See `lines_match` for supported patterns.
    pub fn with_stdout<S: ToString>(mut self, expected: S) -> Execs {
        self.expect_stdout = Some(expected.to_string());
        self
    }

    /// Verify that stderr is equal to the given lines.
    /// See `lines_match` for supported patterns.
    pub fn with_stderr<S: ToString>(mut self, expected: S) -> Execs {
        self._with_stderr(&expected);
        self
    }

    fn _with_stderr(&mut self, expected: &dyn ToString) {
        self.expect_stderr = Some(expected.to_string());
    }

    /// Verify the exit code from the process.
    pub fn with_status(mut self, expected: i32) -> Execs {
        self.expect_exit_code = Some(expected);
        self
    }

    /// Verify that stdout contains the given contiguous lines somewhere in
    /// its output.
    /// See `lines_match` for supported patterns.
    pub fn with_stdout_contains<S: ToString>(mut self, expected: S) -> Execs {
        self.expect_stdout_contains.push(expected.to_string());
        self
    }

    /// Verify that stderr contains the given contiguous lines somewhere in
    /// its output.
    /// See `lines_match` for supported patterns.
    pub fn with_stderr_contains<S: ToString>(mut self, expected: S) -> Execs {
        self.expect_stderr_contains.push(expected.to_string());
        self
    }

    /// Verify that either stdout or stderr contains the given contiguous
    /// lines somewhere in its output.
    /// See `lines_match` for supported patterns.
    pub fn with_either_contains<S: ToString>(mut self, expected: S) -> Execs {
        self.expect_either_contains.push(expected.to_string());
        self
    }

    /// Verify that stdout contains the given contiguous lines somewhere in
    /// its output, and should be repeated `number` times.
    /// See `lines_match` for supported patterns.
    pub fn with_stdout_contains_n<S: ToString>(mut self, expected: S, number: usize) -> Execs {
        self.expect_stdout_contains_n
            .push((expected.to_string(), number));
        self
    }

    /// Verify that stdout does not contain the given contiguous lines.
    /// See `lines_match` for supported patterns.
    /// See note on `with_stderr_does_not_contain`.
    pub fn with_stdout_does_not_contain<S: ToString>(mut self, expected: S) -> Execs {
        self.expect_stdout_not_contains.push(expected.to_string());
        self
    }

    /// Verify that stderr does not contain the given contiguous lines.
    /// See `lines_match` for supported patterns.
    ///
    /// Care should be taken when using this method because there is a
    /// limitless number of possible things that *won't* appear.  A typo means
    /// your test will pass without verifying the correct behavior. If
    /// possible, write the test first so that it fails, and then implement
    /// your fix/feature to make it pass.
    pub fn with_stderr_does_not_contain<S: ToString>(mut self, expected: S) -> Execs {
        self.expect_stderr_not_contains.push(expected.to_string());
        self
    }

    /// Verify that all of the stderr output is equal to the given lines,
    /// ignoring the order of the lines.
    /// See `lines_match` for supported patterns.
    /// This is useful when checking the output of `cargo build -v` since
    /// the order of the output is not always deterministic.
    /// Recommend use `with_stderr_contains` instead unless you really want to
    /// check *every* line of output.
    ///
    /// Be careful when using patterns such as `[..]`, because you may end up
    /// with multiple lines that might match, and this is not smart enough to
    /// do anything like longest-match.  For example, avoid something like:
    ///     [RUNNING] `rustc [..]
    ///     [RUNNING] `rustc --crate-name foo [..]
    /// This will randomly fail if the other crate name is `bar`, and the
    /// order changes.
    pub fn with_stderr_unordered<S: ToString>(mut self, expected: S) -> Execs {
        self.expect_stderr_unordered.push(expected.to_string());
        self
    }

    /// Verify the JSON output matches the given JSON.
    /// Typically used when testing cargo commands that emit JSON.
    /// Each separate JSON object should be separated by a blank line.
    /// Example:
    ///     assert_that(
    ///         p.cargo("metadata"),
    ///         execs().with_json(r#"
    ///             {"example": "abc"}
    ///             {"example": "def"}
    ///         "#)
    ///      );
    /// Objects should match in the order given.
    /// The order of arrays is ignored.
    /// Strings support patterns described in `lines_match`.
    /// Use `{...}` to match any object.
    pub fn with_json(mut self, expected: &str) -> Execs {
        self.expect_json = Some(
            expected
                .split("\n\n")
                .map(|obj| obj.parse().unwrap())
                .collect(),
        );
        self
    }

    fn match_output(&self, actual: &Output) -> MatchResult {
        self.match_status(actual)
            .and(self.match_stdout(actual))
            .and(self.match_stderr(actual))
    }

    fn match_status(&self, actual: &Output) -> MatchResult {
        match self.expect_exit_code {
            None => Ok(()),
            Some(code) if actual.status.code() == Some(code) => Ok(()),
            Some(_) => Err(format!(
                "exited with {}\n--- stdout\n{}\n--- stderr\n{}",
                actual.status,
                String::from_utf8_lossy(&actual.stdout),
                String::from_utf8_lossy(&actual.stderr)
            )),
        }
    }

    fn match_stdout(&self, actual: &Output) -> MatchResult {
        self.match_std(
            self.expect_stdout.as_ref(),
            &actual.stdout,
            "stdout",
            &actual.stderr,
            MatchKind::Exact,
        )?;
        for expect in self.expect_stdout_contains.iter() {
            self.match_std(
                Some(expect),
                &actual.stdout,
                "stdout",
                &actual.stderr,
                MatchKind::Partial,
            )?;
        }
        for expect in self.expect_stderr_contains.iter() {
            self.match_std(
                Some(expect),
                &actual.stderr,
                "stderr",
                &actual.stdout,
                MatchKind::Partial,
            )?;
        }
        for &(ref expect, number) in self.expect_stdout_contains_n.iter() {
            self.match_std(
                Some(expect),
                &actual.stdout,
                "stdout",
                &actual.stderr,
                MatchKind::PartialN(number),
            )?;
        }
        for expect in self.expect_stdout_not_contains.iter() {
            self.match_std(
                Some(expect),
                &actual.stdout,
                "stdout",
                &actual.stderr,
                MatchKind::NotPresent,
            )?;
        }
        for expect in self.expect_stderr_not_contains.iter() {
            self.match_std(
                Some(expect),
                &actual.stderr,
                "stderr",
                &actual.stdout,
                MatchKind::NotPresent,
            )?;
        }
        for expect in self.expect_stderr_unordered.iter() {
            self.match_std(
                Some(expect),
                &actual.stderr,
                "stderr",
                &actual.stdout,
                MatchKind::Unordered,
            )?;
        }
        for expect in self.expect_neither_contains.iter() {
            self.match_std(
                Some(expect),
                &actual.stdout,
                "stdout",
                &actual.stdout,
                MatchKind::NotPresent,
            )?;

            self.match_std(
                Some(expect),
                &actual.stderr,
                "stderr",
                &actual.stderr,
                MatchKind::NotPresent,
            )?;
        }

        for expect in self.expect_either_contains.iter() {
            let match_std = self.match_std(
                Some(expect),
                &actual.stdout,
                "stdout",
                &actual.stdout,
                MatchKind::Partial,
            );
            let match_err = self.match_std(
                Some(expect),
                &actual.stderr,
                "stderr",
                &actual.stderr,
                MatchKind::Partial,
            );

            if let (Err(_), Err(_)) = (match_std, match_err) {
                return Err(format!(
                    "expected to find:\n\
                     {}\n\n\
                     did not find in either output.",
                    expect
                ));
            }
        }

        if let Some(ref objects) = self.expect_json {
            let stdout = str::from_utf8(&actual.stdout)
                .map_err(|_| "stdout was not utf8 encoded".to_owned())?;
            let lines = stdout
                .lines()
                .filter(|line| line.starts_with('{'))
                .collect::<Vec<_>>();
            if lines.len() != objects.len() {
                return Err(format!(
                    "expected {} json lines, got {}, stdout:\n{}",
                    objects.len(),
                    lines.len(),
                    stdout
                ));
            }
            for (obj, line) in objects.iter().zip(lines) {
                self.match_json(obj, line)?;
            }
        }
        Ok(())
    }

    fn match_stderr(&self, actual: &Output) -> MatchResult {
        self.match_std(
            self.expect_stderr.as_ref(),
            &actual.stderr,
            "stderr",
            &actual.stdout,
            MatchKind::Exact,
        )
    }

    fn match_std(
        &self,
        expected: Option<&String>,
        actual: &[u8],
        description: &str,
        extra: &[u8],
        kind: MatchKind,
    ) -> MatchResult {
        let out = match expected {
            Some(out) => out,
            None => return Ok(()),
        };
        let actual = match str::from_utf8(actual) {
            Err(..) => return Err(format!("{} was not utf8 encoded", description)),
            Ok(actual) => actual,
        };
        // Let's not deal with \r\n vs \n on windows...
        let actual = actual.replace('\r', "");
        let actual = actual.replace('\t', "<tab>");

        match kind {
            MatchKind::Exact => {
                let a = actual.lines();
                let e = out.lines();

                let diffs = self.diff_lines(a, e, false);
                if diffs.is_empty() {
                    Ok(())
                } else {
                    Err(format!(
                        "differences:\n\
                         {}\n\n\
                         other output:\n\
                         `{}`",
                        diffs.join("\n"),
                        String::from_utf8_lossy(extra)
                    ))
                }
            }
            MatchKind::Partial => {
                let mut a = actual.lines();
                let e = out.lines();

                let mut diffs = self.diff_lines(a.clone(), e.clone(), true);
                #[allow(clippy::while_let_on_iterator)]
                while let Some(..) = a.next() {
                    let a = self.diff_lines(a.clone(), e.clone(), true);
                    if a.len() < diffs.len() {
                        diffs = a;
                    }
                }
                if diffs.is_empty() {
                    Ok(())
                } else {
                    Err(format!(
                        "expected to find:\n\
                         {}\n\n\
                         did not find in output:\n\
                         {}",
                        out, actual
                    ))
                }
            }
            MatchKind::PartialN(number) => {
                let mut a = actual.lines();
                let e = out.lines();

                let mut matches = 0;

                loop {
                    if self.diff_lines(a.clone(), e.clone(), true).is_empty() {
                        matches += 1;
                    }

                    if a.next().is_none() {
                        break;
                    }
                }

                if matches == number {
                    Ok(())
                } else {
                    Err(format!(
                        "expected to find {} occurrences:\n\
                         {}\n\n\
                         did not find in output:\n\
                         {}",
                        number, out, actual
                    ))
                }
            }
            MatchKind::NotPresent => {
                let mut a = actual.lines();
                let e = out.lines();

                let mut diffs = self.diff_lines(a.clone(), e.clone(), true);
                #[allow(clippy::while_let_on_iterator)]
                while let Some(..) = a.next() {
                    let a = self.diff_lines(a.clone(), e.clone(), true);
                    if a.len() < diffs.len() {
                        diffs = a;
                    }
                }
                if diffs.is_empty() {
                    Err(format!(
                        "expected not to find:\n\
                         {}\n\n\
                         but found in output:\n\
                         {}",
                        out, actual
                    ))
                } else {
                    Ok(())
                }
            }
            MatchKind::Unordered => {
                let mut a = actual.lines().collect::<Vec<_>>();
                let e = out.lines();

                for e_line in e {
                    match a.iter().position(|a_line| lines_match(e_line, a_line)) {
                        Some(index) => a.remove(index),
                        None => {
                            return Err(format!(
                                "Did not find expected line:\n\
                                 {}\n\
                                 Remaining available output:\n\
                                 {}\n",
                                e_line,
                                a.join("\n")
                            ));
                        }
                    };
                }
                if !a.is_empty() {
                    Err(format!(
                        "Output included extra lines:\n\
                         {}\n",
                        a.join("\n")
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }

    fn match_json(&self, expected: &Value, line: &str) -> MatchResult {
        let actual = match line.parse() {
            Err(e) => return Err(format!("invalid json, {}:\n`{}`", e, line)),
            Ok(actual) => actual,
        };

        match find_mismatch(expected, &actual) {
            Some((expected_part, actual_part)) => Err(format!(
                "JSON mismatch\nExpected:\n{}\nWas:\n{}\nExpected part:\n{}\nActual part:\n{}\n",
                serde_json::to_string_pretty(expected).unwrap(),
                serde_json::to_string_pretty(&actual).unwrap(),
                serde_json::to_string_pretty(expected_part).unwrap(),
                serde_json::to_string_pretty(actual_part).unwrap(),
            )),
            None => Ok(()),
        }
    }

    fn diff_lines<'a>(
        &self,
        actual: str::Lines<'a>,
        expected: str::Lines<'a>,
        partial: bool,
    ) -> Vec<String> {
        let actual = actual.take(if partial {
            expected.clone().count()
        } else {
            usize::MAX
        });
        zip_all(actual, expected)
            .enumerate()
            .filter_map(|(i, (a, e))| match (a, e) {
                (Some(a), Some(e)) => {
                    if lines_match(e, a) {
                        None
                    } else {
                        Some(format!("{:3} - |{}|\n    + |{}|\n", i, e, a))
                    }
                }
                (Some(a), None) => Some(format!("{:3} -\n    + |{}|\n", i, a)),
                (None, Some(e)) => Some(format!("{:3} - |{}|\n    +\n", i, e)),
                (None, None) => panic!("Cannot get here"),
            })
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum MatchKind {
    Exact,
    Partial,
    PartialN(usize),
    NotPresent,
    Unordered,
}

/// Compare a line with an expected pattern.
/// - Use `[..]` as a wildcard to match 0 or more characters on the same line
///   (similar to `.*` in a regex).
/// - Use `[EXE]` to optionally add `.exe` on Windows (empty string on other
///   platforms).
/// - There is a wide range of macros (such as `[COMPILING]` or `[WARNING]`)
///   to match cargo's "status" output and allows you to ignore the alignment.
///   See `substitute_macros` for a complete list of macros.
pub fn lines_match(expected: &str, actual: &str) -> bool {
    // Let's not deal with / vs \ (windows...)
    let expected = expected.replace('\\', "/");
    let mut actual: &str = &actual.replace('\\', "/");
    let expected = substitute_macros(&expected);
    for (i, part) in expected.split("[..]").enumerate() {
        match actual.find(part) {
            Some(j) => {
                if i == 0 && j != 0 {
                    return false;
                }
                actual = &actual[j + part.len()..];
            }
            None => return false,
        }
    }
    actual.is_empty() || expected.ends_with("[..]")
}

#[test]
fn lines_match_works() {
    assert!(lines_match("a b", "a b"));
    assert!(lines_match("a[..]b", "a b"));
    assert!(lines_match("a[..]", "a b"));
    assert!(lines_match("[..]", "a b"));
    assert!(lines_match("[..]b", "a b"));

    assert!(!lines_match("[..]b", "c"));
    assert!(!lines_match("b", "c"));
    assert!(!lines_match("b", "cb"));
}

// Compares JSON object for approximate equality.
// You can use `[..]` wildcard in strings (useful for OS dependent things such
// as paths).  You can use a `"{...}"` string literal as a wildcard for
// arbitrary nested JSON (useful for parts of object emitted by other programs
// (e.g. rustc) rather than Cargo itself).  Arrays are sorted before comparison.
fn find_mismatch<'a>(expected: &'a Value, actual: &'a Value) -> Option<(&'a Value, &'a Value)> {
    use serde_json::Value::*;
    match (expected, actual) {
        (Number(l), Number(r)) if l == r => None,
        (Bool(l), Bool(r)) if l == r => None,
        (String(l), String(r)) if lines_match(l, r) => None,
        (Array(l), Array(r)) => {
            if l.len() != r.len() {
                return Some((expected, actual));
            }

            let mut l = l.iter().collect::<Vec<_>>();
            let mut r = r.iter().collect::<Vec<_>>();

            l.retain(
                |l| match r.iter().position(|r| find_mismatch(l, r).is_none()) {
                    Some(i) => {
                        r.remove(i);
                        false
                    }
                    None => true,
                },
            );

            if !l.is_empty() {
                assert!(!r.is_empty());
                Some((l[0], r[0]))
            } else {
                assert_eq!(r.len(), 0);
                None
            }
        }
        (Object(l), Object(r)) => {
            let same_keys = l.len() == r.len() && l.keys().all(|k| r.contains_key(k));
            if !same_keys {
                return Some((expected, actual));
            }

            l.values()
                .zip(r.values())
                .find_map(|(l, r)| find_mismatch(l, r))
        }
        (Null, Null) => None,
        // magic string literal "{...}" acts as wildcard for any sub-JSON
        (String(l), _) if l == "{...}" => None,
        _ => Some((expected, actual)),
    }
}

struct ZipAll<I1: Iterator, I2: Iterator> {
    first: I1,
    second: I2,
}

impl<T, I1: Iterator<Item = T>, I2: Iterator<Item = T>> Iterator for ZipAll<I1, I2> {
    type Item = (Option<T>, Option<T>);
    fn next(&mut self) -> Option<(Option<T>, Option<T>)> {
        let first = self.first.next();
        let second = self.second.next();

        match (first, second) {
            (None, None) => None,
            (a, b) => Some((a, b)),
        }
    }
}

fn zip_all<T, I1: Iterator<Item = T>, I2: Iterator<Item = T>>(a: I1, b: I2) -> ZipAll<I1, I2> {
    ZipAll {
        first: a,
        second: b,
    }
}

impl fmt::Display for Execs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "execs")
    }
}

impl fmt::Debug for Execs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "execs")
    }
}

impl Matcher<ProcessBuilder> for Execs {
    fn matches(&self, mut process: ProcessBuilder) -> MatchResult {
        self.matches(&mut process)
    }
}

impl<'a> Matcher<&'a mut ProcessBuilder> for Execs {
    fn matches(&self, process: &'a mut ProcessBuilder) -> MatchResult {
        println!("running {}", process);
        let res = process.exec_with_output();

        match res {
            Ok(out) => self.match_output(&out),
            Err(err) => {
                if let Some(out) = &err.output {
                    return self.match_output(out);
                }
                Err(format!("could not exec process {}: {}", process, err))
            }
        }
    }
}

impl Matcher<Output> for Execs {
    fn matches(&self, output: Output) -> MatchResult {
        self.match_output(&output)
    }
}

pub fn execs() -> Execs {
    Execs {
        expect_stdout: None,
        expect_stderr: None,
        expect_exit_code: Some(0),
        expect_stdout_contains: Vec::new(),
        expect_stderr_contains: Vec::new(),
        expect_either_contains: Vec::new(),
        expect_stdout_contains_n: Vec::new(),
        expect_stdout_not_contains: Vec::new(),
        expect_stderr_not_contains: Vec::new(),
        expect_stderr_unordered: Vec::new(),
        expect_neither_contains: Vec::new(),
        expect_json: None,
    }
}

fn substitute_macros(input: &str) -> String {
    let macros = [
        ("[RUNNING]", "     Running"),
        ("[COMPILING]", "   Compiling"),
        ("[CHECKING]", "    Checking"),
        ("[CREATED]", "     Created"),
        ("[FINISHED]", "    Finished"),
        ("[ERROR]", "error:"),
        ("[WARNING]", "warning:"),
        ("[DOCUMENTING]", " Documenting"),
        ("[FRESH]", "       Fresh"),
        ("[UPDATING]", "    Updating"),
        ("[ADDING]", "      Adding"),
        ("[REMOVING]", "    Removing"),
        ("[DOCTEST]", "   Doc-tests"),
        ("[PACKAGING]", "   Packaging"),
        ("[DOWNLOADING]", " Downloading"),
        ("[UPLOADING]", "   Uploading"),
        ("[VERIFYING]", "   Verifying"),
        ("[ARCHIVING]", "   Archiving"),
        ("[INSTALLING]", "  Installing"),
        ("[REPLACING]", "   Replacing"),
        ("[UNPACKING]", "   Unpacking"),
        ("[SUMMARY]", "     Summary"),
        ("[FIXING]", "      Fixing"),
        ("[EXE]", if cfg!(windows) { ".exe" } else { "" }),
    ];
    let mut result = input.to_owned();
    for &(pat, subst) in &macros {
        result = result.replace(pat, subst)
    }
    result
}
