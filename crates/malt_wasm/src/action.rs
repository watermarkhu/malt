//! One run of the GitHub Action, as a function of its inputs and a file system.
//!
//! The action's JavaScript is a thin shell: it hands over the raw inputs and three file
//! system callbacks (`stat`, `list`, `read`) and writes back what this module returns.
//! Reading `.malt.toml`, finding the files, linting, filtering by level, formatting the
//! annotations, the step summary and the results file, counting, and deciding whether the
//! step fails all happen here, with the same code as the command line.

use malt_core::files::{self, Fs};
use malt_core::report::{self, Record};
use malt_core::Severity;
use serde::{Deserialize, Serialize};

use crate::wasm::{from_toml_with_str, records_of};

/// The configuration file the action reads, from the root of the workspace.
const CONFIG_FILE: &str = ".malt.toml";

/// The raw inputs of the action, as the runner provides them (all strings).
#[derive(Debug, Default, Deserialize)]
pub struct Inputs {
    #[serde(default)]
    pub paths: String,
    #[serde(default)]
    pub preset: String,
    #[serde(default)]
    pub level: String,
    #[serde(default, rename = "fail-on")]
    pub fail_on: String,
    #[serde(default)]
    pub annotations: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default, rename = "output-json")]
    pub output_json: String,
    /// `GITHUB_WORKSPACE`: annotation paths are made relative to it.
    #[serde(default)]
    pub workspace: String,
}

/// What the shell writes: the log, the job summary, the results file and the outputs.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Outcome {
    /// Workflow commands (annotations, warnings) and log lines, one per line, for stdout.
    pub stdout: String,
    /// Markdown to append to the job summary, when it is on.
    pub summary_markdown: Option<String>,
    /// The JSON results file, when `output-json` is set.
    pub results_json: Option<String>,
    /// The number of findings reported.
    pub findings: usize,
    /// The number of errors.
    pub errors: usize,
    /// The number of warnings.
    pub warnings: usize,
    /// How many files were linted.
    pub files: usize,
    /// Whether the step should fail.
    pub failed: bool,
}

fn rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Info => 2,
    }
}

fn choice(input: &str, name: &str, allowed: &[&str], fallback: &str) -> Result<String, String> {
    let value = input.trim().to_lowercase();
    let value = if value.is_empty() {
        fallback.to_string()
    } else {
        value
    };
    if allowed.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(format!(
            "Input '{name}' must be one of {}, not '{value}'",
            allowed.join(", ")
        ))
    }
}

fn flag(input: &str, name: &str, fallback: bool) -> Result<bool, String> {
    match input.trim().to_lowercase().as_str() {
        "" => Ok(fallback),
        "true" | "yes" | "on" | "1" => Ok(true),
        "false" | "no" | "off" | "0" => Ok(false),
        other => Err(format!(
            "Input '{name}' must be true or false, not '{other}'"
        )),
    }
}

/// A file's text: UTF-8, else Latin-1, as the command line reads it.
fn decode(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => bytes.iter().map(|b| char::from(*b)).collect(),
    }
}

fn record(path: &str, rule: &str, message: String) -> Record {
    Record {
        path: path.to_string(),
        line: 1,
        column: 1,
        rule_id: rule.to_string(),
        severity: Severity::Error,
        message,
        fix: None,
    }
}

