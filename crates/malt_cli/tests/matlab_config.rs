//! malt reads MATLAB's `codeAnalyzerConfiguration.json`. The cases under
//! `tests/matlab_config/` are JSON configurations with a subject file each, and
//! `expected.tsv` holds what MATLAB R2026b's `codeIssues` reports for them (line,
//! check ID and severity only). malt must report the same findings. The other tests
//! cover what MATLAB does not define for malt: warnings, strict mode, discovery, the
//! TOML layering and `ignoredFiles`.

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::matlab::{matlab, matlab_string};

/// A finding as malt prints it: file, line, check ID and severity letter (`E`, `W`, `I`).
type Finding = (String, usize, String, char);

/// What MATLAB reports for one case.
enum Expected {
    Findings(Vec<Finding>),
    /// MATLAB refuses the configuration (codeIssues throws).
    Refused,
    /// MATLAB reports nothing for the subject file.
    Silent,
}

/// The cases where malt differs from MATLAB by design or by gap: custom checks (`rule`),
/// and MNCSN, which malt reports at the function's line where MATLAB reports the
/// nested statement (c19: the rule lives in malt_rules).
const KNOWN_GAPS: &[&str] = &["c18_custom_template_unsupported", "c19_limit_mncsn"];

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/matlab_config")
}

/// The recorded expectations of every case, keyed by case name, with the subject files
/// of each case in order.
fn expectations() -> BTreeMap<String, (Vec<String>, Expected)> {
    let text = fs::read_to_string(dir().join("expected.tsv")).expect("expected.tsv");
    let mut cases: BTreeMap<String, (Vec<String>, Vec<Finding>, bool)> = BTreeMap::new();
    for line in text
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let cols: Vec<&str> = line.split('\t').collect();
        let (case, file) = (cols[0], cols[1]);
        let entry = cases.entry(case.to_string()).or_default();
        if !entry.0.iter().any(|f| f == file) {
            entry.0.push(file.to_string());
        }
        match cols.get(2).copied() {
            Some("NONE") => {}
            Some("ERROR") => entry.2 = true,
            Some(line_no) => {
                let letter = match cols[4] {
                    "error" => 'E',
                    "warning" => 'W',
                    _ => 'I',
                };
                entry.1.push((
                    file.to_string(),
                    line_no.parse().expect("line number"),
                    cols[3].to_string(),
                    letter,
                ));
            }
            None => panic!("bad row: {line}"),
        }
    }
    cases
        .into_iter()
        .map(|(case, (files, findings, refused))| {
            let expected = if refused {
                Expected::Refused
            } else if findings.is_empty() {
                Expected::Silent
            } else {
                Expected::Findings(findings)
            };
            (case, (files, expected))
        })
        .collect()
}

/// A project folder with the case's configuration as `resources/codeAnalyzerConfiguration.json`
/// (none for `c00_default`) and its subject files.
fn project(case: &str, files: &[String]) -> tempfile::TempDir {
    let project = tempfile::tempdir().expect("temp dir");
    let json = dir().join(format!("{case}.json"));
    if json.exists() {
        let resources = project.path().join("resources");
        fs::create_dir(&resources).expect("resources");
        fs::copy(&json, resources.join("codeAnalyzerConfiguration.json")).expect("copy json");
    }
    for file in files {
        fs::copy(dir().join(file), project.path().join(file)).expect("copy subject");
    }
    project
}

