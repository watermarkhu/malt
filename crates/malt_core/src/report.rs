//! The outputs of a run that are data: GitHub Actions annotations and step summary, the
//! GitLab Code Quality report, and the JSON results file with the configuration used.
//!
//! The CLI and the WebAssembly package (and so the GitHub Action) both write their
//! outputs here, so the formats cannot drift apart.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::{json, Map, Value};

use crate::config::{Config, RuleConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::limits::Limits;
use crate::rule::Category;

/// The version of the JSON results file's layout. Bumped when a key changes meaning.
pub const JSON_SCHEMA_VERSION: u32 = 1;

/// One finding of a run, as written to every output: where it is and what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// The file, as given or discovered (relative paths stay relative).
    pub path: String,
    /// 1-based line.
    pub line: usize,
    /// 1-based column.
    pub column: usize,
    /// The check ID (`NOSEMI`, `MP_NAMING_2`, `NOFIL`...).
    pub rule_id: String,
    /// The severity after configuration.
    pub severity: Severity,
    /// The message.
    pub message: String,
    /// Whether a fix is offered: `None`, `"safe"` or `"unsafe"`.
    pub fix: Option<&'static str>,
}

impl Record {
    /// The record of `diagnostic` in the file at `path`.
    pub fn new(path: &Path, diagnostic: &Diagnostic) -> Self {
        Self {
            path: path.display().to_string(),
            line: diagnostic.line,
            column: diagnostic.column,
            rule_id: diagnostic.rule_id.to_string(),
            severity: diagnostic.severity,
            message: diagnostic.message.clone(),
            fix: diagnostic
                .fix
                .as_ref()
                .map(|f| if f.is_safe() { "safe" } else { "unsafe" }),
        }
    }
}

impl Record {
    /// The record as JSON: `file`, `line`, `column`, `rule`, `severity`, `message`, `fix`.
    pub fn to_json(&self) -> Value {
        json!({
            "file": self.path,
            "line": self.line,
            "column": self.column,
            "rule": self.rule_id,
            "severity": severity_name(self.severity),
            "message": self.message,
            "fix": self.fix,
        })
    }

    /// A record from the JSON of [`Record::to_json`]; `None` when a key is missing.
    pub fn from_json(value: &Value) -> Option<Self> {
        let severity = match value.get("severity")?.as_str()? {
            "error" => Severity::Error,
            "warning" => Severity::Warning,
            "info" => Severity::Info,
            _ => return None,
        };
        let fix = match value.get("fix").and_then(Value::as_str) {
            Some("safe") => Some("safe"),
            Some("unsafe") => Some("unsafe"),
            _ => None,
        };
        Some(Self {
            path: value.get("file")?.as_str()?.to_string(),
            line: value.get("line")?.as_u64()? as usize,
            column: value.get("column")?.as_u64()? as usize,
            rule_id: value.get("rule")?.as_str()?.to_string(),
            severity,
            message: value.get("message")?.as_str()?.to_string(),
            fix,
        })
    }
}

/// The lowercase name of a severity, as the configuration spells it.
pub fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

// ---------------------------------------------------------------------------
// GitHub Actions
// ---------------------------------------------------------------------------

/// Escape the message part of a workflow command (`%`, CR and LF).
fn escape_data(text: &str) -> String {
    text.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

/// Escape a property value of a workflow command (also `:` and `,`).
fn escape_property(text: &str) -> String {
    escape_data(text).replace(':', "%3A").replace(',', "%2C")
}

/// The path GitHub should attach the annotation to: relative to the workspace when the
/// file is inside it, with forward slashes.
pub fn github_path(path: &str, workspace: Option<&Path>) -> String {
    let relative = workspace
        .and_then(|ws| Path::new(path).strip_prefix(ws).ok())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| path.to_string());
    relative
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

/// The workflow command of one finding: `::error file=…,line=…,col=…,title=…::message`.
pub fn github_annotation(record: &Record, workspace: Option<&Path>) -> String {
    let command = match record.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "notice",
    };
    format!(
        "::{command} file={},line={},col={},title={}::{}",
        escape_property(&github_path(&record.path, workspace)),
        record.line,
        record.column,
        escape_property(&record.rule_id),
        escape_data(&record.message),
    )
}

