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
//! The same generation also writes `checkcode_messages.tsv` (MATLAB's message text per finding,
//! MathWorks content, kept under `target/`) and `checkcode_files.txt` (every file MATLAB checked).
//! Their paths are given by `MALT_MATLAB_CHECKCODE_MESSAGES` and `MALT_MATLAB_CHECKCODE_FILES`.
//!
//! Tests:
//!
//! - the release that generated the live file is the recorded release (a separate failure, because
//!   drift in reports across releases is expected: re-record, or pin the CI release);
//! - the live (path, line, ID) findings are the recorded ones: a finding MATLAB no longer reports
//!   is a stale expectation, a new one is either a stale expectation or a change in MATLAB;
//! - the files MATLAB checked are the files of `fixtures/matlab_files.txt` (a fixture with no
//!   finding is checked too, so it must be in MATLAB's list);
//! - for every finding both MATLAB and malt report, the message text is the same, after
//!   `normalise_message` on both sides; a text repeated on one line counts once, as a finding does
//!   (the findings test). The accepted differences are in
//!   `tests/matlab/message_allowlist.tsv`, each with a reason; an entry that matches no difference
//!   fails too, so the list only shrinks.
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
use common::expected::{only_in, parse_messages, parse_triples, Triple};
use common::fixtures::{fixtures_dir, lint_fixtures, listed_files};
use common::matlab::{matlab, matlab_string, MatlabRun};

/// Findings to print in full on a failure; the rest are counted.
const SHOWN: usize = 40;

