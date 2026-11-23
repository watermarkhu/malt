//! The recorded MATLAB expectations of the `--preset all` comparison, checked against live MATLAB.
//!
//! `fixtures/matlab_expected_all.tsv` is what MATLAB reported with every check enabled
//! (`tests/matlab/generate_checkcode_all.m`) when it was recorded; `fixtures/matlab_release.txt`
//! names the release. This test asks a live MATLAB the same question and compares the answers.
//! `matlab_conformance_all.rs` compares malt `--preset all` with the recorded file; with both
//! passing, malt agrees with live MATLAB up to the allowlist.
//!
//! Two phases, as in `matlab_live.rs`. Generation: `generate_checkcode_all.m` runs `checkcode` with
//! a generated configuration over every fixture in one MATLAB session and writes
//! `target/matlab/checkcode_all.tsv` (the expectations' format), `checkcode_all.release`
//! (`version('-release')`) and `checkcode_all_files.txt`. CI does that with
//! `matlab-actions/run-command`. Comparison (this test): the TSV named by
//! `MALT_MATLAB_CHECKCODE_ALL`, with MATLAB not started, or, without it, MATLAB started here to
//! generate the same files in a temporary directory. `MALT_MATLAB_CHECKCODE_ALL_FILES` names the
//! list of files MATLAB checked.
//!
//! Tests: the release is the recorded release; the live (path, line, ID) findings are the
//! recorded ones; the files MATLAB checked are the files of `fixtures/matlab_files.txt`.
//!
//! Without MATLAB the tests are skipped (see `common::matlab`); `MALT_MATLAB_REQUIRE=1` fails
//! instead. See `docs/VERIFY.md`, 'Preset all against MATLAB'.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use common::expected::{only_in, parse_triples, Triple};
use common::fixtures::{fixtures_dir, listed_files};
use common::matlab::{matlab, matlab_string, MatlabRun};

/// Findings to print in full on a failure; the rest are counted.
const SHOWN: usize = 40;

/// What the live MATLAB reported with every check on: the findings, the release, and (when
/// available) the files it checked. `None` for the files means that comparison is skipped.
struct Live {
    findings: BTreeSet<Triple>,
    release: String,
    files: Option<BTreeSet<String>>,
}

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// The live output, loaded once for all tests. `None` when skipped; an error text when it cannot
/// run.
fn live() -> &'static Result<Option<Live>, String> {
    static LIVE: OnceLock<Result<Option<Live>, String>> = OnceLock::new();
    LIVE.get_or_init(|| load().map_err(|err| format!("{err:#}")))
}

fn live_or_skip() -> Option<&'static Live> {
    match live() {
        Ok(live) => live.as_ref(),
        Err(err) => {
            panic!("the live MATLAB checkcode output (every check on) is not available:\n{err}")
        }
    }
}

/// With `MALT_MATLAB_CHECKCODE_ALL` set, MATLAB is not started; without it, MATLAB is started
/// here to generate the files (the same script), or the test skips.
fn load() -> Result<Option<Live>> {
    let required = env::var("MALT_MATLAB_REQUIRE").is_ok_and(|v| v == "1");
    let Some(path) = env::var_os("MALT_MATLAB_CHECKCODE_ALL") else {
        let Some(run) = matlab() else {
            return Ok(None);
        };
        return generate(&run).map(Some);
    };
    let path = relative_to_repo(Path::new(&path));
    match fs::read_to_string(&path) {
        Ok(text) => {
            eprintln!(
                "live checkcode output (every check on) read from {} (MATLAB not started)",
                path.display()
            );
            Ok(Some(Live {
                findings: parse_triples(&text),
                release: read_release(&path)?,
                files: optional_files()?,
            }))
        }
        Err(err) if required => bail!(
            "MALT_MATLAB_CHECKCODE_ALL: cannot read {}: {err} (MALT_MATLAB_REQUIRE=1)",
            path.display()
        ),
        Err(err) => {
            eprintln!(
                "SKIP: cannot read the live checkcode output {}: {err} (generate it with \
                 generate_checkcode_all.m; MALT_MATLAB_REQUIRE=1 makes this a failure)",
                path.display()
            );
            Ok(None)
        }
    }
}