/// The markdown step summary: the counts by severity, the rules that fired most, and
/// every finding per file, folded.
pub fn github_step_summary(records: &[Record], files_linted: usize) -> String {
    let mut out = String::from("## malt\n\n");
    let count = |s: Severity| records.iter().filter(|r| r.severity == s).count();
    let _ = writeln!(out, "Linted {files_linted} file{}.\n", plural(files_linted));
    if records.is_empty() {
        out.push_str("No findings.\n");
        return out;
    }
    out.push_str("| Severity | Findings |\n| --- | ---: |\n");
    for severity in [Severity::Error, Severity::Warning, Severity::Info] {
        let _ = writeln!(out, "| {} | {} |", severity_name(severity), count(severity));
    }

    let mut rules: Vec<(&str, usize)> = Vec::new();
    for record in records {
        match rules.iter_mut().find(|(rule, _)| *rule == record.rule_id) {
            Some((_, n)) => *n += 1,
            None => rules.push((&record.rule_id, 1)),
        }
    }
    rules.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    out.push_str("\n### Rules\n\n| Rule | Findings |\n| --- | ---: |\n");
    for (rule, n) in rules.iter().take(15) {
        let _ = writeln!(out, "| `{rule}` | {n} |");
    }

    let mut files: Vec<(&str, Vec<&Record>)> = Vec::new();
    for record in records {
        match files.iter_mut().find(|(path, _)| *path == record.path) {
            Some((_, list)) => list.push(record),
            None => files.push((&record.path, vec![record])),
        }
    }
    out.push_str("\n### Findings\n\n");
    for (path, list) in &files {
        let _ = writeln!(
            out,
            "<details><summary><code>{}</code> ({})</summary>\n",
            path.replace('<', "&lt;"),
            list.len()
        );
        out.push_str(
            "| Line | Col | Severity | Rule | Message |\n| ---: | ---: | --- | --- | --- |\n",
        );
        for record in list {
            let _ = writeln!(
                out,
                "| {} | {} | {} | `{}` | {} |",
                record.line,
                record.column,
                severity_name(record.severity),
                record.rule_id,
                record.message.replace('|', "\\|").replace('\n', " "),
            );
        }
        out.push_str("\n</details>\n\n");
    }
    out
}

// ---------------------------------------------------------------------------
// GitLab Code Quality
// ---------------------------------------------------------------------------

/// The GitLab Code Quality report: a CodeClimate issue list. A finding's fingerprint is
/// a hash of its file, rule and message, so it stays the same when lines move.
pub fn gitlab_report(records: &[Record]) -> Value {
    Value::Array(
        records
            .iter()
            .map(|record| {
                let severity = match record.severity {
                    Severity::Error => "critical",
                    Severity::Warning => "major",
                    Severity::Info => "info",
                };
                let fingerprint = format!(
                    "{:016x}",
                    fnv1a(
                        format!(
                            "{}\u{0}{}\u{0}{}",
                            record.path, record.rule_id, record.message
                        )
                        .as_bytes()
                    )
                );
                json!({
                    "description": record.message,
                    "check_name": record.rule_id,
                    "fingerprint": fingerprint,
                    "severity": severity,
                    "location": {
                        "path": record.path.replace('\\', "/"),
                        "lines": { "begin": record.line },
                    },
                })
            })
            .collect(),
    )
}

/// 64-bit FNV-1a: a hash that does not change between Rust versions, unlike the
/// standard library's hasher.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

// ---------------------------------------------------------------------------
// JSON results file
// ---------------------------------------------------------------------------

/// What the JSON results file holds about a run.
pub struct RunInfo<'a> {
    /// The configuration file that was read, if any.
    pub config_file: Option<&'a Path>,
    /// The MATLAB configuration that was layered in, if any.
    pub matlab_config: Option<&'a Path>,
    /// The files linted, after discovery.
    pub files: &'a [String],
    /// The number of files with findings.
    pub files_with_findings: usize,
}

/// The JSON results file: the version, the configuration used, the files, and the
/// findings with their counts.
pub fn results_json(config: &Config, run: &RunInfo, records: &[Record]) -> Value {
    let mut counts = Map::new();
    for severity in [Severity::Error, Severity::Warning, Severity::Info] {
        let n = records.iter().filter(|r| r.severity == severity).count();
        counts.insert(severity_name(severity).to_string(), json!(n));
    }
    let mut rule_counts: Vec<(&str, usize)> = Vec::new();
    for record in records {
        match rule_counts
            .iter_mut()
            .find(|(rule, _)| *rule == record.rule_id)
        {
            Some((_, n)) => *n += 1,
            None => rule_counts.push((&record.rule_id, 1)),
        }
    }
    rule_counts.sort_by(|a, b| a.0.cmp(b.0));
    let by_rule: Map<String, Value> = rule_counts
        .into_iter()
        .map(|(rule, n)| (rule.to_string(), json!(n)))
        .collect();

    json!({
        "schema_version": JSON_SCHEMA_VERSION,
        "malt_version": env!("CARGO_PKG_VERSION"),
        "config": config_json(config, run.config_file, run.matlab_config),
        "files": run.files,
        "summary": {
            "files_linted": run.files.len(),
            "files_with_findings": run.files_with_findings,
            "findings": records.len(),
            "by_severity": counts,
            "by_rule": by_rule,
        },
        "findings": records.iter().map(Record::to_json).collect::<Vec<_>>(),
    })
}

