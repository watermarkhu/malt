//! The recorded MATLAB expectations checked against live MATLAB, on the same fixtures.
//!
//! `fixtures/matlab_expected.tsv` is what MATLAB reported when it was recorded
//! (`fixtures/matlab_release.txt` names that release). This test asks a live MATLAB the same
//! question and compares the answers, so it catches expectations gone stale and drift of MATLAB's
//! reports across releases. `matlab_conformance.rs` compares malt with the recorded file; with both
//! passing, malt agrees with live MATLAB.
//!
//! Two phases, as in `matlab_namespace.rs`. Generation: `tests/matlab/generate_checkcode.m` runs
//! `checkcode` over every fixture in one MATLAB session and writes `target/matlab/checkcode.tsv`
//! (the file's format) and `checkcode.release` (`version('-release')`). CI does that with
//! `matlab-actions/run-command`, the only licensed way to run MATLAB on a public repository.
//! Comparison (this test): the TSV named by `MALT_MATLAB_CHECKCODE`, with MATLAB not started, or,
//! without it, MATLAB started here to generate the same files in a temporary directory.
//!
//! Two tests:
//!
//! - the release that generated the live file is the recorded release (a separate failure, because
//!   drift in reports across releases is expected: re-record, or pin the CI release);
//! - the live (path, line, ID) findings are the recorded ones: a finding MATLAB no longer reports
//!   is a stale expectation, a new one is either a stale expectation or a change in MATLAB.
//!
//! Without MATLAB the tests are skipped (see `common::matlab`); `MALT_MATLAB_REQUIRE=1` fails
//! instead. See `docs/VERIFY.md`, 'Live checkcode check'.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use common::expected::{only_in, parse_triples, Triple};
use common::matlab::{matlab, matlab_string, MatlabRun};

/// Findings to print in full on a failure; the rest are counted.
const SHOWN: usize = 40;

/// What the live MATLAB reported: the findings, and the release that reported them.
struct Live {
    findings: BTreeSet<Triple>,
    release: String,
}

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn fixtures() -> PathBuf {
    manifest().join("tests/fixtures")
}

/// The live MATLAB output, loaded once for both tests (they run in parallel in one binary).
/// `None` when the test is skipped; an error text when it cannot run.
fn live() -> &'static Result<Option<Live>, String> {
    static LIVE: OnceLock<Result<Option<Live>, String>> = OnceLock::new();
    LIVE.get_or_init(|| load().map_err(|err| format!("{err:#}")))
}

/// The live output, or `None` when the test is skipped. Panics on a failure to produce it.
fn live_or_skip() -> Option<&'static Live> {
    match live() {
        Ok(live) => live.as_ref(),
        Err(err) => panic!("the live MATLAB checkcode output is not available:\n{err}"),
    }
}

/// Where the live output comes from. With `MALT_MATLAB_CHECKCODE` set, MATLAB is not started.
/// Without it, MATLAB is started here to generate the files (the same script), or the test skips.
fn load() -> Result<Option<Live>> {
    let required = env::var("MALT_MATLAB_REQUIRE").is_ok_and(|v| v == "1");
    let Some(path) = env::var_os("MALT_MATLAB_CHECKCODE") else {
        let Some(run) = matlab() else {
            return Ok(None);
        };
        return generate(&run).map(Some);
    };
    let path = relative_to_repo(Path::new(&path));
    match fs::read_to_string(&path) {
        Ok(text) => {
            let release = read_release(&path)?;
            eprintln!(
                "live checkcode output read from {} (MATLAB not started)",
                path.display()
            );
            Ok(Some(Live {
                findings: parse_triples(&text),
                release,
            }))
        }
        Err(err) if required => bail!(
            "MALT_MATLAB_CHECKCODE: cannot read {}: {err} (MALT_MATLAB_REQUIRE=1)",
            path.display()
        ),
        Err(err) => {
            eprintln!(
                "SKIP: cannot read the live checkcode output {}: {err} (generate it with \
                 generate_checkcode.m; MALT_MATLAB_REQUIRE=1 makes this a failure)",
                path.display()
            );
            Ok(None)
        }
    }
}

/// A relative path is relative to the repository root: cargo runs a test in its crate.
fn relative_to_repo(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let joined = manifest().join("../..").join(path);
    fs::canonicalize(&joined).unwrap_or(joined)
}

/// The release a generation wrote beside its TSV (`checkcode.tsv` has `checkcode.release`).
/// A TSV without it cannot be checked against a release, so it is an error, not a skip.
fn read_release(tsv: &Path) -> Result<String> {
    let file = tsv.with_extension("release");
    let text = fs::read_to_string(&file).with_context(|| {
        format!(
            "reading {}: the MATLAB release that wrote the TSV (generate_checkcode.m writes it)",
            file.display()
        )
    })?;
    Ok(text.trim().to_string())
}

/// Generation with MATLAB: the generator script, as CI runs it, writes into a temporary folder.
fn generate(run: &MatlabRun) -> Result<Live> {
    let dir = tempfile::tempdir()?;
    let out = dir.path().join("checkcode.tsv");
    let script = manifest().join("tests/matlab/generate_checkcode.m");
    let stdout = run.run_batch_with(
        &format!("run({})", matlab_string(&script)),
        run.timeout(),
        &[("MALT_CHECKCODE_OUT", out.as_os_str())],
    )?;
    if let Some(summary) = stdout.lines().find(|l| l.starts_with("checkcode:")) {
        eprintln!("{summary}");
    }
    let text =
        fs::read_to_string(&out).context("reading the TSV written by generate_checkcode.m")?;
    Ok(Live {
        findings: parse_triples(&text),
        release: read_release(&out)?,
    })
}

/// The release the expectations were recorded with (`fixtures/matlab_release.txt`).
fn recorded_release() -> Result<String> {
    let path = fixtures().join("matlab_release.txt");
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
    let text = fs::read_to_string(fixtures().join("matlab_expected.tsv")).unwrap_or_default();
    parse_triples(&text)
}

/// The report of the differences: grouped by check ID with counts, then per file, then the first
/// entries. `-` marks a recorded finding MATLAB no longer reports, `+` a new one.
fn report(recorded: &BTreeSet<Triple>, live: &BTreeSet<Triple>, release: &str) -> String {
    let stale = only_in(recorded, live);
    let new = only_in(live, recorded);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} difference(s) between the recorded expectations and live MATLAB {release}: \
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
    for (path, count) in &files {
        let _ = writeln!(out, "    {path} ({count})");
    }
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
fn live_matlab_release_is_the_recorded_release() {
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
fn live_checkcode_agrees_with_the_recorded_expectations() {
    let Some(live) = live_or_skip() else {
        return;
    };
    let recorded = recorded_findings();
    if recorded == live.findings {
        eprintln!(
            "live checkcode (MATLAB {}) agrees with the {} recorded findings",
            live.release,
            recorded.len()
        );
        return;
    }
    let text = report(&recorded, &live.findings, &live.release);
    print!("{text}");
    panic!(
        "{text}\nupdate matlab_expected.tsv (docs/VERIFY.md, 'Live checkcode check') or find the \
         fixture or MATLAB change behind it"
    );
}
