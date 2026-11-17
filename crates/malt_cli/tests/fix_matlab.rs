//! malt's Safe autofixes checked against MATLAB's own `fix`, on the same pairs.
//!
//! `fixtures/fix_matlab/<fixture>.<ID>.m` holds the text MATLAB's
//! `codeIssues(...).fix(ID)` produces for the fixture `<fixture>.m`. Every
//! committed pair is one of malt's Safe fixes that must equal MATLAB's output
//! byte for byte: the default `--fix` applies them, so a difference here is
//! what a user would see. Unsafe fixes (AND2, OR2, IJCL, STREMP, FNAN, MCFIL,
//! ...) are deliberately not committed: their rewrites are right in the common
//! case and can change behaviour, and the shapes where malt is knowingly
//! deeper or shallower than MATLAB (NOSEMI's convergence, DSPSP's
//! statement-only form) are documented in `docs/VERIFY.md`, '5.5 Curated safe
//! autofixes'.
//!
//! Two phases, as in `matlab_live.rs`. Generation:
//! `tests/matlab/generate_fixes.m` copies each committed pair's fixture under
//! its own name into a work folder, runs `codeIssues` (factory configuration)
//! and `fix(issues, ID)`, and writes the fixed texts plus `fixes.tsv` — CI
//! does that with `matlab-actions/run-command`. Comparison (this test): the
//! folder named by `MALT_MATLAB_FIXES`, with MATLAB not started, or, without
//! it, MATLAB started here to generate the same folder.
//!
//! Without MATLAB the live test is skipped (see `common::matlab`);
//! `MALT_MATLAB_REQUIRE=1` fails instead.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use common::matlab::{matlab, MatlabRun};

/// One committed pair: the fixture to fix and the check ID to fix it with.
struct Pair {
    /// The fixture file name, without extension (its location is found by name).
    fixture: String,
    /// The check ID MATLAB marks auto-fixable on it.
    id: String,
    /// The committed MATLAB-fixed text.
    expected: String,
}

/// The committed pairs, read from `fixtures/fix_matlab/`. `pairs.json` would be
/// overkill: the file names are the list.
fn pairs() -> Result<Vec<Pair>> {
    let dir = common::fixtures::fixtures_dir().join("fix_matlab");
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().is_none_or(|x| x != "m") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".m") else {
            continue;
        };
        let Some((fixture, id)) = stem.rsplit_once('.') else {
            bail!("{}: the name must be <fixture>.<ID>.m", path.display());
        };
        if fixture.is_empty() || id.is_empty() {
            bail!("{}: the name must be <fixture>.<ID>.m", path.display());
        }
        let expected =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        out.push(Pair {
            fixture: fixture.to_string(),
            id: id.to_string(),
            expected,
        });
    }
    out.sort_by(|a, b| (&a.fixture, &a.id).cmp(&(&b.fixture, &b.id)));
    Ok(out)
}

/// The path of the fixture named `stem` (searched by file name, as the
/// generator searches it).
fn fixture_path(stem: &str) -> Result<PathBuf> {
    let name = format!("{stem}.m");
    let root = common::fixtures::fixtures_dir();
    let mut found: Option<PathBuf> = None;
    visit(&root, &mut |p| {
        if found.is_none() && p.file_name().is_some_and(|n| n == name.as_str()) {
            found = Some(p.to_path_buf());
        }
    })?;
    found.with_context(|| format!("no fixture named {name} under {}", root.display()))
}

/// Walk `dir` and every subfolder, calling `f` for each `.m` file.
fn visit(dir: &Path, f: &mut impl FnMut(&Path)) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            visit(&path, f)?;
        } else if path.extension().is_some_and(|x| x == "m") {
            f(&path);
        }
    }
    Ok(())
}

/// malt's fixed text for one pair: the default configuration, only the fixes
/// of `id` (Safe only — the committed pairs are Safe fixes), repeated until
/// the source stops changing, as `fix_source` does.
fn malt_fixed(source: &str, path: &Path, id: &str) -> String {
    let mut linter = linter();
    let mut current = source.to_string();
    for _ in 0..10 {
        let diagnostics = linter.lint(&current, path).unwrap();
        let mine: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.rule_id == id)
            .cloned()
            .collect();
        let (next, applied) =
            malt_core::apply_fixes_with(&current, &mine, malt_core::FixMode::Safe);
        if applied == 0 || next == current {
            return current;
        }
        current = next;
    }
    current
}

fn linter() -> malt_core::Linter {
    let config = malt_core::Config::default();
    let registry = malt_core::RuleRegistry::new(malt_rules::active_rules(&config), &config);
    let mut linter = malt_core::Linter::new(registry);
    linter.set_inline_suppression(config.inline_suppression);
    linter
}