/// The list of checked files, named by `MALT_MATLAB_CHECKCODE_ALL_FILES`. Unset or unreadable: a
/// failure with `MALT_MATLAB_REQUIRE=1`, else a `SKIP` line and `None`.
fn optional_files() -> Result<Option<BTreeSet<String>>> {
    let required = env::var("MALT_MATLAB_REQUIRE").is_ok_and(|v| v == "1");
    let var = "MALT_MATLAB_CHECKCODE_ALL_FILES";
    let Some(path) = env::var_os(var) else {
        if required {
            bail!("{var} is not set (MALT_MATLAB_REQUIRE=1)");
        }
        eprintln!("SKIP: {var} is not set: the file-list comparison is skipped");
        return Ok(None);
    };
    let path = relative_to_repo(Path::new(&path));
    match fs::read_to_string(&path) {
        Ok(text) => Ok(Some(parse_files(&text))),
        Err(err) if required => bail!(
            "{var}: cannot read {}: {err} (MALT_MATLAB_REQUIRE=1)",
            path.display()
        ),
        Err(err) => {
            eprintln!(
                "SKIP: cannot read {}: {err}: the file-list comparison is skipped",
                path.display()
            );
            Ok(None)
        }
    }
}

fn parse_files(text: &str) -> BTreeSet<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// A relative path is relative to the repository root: cargo runs a test in its crate.
fn relative_to_repo(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let joined = manifest().join("../..").join(path);
    fs::canonicalize(&joined).unwrap_or(joined)
}

/// The release a generation wrote beside its TSV (`checkcode_all.tsv` has `checkcode_all.release`).
fn read_release(tsv: &Path) -> Result<String> {
    let file = tsv.with_extension("release");
    let text = fs::read_to_string(&file).with_context(|| {
        format!(
            "reading {}: the MATLAB release that wrote the TSV (generate_checkcode_all.m writes it)",
            file.display()
        )
    })?;
    Ok(text.trim().to_string())
}

/// Generation with MATLAB: the generator script, as CI runs it, writes into a temporary folder.
fn generate(run: &MatlabRun) -> Result<Live> {
    let dir = tempfile::tempdir()?;
    let out = dir.path().join("checkcode_all.tsv");
    let script = manifest().join("tests/matlab/generate_checkcode_all.m");
    let stdout = run.run_batch_with(
        &format!("run({})", matlab_string(&script)),
        run.timeout(),
        &[("MALT_CHECKCODE_ALL_OUT", out.as_os_str())],
    )?;
    if let Some(summary) = stdout.lines().find(|l| l.starts_with("checkcode_all:")) {
        eprintln!("{summary}");
    }
    let text =
        fs::read_to_string(&out).context("reading the TSV written by generate_checkcode_all.m")?;
    let files = fs::read_to_string(dir.path().join("checkcode_all_files.txt"))
        .context("reading checkcode_all_files.txt (generate_checkcode_all.m writes it)")?;
    Ok(Live {
        findings: parse_triples(&text),
        release: read_release(&out)?,
        files: Some(parse_files(&files)),
    })
}

fn recorded_release() -> Result<String> {
    let path = fixtures_dir().join("matlab_release.txt");
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .with_context(|| format!("{} names no release", path.display()))
}

/// A release as a comparable key: `R2026b`, `2026b` and `2026B` are the same release.
fn release_key(text: &str) -> String {
    text.trim().trim_start_matches(['R', 'r']).to_lowercase()
}

fn recorded_findings() -> BTreeSet<Triple> {
    let text =
        fs::read_to_string(fixtures_dir().join("matlab_expected_all.tsv")).unwrap_or_default();
    parse_triples(&text)
}

