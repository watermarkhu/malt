//! The MonkeyProof Coding Standard for MATLAB, as an extra set of checks.
//!
//! The standard is published by MonkeyProof Solutions B.V. at
//! <https://doc.monkeyproofsolutions.nl/code-checker-for-matlab/monkeyproof-coding-standard-for-matlab/>.
//! It is not part of MATLAB's Code Analyzer. The checks here are malt's own reading of the
//! rules the standard marks as automatically checkable, so they are off unless the
//! `monkeyproof` preset or the `monkeyproof` category is switched on.
//!
//! The checks are grouped as the standard groups its rules, one engine per group:
//!
//! | Module | Engine | Standard |
//! | --- | --- | --- |
//! | [`naming`] | `MP_NAMING_ENGINE` | Naming conventions (`NAMING-n`) |
//! | [`layout`] | `MP_LAYOUT_ENGINE` | Layout and comments (`LAYOUT-n`) |
//! | [`statements`] | `MP_STATEMENTS_ENGINE` | Statements and expressions (`STAT-n`) |
//! | [`functions`] | `MP_FUNCTIONS_ENGINE` | Higher level rules (`FCNS-n`) |
//! | [`security`] | `MP_SECURITY_ENGINE` | Security (`SECURITY-n`) |
//!
//! A check ID is the standard's ID with its dash turned into an underscore:
//! `NAMING-2` is `MP_NAMING_2`. That keeps it usable in `%#ok<...>` directives and as an
//! unquoted key in `.malt.toml`.
//!
//! # Adding a check
//!
//! A check is one file, `<group>/check_<name>.rs`, with a function
//!
//! ```ignore
//! pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>)
//! ```
//!
//! that pushes a [`finding`] for each violation, plus one row in the group's
//! `CHECKS` table (check ID, the standard's severity level, the function) and one
//! row in the group's docstring table. [`GroupEngine`] runs the checks that are
//! enabled, stamps the severity (see [`severity_of`]) and applies a per-check
//! `severity` override. A threshold is a parameter of the check's own table
//! (`[lint.rules.MP_LAYOUT_5] max_length = 120`), read with `config.rule_params`.

use std::ops::Range;

use malt_core::{position_of, Category, Config, Diagnostic, FileContext, Rule, Severity};

pub mod functions;
pub mod layout;
pub mod naming;
pub mod security;
pub mod statements;

/// The severity malt gives a finding of a standard's severity level (1 is the most
/// severe, 10 the least): levels 1 and 2 are errors, 3 to 5 warnings and 6 to 10 info.
///
/// The standard defines level 1 as certainly a programming error, 2 as very likely one,
/// 3 to 5 as pitfalls and design issues, 6 to 10 as portability, comment, naming and style
/// issues.
pub(crate) fn severity_of(level: u8) -> Severity {
    match level {
        0..=2 => Severity::Error,
        3..=5 => Severity::Warning,
        _ => Severity::Info,
    }
}

/// A finding of check `id` at `range` of `source`. The severity is set by the engine from
/// the check's level, so it is a placeholder here.
// Unused until the first check of a group uses it.
#[allow(dead_code)]
pub(crate) fn finding(
    source: &str,
    id: &'static str,
    range: Range<usize>,
    message: impl Into<String>,
) -> Diagnostic {
    let (line, column) = position_of(source, range.start);
    Diagnostic {
        rule_id: id,
        message: message.into(),
        severity: Severity::Info,
        byte_range: range,
        line,
        column,
        fix: None,
    }
}

/// One check of a group: its ID, the standard's severity level and its function.
pub(crate) struct Check {
    /// The check ID (`MP_NAMING_2`).
    pub id: &'static str,
    /// The severity level of the standard, 1 to 10 (see [`severity_of`]).
    pub level: u8,
    /// Runs the check over a file and pushes its findings.
    pub run: fn(&Config, &FileContext, &mut Vec<Diagnostic>),
}

/// The engine of one group of the standard: runs the enabled checks of its table over
/// each file.
pub(crate) struct GroupEngine {
    id: &'static str,
    description: &'static str,
    checks: &'static [Check],
    config: Config,
}

impl GroupEngine {
    /// An engine for `checks`, with the configuration it reads thresholds from.
    pub(crate) fn boxed(
        id: &'static str,
        description: &'static str,
        checks: &'static [Check],
        config: &Config,
    ) -> Box<dyn Rule> {
        Box::new(Self {
            id,
            description,
            checks,
            config: config.clone(),
        })
    }
}

impl Rule for GroupEngine {
    fn id(&self) -> &'static str {
        self.id
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn category(&self) -> Category {
        Category::MonkeyProof
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        &[]
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for check in self.checks {
            if !self.config.is_rule_enabled(check.id) {
                continue;
            }
            let start = out.len();
            (check.run)(&self.config, ctx, &mut out);
            let severity = self
                .config
                .rules
                .get(check.id)
                .and_then(|rule| rule.severity)
                .unwrap_or_else(|| severity_of(check.level));
            for diagnostic in &mut out[start..] {
                diagnostic.severity = severity;
            }
        }
        out
    }
}

/// Run one group's engine over `source`, with `toml` as the configuration. For tests.
// Unused until the first check of a group is tested.
#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn lint(
    from_config: fn(&Config) -> Box<dyn Rule>,
    source: &str,
    toml: &str,
) -> Vec<Diagnostic> {
    let config = Config::from_toml(toml).expect("valid test configuration");
    let engine = from_config(&config);
    crate::test_util::lint_file(engine.as_ref(), source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_map_to_severities() {
        assert_eq!(severity_of(1), Severity::Error);
        assert_eq!(severity_of(2), Severity::Error);
        assert_eq!(severity_of(3), Severity::Warning);
        assert_eq!(severity_of(5), Severity::Warning);
        assert_eq!(severity_of(6), Severity::Info);
        assert_eq!(severity_of(10), Severity::Info);
    }

    #[test]
    fn a_finding_has_a_one_based_position() {
        let d = finding("a = 1;\nb = 2;\n", "MP_TEST_1", 11..12, "x");
        assert_eq!((d.line, d.column), (2, 5));
    }
}