/// Run the action over `fs` (rooted at the workspace) with `inputs`. An `Err` is a message
/// for the shell to show as an error; findings, even errors, are an `Ok`.
pub fn run(fs: &dyn Fs, inputs: &Inputs) -> Result<Outcome, String> {
    let level = choice(
        &inputs.level,
        "level",
        &["error", "warning", "info"],
        "info",
    )?;
    let fail_on = choice(
        &inputs.fail_on,
        "fail-on",
        &["error", "warning", "info", "never"],
        "error",
    )?;
    let annotations = flag(&inputs.annotations, "annotations", true)?;
    let summary = flag(&inputs.summary, "summary", true)?;
    let preset = inputs.preset.trim().to_lowercase();
    let names: Vec<&str> = malt_core::preset::PRESETS.iter().map(|p| p.name).collect();
    if !preset.is_empty() && !names.contains(&preset.as_str()) {
        return Err(format!(
            "Input 'preset' must be one of {}, not '{preset}'",
            names.join(", ")
        ));
    }
    let named: Vec<String> = inputs.paths.split_whitespace().map(String::from).collect();
    let named = if named.is_empty() {
        vec![".".to_string()]
    } else {
        named
    };

    let mut out = String::new();
    let mut say = |line: &str| {
        out.push_str(line);
        out.push('\n');
    };

    // The configuration: `.malt.toml` of the workspace, with the inputs over it.
    let config_text = fs.read(CONFIG_FILE).map(|b| decode(&b));
    let overrides = if preset.is_empty() {
        "{}".to_string()
    } else {
        serde_json::json!({ "preset": preset }).to_string()
    };
    let mut linter = from_toml_with_str(config_text.as_deref().unwrap_or(""), &overrides)?;
    for warning in linter.config_warnings() {
        say(&format!(
            "::warning file={CONFIG_FILE},title=malt configuration::{}",
            report::escape_data(warning)
        ));
    }
    files::validate_patterns(linter.config())?;

    let found = files::discover(fs, &named, linter.config());
    for warning in &found.warnings {
        say(&format!("::warning::{}", report::escape_data(warning)));
    }

    let linted = found.files.len() - found.missing.len();
    let mut records: Vec<Record> = Vec::new();
    for file in &found.files {
        // A named path that does not exist is a NOFIL error, in the place it was named.
        if found.missing.contains(file) {
            records.push(record(
                file,
                "NOFIL",
                format!("Unable to open file {file}. File is not found."),
            ));
            continue;
        }
        match fs.read(file) {
            Some(bytes) => records.extend(records_of(&mut linter, &decode(&bytes), file)),
            None => records.push(record(
                file,
                "RDERR",
                format!("Unable to read file {file}."),
            )),
        }
    }
    let shown = |severity: Severity| rank(severity) <= rank_of(&level);
    records.retain(|r| shown(r.severity));

    if found.files.is_empty() {
        say(&format!(
            "::warning::malt found no MATLAB files under {}",
            named.join(", ")
        ));
    }
    if annotations && !records.is_empty() {
        let workspace = (!inputs.workspace.is_empty()).then_some(inputs.workspace.as_str());
        for r in &records {
            say(&report::github_annotation(
                r,
                workspace.map(std::path::Path::new),
            ));
        }
    }
    let summary_markdown = summary.then(|| report::github_step_summary(&records, linted));

    let listed = &found.files;
    let results_json = if inputs.output_json.trim().is_empty() {
        None
    } else {
        let mut with_findings: Vec<&str> = records.iter().map(|r| r.path.as_str()).collect();
        with_findings.sort_unstable();
        with_findings.dedup();
        let run = report::RunInfo {
            config_file: config_text
                .is_some()
                .then(|| std::path::Path::new(CONFIG_FILE)),
            matlab_config: None,
            files: listed,
            files_with_findings: with_findings.len(),
        };
        let json = report::results_json(linter.config(), &run, &records);
        Some(serde_json::to_string_pretty(&json).map_err(|e| e.to_string())? + "\n")
    };

    let count = |s: Severity| records.iter().filter(|r| r.severity == s).count();
    let (errors, warnings, infos) = (
        count(Severity::Error),
        count(Severity::Warning),
        count(Severity::Info),
    );
    let failing = if fail_on == "never" {
        0
    } else {
        records
            .iter()
            .filter(|r| rank(r.severity) <= rank_of(&fail_on))
            .count()
    };
    let plural = |n: usize| if n == 1 { "" } else { "s" };
    say(&format!(
        "malt {}: linted {} file{}, {} finding{} ({errors} error, {warnings} warning, {infos} info)",
        env!("CARGO_PKG_VERSION"),
        linted,
        plural(linted),
        records.len(),
        plural(records.len()),
    ));
    if failing > 0 {
        say(&format!(
            "::error title=malt::{failing} finding{} at or above '{fail_on}'",
            plural(failing)
        ));
    }

    Ok(Outcome {
        stdout: out,
        summary_markdown,
        results_json,
        findings: records.len(),
        errors,
        warnings,
        files: linted,
        failed: failing > 0,
    })
}

