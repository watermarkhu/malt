//! The `--output` and `--log-level` choices of the command line. The formats themselves
//! are in `malt_core::report`, shared with the WebAssembly package.

use clap::ValueEnum;
use malt_core::Severity;

/// The format of the findings on stdout (`--output`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// `file:line:col [E] RULE: message`, one line per finding (the default).
    Text,
    /// GitHub Actions workflow commands (annotations) and, with `GITHUB_STEP_SUMMARY`
    /// set, a markdown step summary.
    Github,
    /// The GitLab Code Quality report (CodeClimate JSON) on stdout. The text lines go
    /// to stderr, so the report stays valid JSON.
    Gitlab,
}

/// Which finding lines are written as text (`--log-level`), whatever the output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    /// No finding lines.
    Off,
    /// Errors only.
    Error,
    /// Errors and warnings.
    Warning,
    /// Everything (the default).
    Info,
}

impl LogLevel {
    /// Whether a finding of `severity` is written as a text line at this level.
    pub fn shows(self, severity: Severity) -> bool {
        match self {
            LogLevel::Off => false,
            LogLevel::Error => severity == Severity::Error,
            LogLevel::Warning => severity != Severity::Info,
            LogLevel::Info => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_level_filters_by_severity() {
        assert!(!LogLevel::Off.shows(Severity::Error));
        assert!(LogLevel::Error.shows(Severity::Error));
        assert!(!LogLevel::Error.shows(Severity::Warning));
        assert!(LogLevel::Warning.shows(Severity::Warning));
        assert!(!LogLevel::Warning.shows(Severity::Info));
        assert!(LogLevel::Info.shows(Severity::Info));
    }
}