/// What the live MATLAB reported: the findings, the release that reported them, and (when they
/// were available) the message text of each finding and the files it checked. `None` for the
/// last two means that comparison is skipped.
struct Live {
    findings: BTreeSet<Triple>,
    release: String,
    messages: Option<BTreeMap<Triple, Vec<String>>>,
    files: Option<BTreeSet<String>>,
}

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn fixtures() -> PathBuf {
    fixtures_dir()
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
                messages: optional_file("MALT_MATLAB_CHECKCODE_MESSAGES", parse_messages)?,
                files: optional_file("MALT_MATLAB_CHECKCODE_FILES", parse_files)?,
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

/// One of the optional live files, named by an environment variable. Unset or unreadable: a
/// failure with `MALT_MATLAB_REQUIRE=1`, else a `SKIP` line and `None` (the same contract as the
/// TSV).
fn optional_file<T>(var: &str, parse: fn(&str) -> T) -> Result<Option<T>> {
    let required = env::var("MALT_MATLAB_REQUIRE").is_ok_and(|v| v == "1");
    let Some(path) = env::var_os(var) else {
        if required {
            bail!("{var} is not set (MALT_MATLAB_REQUIRE=1)");
        }
        eprintln!("SKIP: {var} is not set: the comparison it feeds is skipped");
        return Ok(None);
    };
    let path = relative_to_repo(Path::new(&path));
    match fs::read_to_string(&path) {
        Ok(text) => Ok(Some(parse(&text))),
        Err(err) if required => bail!(
            "{var}: cannot read {}: {err} (MALT_MATLAB_REQUIRE=1)",
            path.display()
        ),
        Err(err) => {
            eprintln!(
                "SKIP: cannot read {}: {err}: the comparison it feeds is skipped",
                path.display()
            );
            Ok(None)
        }
    }
}

/// The files of `checkcode_files.txt`: one relative path per line.
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
    let beside = |name: &str| -> Result<String> {
        let file = dir.path().join(name);
        fs::read_to_string(&file).with_context(|| {
            format!(
                "reading {} (generate_checkcode.m writes it)",
                file.display()
            )
        })
    };
    Ok(Live {
        findings: parse_triples(&text),
        release: read_release(&out)?,
        messages: Some(parse_messages(&beside("checkcode_messages.tsv")?)),
        files: Some(parse_files(&beside("checkcode_files.txt")?)),
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

/// The message text of each finding malt reports on the fixtures, grouped by finding.
fn malt_messages() -> BTreeMap<Triple, Vec<String>> {
    let mut out: BTreeMap<Triple, Vec<String>> = BTreeMap::new();
    for (triple, message) in lint_fixtures() {
        out.entry(triple).or_default().push(message);
    }
    out
}

/// The accepted message differences: (path, line, ID) to the reason (`message_allowlist.tsv`).
fn message_allowlist() -> BTreeMap<Triple, String> {
    let path = manifest().join("tests/matlab/message_allowlist.tsv");
    let text = fs::read_to_string(&path).unwrap_or_default();
    let mut out = BTreeMap::new();
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let cols: Vec<&str> = line.splitn(4, '\t').collect();
        let reason = cols.get(3).map_or("", |r| r.trim());
        assert!(
            cols.len() == 4 && !reason.is_empty(),
            "{}: each line is path, line, ID and a reason (non-empty): {line}",
            path.display()
        );
        let line_no: usize = cols[1].parse().expect("a line number");
        out.insert(
            (cols[0].to_string(), line_no, cols[2].to_string()),
            reason.to_string(),
        );
    }
    out
}

#[test]
fn live_checkcode_checked_every_listed_file() {
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
         checked (-), {} checked but not listed (+).\n{}\n{}\nSee tests/fixtures/README.md (a new \
         fixture is listed, and checked by the generator even with no finding).",
        missed.len(),
        extra.len(),
        missed
            .iter()
            .take(SHOWN)
            .map(|f| format!("  - {f}"))
            .collect::<Vec<_>>()
            .join("\n"),
        extra
            .iter()
            .take(SHOWN)
            .map(|f| format!("  + {f}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    eprintln!(
        "live MATLAB checked the {} files of matlab_files.txt",
        listed.len()
    );
}

#[test]
fn live_messages_agree_with_malt() {
    let Some(live) = live_or_skip() else {
        return;
    };
    let Some(matlab_messages) = &live.messages else {
        return;
    };
    let malt = malt_messages();
    let allowed = message_allowlist();
    let mut used: BTreeSet<Triple> = BTreeSet::new();
    let mut differences: Vec<(Triple, Vec<String>, Vec<String>)> = Vec::new();
    for (key, malt_texts) in &malt {
        let Some(matlab_texts) = matlab_messages.get(key) else {
            continue;
        };
        let (mut want, mut got) = (matlab_texts.clone(), malt_texts.clone());
        want.sort();
        want.dedup();
        got.sort();
        got.dedup();
        if want == got {
            continue;
        }
        if allowed.contains_key(key) {
            used.insert(key.clone());
        } else {
            differences.push((key.clone(), want, got));
        }
    }
    let stale: Vec<&Triple> = allowed.keys().filter(|k| !used.contains(*k)).collect();
    let compared: usize = malt
        .keys()
        .filter(|k| matlab_messages.contains_key(*k))
        .count();
    eprintln!(
        "{compared} findings compared with live MATLAB {}: {} message difference(s), {} accepted",
        live.release,
        differences.len(),
        used.len()
    );
    if differences.is_empty() && stale.is_empty() {
        return;
    }
    let mut text = String::new();
    let _ = writeln!(
        text,
        "{} message difference(s) between live MATLAB {} and malt (same finding, other text):",
        differences.len(),
        live.release
    );
    let mut per_id: BTreeMap<&str, usize> = BTreeMap::new();
    for ((_, _, id), _, _) in &differences {
        *per_id.entry(id.as_str()).or_default() += 1;
    }
    for (id, count) in &per_id {
        let _ = writeln!(text, "    {id:<16} {count}");
    }
    let _ = writeln!(text, "  first differences:");
    for ((path, line, id), want, got) in differences.iter().take(SHOWN) {
        let _ = writeln!(text, "    {id}  {path}:{line}");
        let _ = writeln!(text, "      matlab: {}", want.join(" | "));
        let _ = writeln!(text, "      malt:   {}", got.join(" | "));
    }
    if differences.len() > SHOWN {
        let _ = writeln!(text, "    ... and {} more", differences.len() - SHOWN);
    }
    for (path, line, id) in &stale {
        let _ = writeln!(
            text,
            "  stale allowlist entry (no such difference now, remove it): {path}:{line}\t{id}"
        );
    }
    print!("{text}");
    panic!(
        "{text}\nfix the message in malt_rules (the check's engine or data file), or accept it in \
         tests/matlab/message_allowlist.tsv with a reason"
    );
}