fn rank_of(level: &str) -> u8 {
    match level {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use malt_core::files::MemFs;

    // NOPTS (info) on line 1, EOLPAR (error) on line 2.
    const BAD: &str = "x = 1\nfunction f(\n";

    fn go(fs: &MemFs, inputs: Inputs) -> Outcome {
        run(fs, &inputs).expect("the inputs are valid")
    }

    fn inputs(pairs: &[(&str, &str)]) -> Inputs {
        let map: serde_json::Map<String, serde_json::Value> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
            .collect();
        serde_json::from_value(serde_json::Value::Object(map)).unwrap()
    }

    #[test]
    fn annotates_summarizes_counts_and_fails_on_errors() {
        let fs = MemFs::new(&[("a.m", BAD)]);
        let outcome = go(&fs, inputs(&[("workspace", "/w")]));
        assert!(outcome
            .stdout
            .contains("::notice file=a.m,line=1,col=3,title=NOPTS::"));
        assert!(outcome
            .stdout
            .contains("::error file=a.m,line=2,col=11,title=EOLPAR::"));
        assert_eq!(
            (
                outcome.findings,
                outcome.errors,
                outcome.warnings,
                outcome.files
            ),
            (2, 1, 0, 1)
        );
        assert!(outcome.failed);
        assert!(outcome.summary_markdown.unwrap().starts_with("## malt"));
        assert!(outcome.results_json.is_none());
    }

    #[test]
    fn fail_on_decides_the_exit_and_level_the_reporting() {
        let fs = MemFs::new(&[("a.m", "x = 1\n")]);
        assert!(!go(&fs, inputs(&[])).failed);
        assert!(go(&fs, inputs(&[("fail-on", "info")])).failed);
        let quiet = go(&fs, inputs(&[("level", "warning"), ("fail-on", "info")]));
        assert_eq!((quiet.findings, quiet.failed), (0, false));
        let bad = MemFs::new(&[("a.m", BAD)]);
        assert!(!go(&bad, inputs(&[("fail-on", "never")])).failed);
    }

    #[test]
    fn annotations_and_summary_can_be_switched_off() {
        let fs = MemFs::new(&[("a.m", BAD)]);
        let outcome = go(&fs, inputs(&[("annotations", "false"), ("summary", "no")]));
        assert!(!outcome.stdout.contains("::error file="));
        assert!(outcome.summary_markdown.is_none());
    }

    #[test]
    fn the_preset_input_overrules_the_toml() {
        let fs = MemFs::new(&[
            ("a.m", "function f(x)\n  y = 1;\nend\n"),
            (".malt.toml", "[lint]\npreset = \"mathworks\"\n"),
        ]);
        assert!(!go(&fs, inputs(&[])).stdout.contains("MP_"));
        assert!(go(&fs, inputs(&[("preset", "monkeyproof")]))
            .stdout
            .contains("title=MP_"));
    }

    #[test]
    fn the_toml_decides_which_files_are_found() {
        let fs = MemFs::new(&[
            ("a.m", BAD),
            ("gen/g.m", BAD),
            (".malt.toml", "[lint]\nextend_exclude = [\"gen/\"]\n"),
        ]);
        assert_eq!(go(&fs, inputs(&[])).files, 1);
    }

    #[test]
    fn paths_narrow_the_run_and_a_missing_one_is_a_nofil_error() {
        let fs = MemFs::new(&[("src/a.m", "x = 1;\n"), ("other/b.m", BAD)]);
        let narrow = go(&fs, inputs(&[("paths", "src")]));
        assert_eq!((narrow.findings, narrow.files), (0, 1));
        let outcome = go(&fs, inputs(&[("paths", "src\nmissing.m")]));
        assert_eq!((outcome.errors, outcome.failed), (1, true));
        assert!(outcome
            .stdout
            .contains("title=NOFIL::Unable to open file missing.m"));
    }

    #[test]
    fn output_json_holds_the_version_the_configuration_and_the_findings() {
        let fs = MemFs::new(&[
            ("a.m", BAD),
            (".malt.toml", "[lint]\nextend_exclude = [\"gen/\"]\n"),
        ]);
        let outcome = go(&fs, inputs(&[("output-json", "results.json")]));
        let value: serde_json::Value =
            serde_json::from_str(&outcome.results_json.unwrap()).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["malt_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(value["config"]["config_file"], ".malt.toml");
        assert!(value["config"]["exclude"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == "gen/"));
        assert_eq!(value["files"], serde_json::json!(["a.m"]));
        let rules: Vec<&str> = value["findings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["rule"].as_str().unwrap())
            .collect();
        assert_eq!(rules, ["NOPTS", "EOLPAR"]);
    }

    #[test]
    fn an_empty_workspace_warns_and_passes() {
        let outcome = go(&MemFs::new(&[]), inputs(&[]));
        assert_eq!(
            (outcome.findings, outcome.files, outcome.failed),
            (0, 0, false)
        );
        assert!(outcome
            .stdout
            .contains("::warning::malt found no MATLAB files"));
    }

    #[test]
    fn a_file_that_is_not_utf8_is_read_as_latin1() {
        let fs = MemFs::new(&[]).with_bytes("l.m", b"% caf\xe9\nx = 1;\n");
        assert_eq!(go(&fs, inputs(&[])).files, 1);
    }

    #[test]
    fn config_warnings_become_warning_annotations() {
        let fs = MemFs::new(&[
            ("a.m", "x = 1;\n"),
            (".malt.toml", "[lint.rules]\nNOSUCHRULE = \"off\"\n"),
        ]);
        assert!(go(&fs, inputs(&[]))
            .stdout
            .contains("::warning file=.malt.toml"));
    }

    #[test]
    fn bad_inputs_are_errors_that_name_the_input() {
        let fs = MemFs::new(&[("a.m", "x = 1;\n")]);
        for (name, value) in [
            ("preset", "strict"),
            ("level", "debug"),
            ("fail-on", "always"),
            ("summary", "maybe"),
        ] {
            let error = run(&fs, &inputs(&[(name, value)])).unwrap_err();
            assert!(error.contains(&format!("'{name}'")), "{error}");
        }
    }

    #[test]
    fn an_invalid_pattern_is_an_error() {
        let fs = MemFs::new(&[("a.m", ""), (".malt.toml", "[lint]\nexclude = [\"{a,\"]\n")]);
        assert!(run(&fs, &inputs(&[])).unwrap_err().contains("{a,"));
    }

    #[test]
    fn an_unreadable_file_is_an_rderr() {
        struct Locked;
        impl Fs for Locked {
            fn kind(&self, p: &str) -> Option<files::Kind> {
                Some(if p == "." {
                    files::Kind::Dir
                } else {
                    files::Kind::File
                })
            }
            fn list(&self, _: &str) -> Result<Vec<(String, files::Kind)>, String> {
                Ok(vec![("a.m".into(), files::Kind::File)])
            }
            fn read(&self, _: &str) -> Option<Vec<u8>> {
                None
            }
        }
        let outcome = run(&Locked, &Inputs::default()).unwrap();
        assert!(outcome
            .stdout
            .contains("title=RDERR::Unable to read file a.m."));
        assert!(outcome.failed);
    }
}