/// The pairs where malt's Safe fix does not reproduce MATLAB's text.
#[test]
fn malt_safe_fixes_equal_matlab_fix_texts() {
    let problems: Vec<String> = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(|| -> Vec<String> {
            let mut problems = Vec::new();
            for pair in pairs().expect("the committed pairs") {
                let path = fixture_path(&pair.fixture).expect("the fixture");
                let source = fs::read_to_string(&path).expect("the fixture source");
                let fixed = malt_fixed(&source, &path, &pair.id);
                if fixed != pair.expected {
                    problems.push(format!(
                        "{}: malt's {} fix differs from MATLAB's text:\n--- malt ---\n{}\
                         --- matlab ---\n{}",
                        path.display(),
                        pair.id,
                        show_diff(&source, &fixed),
                        show_diff(&source, &pair.expected),
                    ));
                }
            }
            problems
        })
        .unwrap()
        .join()
        .unwrap();
    assert!(
        problems.is_empty(),
        "{} committed pair(s) where malt's Safe fix is not MATLAB's:\n{}",
        problems.len(),
        problems.join("\n")
    );
}

/// A short hint of what changed, for the failure message.
fn show_diff(source: &str, fixed: &str) -> String {
    let mut out = String::new();
    for (a, b) in source.lines().zip(fixed.lines()) {
        if a != b {
            out.push_str(&format!("  -{a}\n  +{b}\n"));
        }
    }
    out
}

/// What the live MATLAB fixed. `None` skips the live comparison.
struct Live {
    /// The folder the generator wrote (`<pair>.<ID>.m` plus `fixes.tsv`).
    dir: PathBuf,
}

impl Live {
    /// Read the live output from `MALT_MATLAB_FIXES`, with MATLAB not started.
    fn from_env() -> Result<Option<Live>> {
        let Some(dir) = std::env::var_os("MALT_MATLAB_FIXES") else {
            return Ok(None);
        };
        let mut dir = PathBuf::from(dir);
        if dir.is_relative() {
            // Relative to the repository root, as the other MALT_MATLAB_* paths are: cargo
            // runs a test in its crate folder.
            dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(dir);
        }
        if !dir.is_dir() {
            bail!("MALT_MATLAB_FIXES: {} is not a folder", dir.display());
        }
        Ok(Some(Live { dir }))
    }

    /// Run the generator in a temporary folder with the MATLAB found here.
    fn from_matlab(run: &MatlabRun) -> Result<Live> {
        let dir = tempfile::TempDir::new().context("the MATLAB fixes folder")?;
        let script = common::fixtures::fixtures_dir()
            .join("../matlab/generate_fixes.m")
            .canonicalize()
            .context("finding tests/matlab/generate_fixes.m")?;
        let code = format!(
            "run('{}');",
            script.display().to_string().replace('\'', "''")
        );
        run.run_batch_with(
            &code,
            run.timeout(),
            &[("MALT_FIXES_OUT", dir.path().as_os_str())],
        )?;
        Ok(Live { dir: dir.keep() })
    }
}

/// The live MATLAB's own fixes agree with the committed texts, and it fixed
/// every pair.
#[test]
fn live_matlab_fixes_agree_with_committed() {
    let Some(run) = matlab() else {
        return;
    };
    let live = match Live::from_env() {
        // The generator's folder (what CI does), or MATLAB started here.
        Err(err) => panic!("MALT_MATLAB_FIXES: {err:#}"),
        Ok(Some(live)) => live,
        Ok(None) => Live::from_matlab(&run).expect("running generate_fixes.m"),
    };
    let problems: Vec<String> = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || -> Vec<String> {
            let mut problems = Vec::new();
            for row in fs::read_to_string(live.dir.join("fixes.tsv"))
                .expect("fixes.tsv")
                .lines()
                .skip(1)
            {
                let mut parts = row.split('\t');
                let name = parts.next().unwrap_or("");
                let changed = parts.next().unwrap_or("0");
                let success = parts.next().unwrap_or("0");
                if success != "1" {
                    problems.push(format!("{name}: MATLAB's fix did not succeed"));
                    continue;
                }
                if changed != "1" {
                    problems.push(format!("{name}: MATLAB's fix changed nothing"));
                    continue;
                }
                let generated =
                    fs::read_to_string(live.dir.join(format!("{name}.m"))).unwrap_or_default();
                let committed = fs::read_to_string(
                    common::fixtures::fixtures_dir()
                        .join("fix_matlab")
                        .join(format!("{name}.m")),
                )
                .unwrap_or_default();
                if generated != committed {
                    problems.push(format!(
                        "{name}: MATLAB's live fix text differs from the committed one"
                    ));
                }
            }
            problems
        })
        .unwrap()
        .join()
        .unwrap();
    assert!(
        problems.is_empty(),
        "{} live difference(s) against the committed MATLAB fixes:\n{}",
        problems.len(),
        problems.join("\n")
    );
}