fn malt(project: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_malt"))
        .current_dir(project)
        .args(args)
        .output()
        .expect("failed to run malt")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The findings in malt's output, in the order printed.
fn findings(stdout: &str) -> Vec<Finding> {
    stdout
        .lines()
        .filter_map(|line| {
            let (place, rest) = line.split_once(" [")?;
            let mut parts = place.rsplitn(3, ':');
            let _column = parts.next()?;
            let line_no: usize = parts.next()?.parse().ok()?;
            let file = parts.next()?.to_string();
            let letter = rest.chars().next()?;
            let id = rest.split_once("] ")?.1.split_once(':')?.0.to_string();
            Some((file, line_no, id, letter))
        })
        .collect()
}

fn sorted(mut list: Vec<Finding>) -> Vec<Finding> {
    list.sort();
    list
}

#[test]
fn findings_match_matlab_for_every_case() {
    for (case, (files, expected)) in expectations() {
        if KNOWN_GAPS.contains(&case.as_str()) {
            continue;
        }
        let project = project(&case, &files);
        let out = malt(
            project.path(),
            &files.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        let stderr = text(&out.stderr);
        match expected {
            Expected::Refused => {
                assert_eq!(out.status.code(), Some(2), "{case}: stderr: {stderr}");
                assert!(
                    stderr.contains("invalid MATLAB configuration"),
                    "{case}: {stderr}"
                );
            }
            Expected::Silent => {
                assert_eq!(out.status.code(), Some(0), "{case}: stderr: {stderr}");
                assert!(out.stdout.is_empty(), "{case}: {}", text(&out.stdout));
            }
            Expected::Findings(list) => {
                let got = sorted(findings(&text(&out.stdout)));
                assert_eq!(got, sorted(list), "{case}: stdout: {}", text(&out.stdout));
            }
        }
    }
}

#[test]
fn explicit_json_config_gives_the_same_findings() {
    for (case, (files, expected)) in expectations() {
        let Expected::Findings(list) = expected else {
            continue;
        };
        if KNOWN_GAPS.contains(&case.as_str()) || !dir().join(format!("{case}.json")).exists() {
            continue;
        }
        let project = project(&case, &files);
        let args: Vec<&str> = ["--config", "resources/codeAnalyzerConfiguration.json"]
            .into_iter()
            .chain(files.iter().map(String::as_str))
            .collect();
        let out = malt(project.path(), &args);
        let got = sorted(findings(&text(&out.stdout)));
        assert_eq!(got, sorted(list), "{case}");
    }
}

#[test]
fn custom_templates_are_warned_and_not_applied() {
    let (files, _) = expectations()
        .remove("c18_custom_template_unsupported")
        .expect("c18");
    let project = project("c18_custom_template_unsupported", &files);
    let out = malt(project.path(), &["disp_c.m"]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert!(stderr.contains("checks.noDisp"), "{stderr}");
    assert!(
        stderr.contains("custom checks are not supported"),
        "{stderr}"
    );
    assert!(
        !text(&out.stdout).contains("noDisp"),
        "{}",
        text(&out.stdout)
    );
}

#[test]
fn strict_config_makes_matlab_warnings_an_error() {
    let (files, _) = expectations()
        .remove("c18_custom_template_unsupported")
        .expect("c18");
    let project = project("c18_custom_template_unsupported", &files);
    let out = malt(project.path(), &["--strict-config", "disp_c.m"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("(--strict-config)"),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn ignored_settings_are_warned_and_the_rest_applies() {
    let (files, _) = expectations()
        .remove("c12_invalid_severity_ignored")
        .expect("c12");
    let project = project("c12_invalid_severity_ignored", &files);
    let out = malt(
        project.path(),
        &files.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    let stderr = text(&out.stderr);
    assert!(stderr.contains("checks.NOPRT.severity"), "{stderr}");
    assert_eq!(out.status.code(), Some(1), "{stderr}");
}

#[test]
fn malformed_json_is_an_error_naming_the_file() {
    let (files, _) = expectations().remove("c17_malformed_json").expect("c17");
    let project = project("c17_malformed_json", &files);
    let out = malt(project.path(), &["func_a.m"]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("resources/codeAnalyzerConfiguration.json"),
        "{stderr}"
    );
    assert!(stderr.contains("not valid JSON"), "{stderr}");
}

#[test]
fn toml_without_matlab_config_does_not_use_the_json_file() {
    let project = project("c01_disable_noprt", &["func_a.m".to_string()]);
    fs::write(project.path().join(".malt.toml"), "[lint]\nexclude = []\n").unwrap();
    let out = malt(project.path(), &["-v", "func_a.m"]);
    let stdout = text(&out.stdout);
    let stderr = text(&out.stderr);
    assert!(stdout.contains("NOPRT"), "{stdout}");
    assert!(
        stderr.contains("not using resources/codeAnalyzerConfiguration.json"),
        "{stderr}"
    );
}

#[test]
fn toml_layers_over_the_matlab_config_key_by_key() {
    let project = project("c07_naming_variable", &["func_a.m".to_string()]);
    fs::write(
        project.path().join(".malt.toml"),
        "[lint]\nmatlab_config = \"resources/codeAnalyzerConfiguration.json\"\n\n\
         [lint.rules.\"naming.variable.maxLength\"]\nmax = 10\n",
    )
    .unwrap();
    let out = malt(project.path(), &["func_a.m"]);
    let got = sorted(findings(&text(&out.stdout)));
    let max_length: Vec<usize> = got
        .iter()
        .filter(|f| f.2 == "naming.variable.maxLength")
        .map(|f| f.1)
        .collect();
    assert_eq!(max_length, vec![2], "{}", text(&out.stdout));
    assert!(got
        .iter()
        .any(|f| f.2 == "naming.variable.casing" && f.1 == 3));
}

#[test]
fn ignored_files_are_skipped_relative_to_the_resources_parent() {
    let project = tempfile::tempdir().expect("temp dir");
    let resources = project.path().join("resources");
    fs::create_dir_all(&resources).unwrap();
    fs::write(
        resources.join("codeAnalyzerConfiguration.json"),
        r#"{"schemaVersion":"1.1.0","ignoredFiles":["sub/*.m"]}"#,
    )
    .unwrap();
    let subject = fs::read_to_string(dir().join("func_a.m")).unwrap();
    fs::write(project.path().join("func_a.m"), &subject).unwrap();
    fs::create_dir(project.path().join("sub")).unwrap();
    fs::write(project.path().join("sub/func_a.m"), &subject).unwrap();

    let out = malt(project.path(), &["-v", "func_a.m", "sub/func_a.m"]);
    let got = findings(&text(&out.stdout));
    assert!(!got.is_empty());
    assert!(got.iter().all(|f| f.0 == "func_a.m"), "{got:?}");
    assert!(text(&out.stderr).contains("skipped 1 file (ignoredFiles)"));
}

/// Live variant: MATLAB's `codeIssues` on each case, now, against malt. Skipped without
/// MATLAB (see `common::matlab`).
#[test]
fn live_matlab_agrees_with_malt_on_every_case() {
    let Some(matlab) = matlab() else {
        return;
    };
    let mut code = String::new();
    let mut projects = Vec::new();
    for (case, (files, _)) in expectations() {
        if KNOWN_GAPS.contains(&case.as_str()) {
            continue;
        }
        let project = project(&case, &files);
        for file in &files {
            let key = format!("{case}/{file}");
            let path = project.path().join(file);
            code.push_str(&format!(
                "matlab.codeanalysis.refreshConfiguration; try, t = codeIssues({}).Issues; for r = 1:height(t), fprintf('%s|%d|%s|%s\\n', {}, t.LineStart(r), string(t.CheckID(r)), string(t.Severity(r))); end, catch, fprintf('%s|ERROR\\n', {}); end; ",
                matlab_string(&path),
                matlab_string(Path::new(&key)),
                matlab_string(Path::new(&key)),
            ));
        }
        projects.push((case, files, project));
    }
    let output = matlab
        .run_batch(&code, matlab.timeout())
        .expect("codeIssues in MATLAB");

    let mut live: BTreeMap<String, Vec<Finding>> = BTreeMap::new();
    let mut refused = Vec::new();
    for line in output.lines() {
        let cols: Vec<&str> = line.split('|').collect();
        let Some((case, file)) = cols[0].split_once('/') else {
            continue;
        };
        if cols.get(1) == Some(&"ERROR") {
            refused.push(cols[0].to_string());
            continue;
        }
        if cols.len() == 4 {
            let letter = match cols[3] {
                "error" => 'E',
                "warning" => 'W',
                _ => 'I',
            };
            live.entry(case.to_string()).or_default().push((
                file.to_string(),
                cols[1].parse().expect("line"),
                cols[2].to_string(),
                letter,
            ));
        }
    }

    for (case, files, project) in projects {
        let args: Vec<&str> = files.iter().map(String::as_str).collect();
        let out = malt(project.path(), &args);
        let key = format!("{case}/");
        if refused.iter().any(|k| k.starts_with(&key)) {
            assert_eq!(out.status.code(), Some(2), "{case}");
            continue;
        }
        let want = sorted(live.remove(&case).unwrap_or_default());
        let got = sorted(findings(&text(&out.stdout)));
        assert_eq!(got, want, "{case}: malt against live MATLAB");
    }
}