/// The configuration of a run as data: the preset, categories, per-rule settings with
/// their parameters, the limits and the path settings.
pub fn config_json(
    config: &Config,
    config_file: Option<&Path>,
    matlab_config: Option<&Path>,
) -> Value {
    let mut categories: Vec<(&str, Value)> = config
        .categories
        .iter()
        .map(|(category, severity)| (category_name(*category), severity_value(*severity)))
        .collect();
    categories.sort_by(|a, b| a.0.cmp(b.0));

    let mut rules: Vec<(&String, &RuleConfig)> = config.rules.iter().collect();
    rules.sort_by(|a, b| a.0.cmp(b.0));
    let rules: Map<String, Value> = rules
        .into_iter()
        .map(|(id, rule)| {
            let value = json!({
                "enabled": rule.enabled,
                "severity": rule.severity.map(severity_name),
                "params": serde_json::to_value(&rule.params).unwrap_or(Value::Null),
            });
            (id.clone(), value)
        })
        .collect();

    // The limits as they apply to this run: the configured ones, else the defaults.
    let limits = config.limits.resolve(Limits::native());
    json!({
        "config_file": config_file.map(|p| p.display().to_string()),
        "matlab_config": matlab_config.map(|p| p.display().to_string()),
        "preset": config.preset.name,
        "experimental": config.experimental,
        "inline_suppression": config.inline_suppression,
        "include": config.include,
        "exclude": config.exclude,
        "respect_gitignore": config.respect_gitignore,
        "force_exclude": config.force_exclude,
        "categories": categories.into_iter().map(|(k, v)| (k.to_string(), v)).collect::<Map<String, Value>>(),
        "rules": rules,
        "limits": {
            "max_file_size": limits.max_file_size,
            "max_tree_depth": limits.max_tree_depth,
            "max_nesting": limits.max_nesting,
            "max_threads": limits.max_threads,
        },
    })
}

fn severity_value(severity: Option<Severity>) -> Value {
    match severity {
        Some(severity) => json!(severity_name(severity)),
        None => json!("off"),
    }
}

fn category_name(category: Category) -> &'static str {
    category.as_str()
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(path: &str, rule: &str, severity: Severity, message: &str) -> Record {
        Record {
            path: path.into(),
            line: 3,
            column: 7,
            rule_id: rule.into(),
            severity,
            message: message.into(),
            fix: None,
        }
    }

    #[test]
    fn annotation_escapes_the_message_and_properties() {
        let r = record("a,b:c.m", "NOSEMI", Severity::Info, "50% done\nnext, here");
        assert_eq!(
            github_annotation(&r, None),
            "::notice file=a%2Cb%3Ac.m,line=3,col=7,title=NOSEMI::50%25 done%0Anext, here"
        );
    }

    #[test]
    fn annotation_maps_severities() {
        let error = github_annotation(&record("a.m", "X", Severity::Error, "m"), None);
        let warning = github_annotation(&record("a.m", "X", Severity::Warning, "m"), None);
        assert!(error.starts_with("::error "));
        assert!(warning.starts_with("::warning "));
    }

    #[test]
    fn annotation_path_is_relative_to_the_workspace() {
        let r = record("/work/repo/src/a.m", "X", Severity::Info, "m");
        assert!(github_annotation(&r, Some(Path::new("/work/repo"))).contains("file=src/a.m,"));
    }

    #[test]
    fn gitlab_fingerprint_ignores_the_line() {
        let mut moved = record("a.m", "NOSEMI", Severity::Info, "m");
        let first = gitlab_report(std::slice::from_ref(&moved));
        moved.line = 99;
        let second = gitlab_report(std::slice::from_ref(&moved));
        assert_eq!(first[0]["fingerprint"], second[0]["fingerprint"]);
        assert_eq!(second[0]["location"]["lines"]["begin"], 99);
    }

    #[test]
    fn gitlab_severity_mapping() {
        let report = gitlab_report(&[
            record("a.m", "X", Severity::Error, "e"),
            record("a.m", "X", Severity::Warning, "w"),
            record("a.m", "X", Severity::Info, "i"),
        ]);
        assert_eq!(report[0]["severity"], "critical");
        assert_eq!(report[1]["severity"], "major");
        assert_eq!(report[2]["severity"], "info");
    }

    #[test]
    fn step_summary_escapes_table_cells() {
        let s = github_step_summary(&[record("a.m", "X", Severity::Warning, "a | b")], 1);
        assert!(s.contains("a \\| b"));
        assert!(s.contains("| warning | 1 |"));
    }

    #[test]
    fn results_json_carries_version_config_and_findings() {
        let config = Config::default();
        let files = vec!["a.m".to_string()];
        let run = RunInfo {
            config_file: None,
            matlab_config: None,
            files: &files,
            files_with_findings: 1,
        };
        let v = results_json(
            &config,
            &run,
            &[record("a.m", "NOSEMI", Severity::Info, "m")],
        );
        assert_eq!(v["schema_version"], JSON_SCHEMA_VERSION);
        assert_eq!(v["malt_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(v["config"]["preset"], config.preset.name);
        assert_eq!(v["summary"]["findings"], 1);
        assert_eq!(v["findings"][0]["rule"], "NOSEMI");
    }
}