/// The report of the differences: per check ID with counts, per file, then the first entries.
/// `-` marks a recorded finding MATLAB no longer reports, `+` a new one.
fn report(recorded: &BTreeSet<Triple>, live: &BTreeSet<Triple>, release: &str) -> String {
    let stale = only_in(recorded, live);
    let new = only_in(live, recorded);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} difference(s) between the recorded all-checks expectations and live MATLAB {release}: \
         {} recorded but no longer reported (-), {} reported but not recorded (+)",
        stale.len() + new.len(),
        stale.len(),
        new.len()
    );
    let mut per_id: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for (_, _, id) in &stale {
        per_id.entry(id.as_str()).or_default().0 += 1;
    }
    for (_, _, id) in &new {
        per_id.entry(id.as_str()).or_default().1 += 1;
    }
    let _ = writeln!(out, "  per check ID (-: recorded only, +: live only):");
    for (id, (minus, plus)) in &per_id {
        let _ = writeln!(out, "    {id:<16} -{minus:<5} +{plus}");
    }
    let mut files: BTreeMap<&str, usize> = BTreeMap::new();
    for (path, _, _) in stale.iter().chain(&new) {
        *files.entry(path.as_str()).or_default() += 1;
    }
    let _ = writeln!(out, "  files with differences: {}", files.len());
    let _ = writeln!(out, "  first differences:");
    let all = stale
        .iter()
        .map(|t| ('-', t))
        .chain(new.iter().map(|t| ('+', t)));
    for (sign, (path, line, id)) in all.clone().take(SHOWN) {
        let _ = writeln!(out, "    {sign} {path}:{line}\t{id}");
    }
    if all.count() > SHOWN {
        let _ = writeln!(out, "    ... and {} more", stale.len() + new.len() - SHOWN);
    }
    out
}

#[test]
fn live_all_checks_release_is_the_recorded_release() {
    let Some(live) = live_or_skip() else {
        return;
    };
    let recorded = recorded_release().unwrap();
    let (want, got) = (release_key(&recorded), release_key(&live.release));
    assert!(
        want == got,
        "expectations recorded with R{want}, live MATLAB is R{got}: re-record or pin the CI \
         release (the reports are expected to drift across releases, so a different release is \
         not a bug in malt)"
    );
    eprintln!("live MATLAB release R{got} matches the recorded release {recorded}");
}

#[test]
fn live_all_checks_agree_with_the_recorded_expectations() {
    let Some(live) = live_or_skip() else {
        return;
    };
    let recorded = recorded_findings();
    if recorded == live.findings {
        eprintln!(
            "live checkcode with every check on (MATLAB {}) agrees with the {} recorded findings",
            live.release,
            recorded.len()
        );
        return;
    }
    let text = report(&recorded, &live.findings, &live.release);
    print!("{text}");
    panic!(
        "{text}\nupdate matlab_expected_all.tsv (tests/fixtures/README.md, 'matlab_expected_all.tsv') \
         or find the fixture or MATLAB change behind it"
    );
}

#[test]
fn live_all_checks_checked_every_listed_file() {
    let Some(live) = live_or_skip() else {
        return;
    };
    let Some(checked) = &live.files else {
        return;
    };
    let listed = listed_files();
    let missed: Vec<&String> = listed.difference(checked).collect();
    let extra: Vec<&String> = checked.difference(&listed).collect();
    assert!(
        missed.is_empty() && extra.is_empty(),
        "the files MATLAB checked differ from tests/fixtures/matlab_files.txt: {} listed but not \
         checked, {} checked but not listed.\n{}",
        missed.len(),
        extra.len(),
        missed
            .iter()
            .map(|f| format!("  - {f}"))
            .chain(extra.iter().map(|f| format!("  + {f}")))
            .take(SHOWN)
            .collect::<Vec<_>>()
            .join("\n"),
    );
    eprintln!(
        "live MATLAB (every check on) checked the {} files of matlab_files.txt",
        listed.len()
    );
}
