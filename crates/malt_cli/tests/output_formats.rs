//! `--output github|gitlab`, `--log-level` and `--output-json`: what each writes, and that
//! the text lines can be silenced without losing annotations, the report or the exit code.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

/// NOPTS (info) on line 1 and EOLPAR (error) on line 2.
const SOURCE: &str = "x = 1\nfunction f(\n";

fn malt(dir: &Path, args: &[&str], envs: &[(&str, &Path)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_malt"));
    command
        .current_dir(dir)
        .args(args)
        .env_remove("MALT_CONFIG")
        .env_remove("GITHUB_STEP_SUMMARY")
        .env_remove("GITHUB_WORKSPACE");
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("failed to run malt")
}

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    fs::write(dir.path().join("a.m"), SOURCE).unwrap();
    dir
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn github_writes_annotations_and_the_step_summary() {
    let dir = project();
    let summary = dir.path().join("summary.md");
    let out = malt(
        dir.path(),
        &["--output", "github", "a.m"],
        &[
            ("GITHUB_STEP_SUMMARY", &summary),
            ("GITHUB_WORKSPACE", dir.path()),
        ],
    );
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stdout.contains("::notice file=a.m,line=1,col=3,title=NOPTS::"),
        "{stdout}"
    );
    assert!(
        stdout.contains("::error file=a.m,line=2,col=11,title=EOLPAR::"),
        "{stdout}"
    );
    let md = fs::read_to_string(&summary).unwrap();
    assert!(md.starts_with("## malt"), "{md}");
    assert!(
        md.contains("| error | 1 |") && md.contains("| info | 1 |"),
        "{md}"
    );
    assert!(md.contains("a.m"), "{md}");
}

#[test]
fn github_without_a_summary_file_only_annotates() {
    let dir = project();
    let out = malt(dir.path(), &["--output", "github", "a.m"], &[]);
    assert!(text(&out.stdout).contains("::error "));
    assert!(out.status.code() == Some(1));
}

#[test]
fn log_level_off_silences_text_lines_but_not_annotations_or_the_exit_code() {
    let dir = project();
    let out = malt(
        dir.path(),
        &["--output", "github", "--log-level", "off", "a.m"],
        &[],
    );
    let stdout = text(&out.stdout);
    assert!(
        !stdout.lines().any(|l| l.starts_with("a.m:")),
        "no text lines expected:\n{stdout}"
    );
    assert!(stdout.contains("::error "), "{stdout}");
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn log_level_filters_text_lines_by_severity_in_the_default_format() {
    let dir = project();
    let out = malt(dir.path(), &["--log-level", "error", "a.m"], &[]);
    let stdout = text(&out.stdout);
    assert!(stdout.contains("EOLPAR"), "{stdout}");
    assert!(!stdout.contains("NOPTS"), "{stdout}");
    assert!(
        stdout.contains("Found 2 issues"),
        "the count is not filtered:\n{stdout}"
    );
}

#[test]
fn gitlab_prints_a_code_quality_report_and_moves_text_to_stderr() {
    let dir = project();
    let out = malt(dir.path(), &["--output", "gitlab", "a.m"], &[]);
    let report: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout is exactly the JSON report");
    let issues = report.as_array().unwrap();
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0]["check_name"], "NOPTS");
    assert_eq!(issues[0]["severity"], "info");
    assert_eq!(issues[1]["severity"], "critical");
    assert_eq!(issues[1]["location"]["path"], "a.m");
    assert_eq!(issues[1]["location"]["lines"]["begin"], 2);
    assert!(issues[0]["fingerprint"].as_str().unwrap().len() >= 16);
    let stderr = text(&out.stderr);
    assert!(
        stderr.contains("a.m:1:3") && stderr.contains("Found 2 issues"),
        "{stderr}"
    );
}

#[test]
fn gitlab_report_of_a_clean_run_is_an_empty_list() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ok.m"), "x = 1;\n").unwrap();
    let out = malt(dir.path(), &["--output", "gitlab", "ok.m"], &[]);
    assert_eq!(out.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report, serde_json::json!([]));
}

#[test]
fn output_json_holds_the_version_configuration_and_findings() {
    let dir = project();
    fs::write(
        dir.path().join(".malt.toml"),
        "[lint]\nextend_exclude = [\"gen/\"]\n\n[lint.rules.NOPTS]\nseverity = \"warn\"\n",
    )
    .unwrap();
    let out = malt(
        dir.path(),
        &["--output-json", "results.json", "--log-level", "off", "a.m"],
        &[],
    );
    assert_eq!(out.status.code(), Some(1));
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join("results.json")).unwrap())
            .unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["malt_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(value["config"]["preset"], "mathworks");
    assert_eq!(value["config"]["config_file"], ".malt.toml");
    assert_eq!(value["config"]["rules"]["NOPTS"]["severity"], "warning");
    assert!(value["config"]["exclude"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p == "gen/"));
    assert_eq!(value["files"], serde_json::json!(["a.m"]));
    assert_eq!(value["summary"]["by_severity"]["error"], 1);
    let findings = value["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0]["rule"], "NOPTS");
    assert_eq!(findings[0]["fix"], "safe");
    assert_eq!(findings[1]["severity"], "error");
}

#[test]
fn output_json_is_written_even_when_there_are_no_findings() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ok.m"), "x = 1;\n").unwrap();
    let out = malt(dir.path(), &["--output-json", "r.json", "ok.m"], &[]);
    assert_eq!(out.status.code(), Some(0));
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join("r.json")).unwrap()).unwrap();
    assert_eq!(value["summary"]["findings"], 0);
    assert_eq!(value["findings"], serde_json::json!([]));
}

#[test]
fn output_json_combines_with_an_output_format() {
    let dir = project();
    let out = malt(
        dir.path(),
        &["--output", "gitlab", "--output-json", "r.json", "a.m"],
        &[],
    );
    assert!(serde_json::from_slice::<serde_json::Value>(&out.stdout).is_ok());
    assert!(dir.path().join("r.json").is_file());
}

#[test]
fn an_unwritable_json_path_is_an_error() {
    let dir = project();
    let out = malt(
        dir.path(),
        &["--output-json", "no/such/dir/r.json", "a.m"],
        &[],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("results file"));
}
