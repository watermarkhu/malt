//! The conformance fixtures, read and linted by malt the way the MATLAB tests read them: every
//! `.m` file under `tests/fixtures` at any depth, with the default configuration (`mathworks`).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use malt_core::{Config, Linter, RuleRegistry};

use super::expected::{normalise_message, Triple};

/// Linting recurses into the parse tree: run it on a thread with a large stack.
const STACK: usize = 256 * 1024 * 1024;

/// The fixtures folder (`tests/fixtures`).
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Every `.m` file under `dir`, at any depth, in sorted order.
pub fn fixture_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_m_files(&fixtures_dir(), &mut files);
    files.sort();
    files
}

fn collect_m_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_m_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "m") {
            out.push(path);
        }
    }
}

/// The fixture files `tests/fixtures/matlab_files.txt` lists: relative paths, one per line.
pub fn listed_files() -> BTreeSet<String> {
    let path = fixtures_dir().join("matlab_files.txt");
    let text = fs::read_to_string(&path).unwrap_or_default();
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// A fixture path relative to the fixtures folder, with forward slashes.
pub fn relative(path: &Path) -> String {
    path.strip_prefix(fixtures_dir())
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

/// Everything malt reports on the fixtures: (path relative to the fixtures folder, line, ID)
/// with the message text, normalised as `normalise_message` does. A message appears once per
/// finding, so one triple can have several.
pub fn lint_fixtures() -> Vec<(Triple, String)> {
    std::thread::Builder::new()
        .stack_size(STACK)
        .spawn(|| {
            let config = Config::default();
            let registry = RuleRegistry::new(malt_rules::active_rules(&config), &config);
            let mut linter = Linter::new(registry);
            let mut found = Vec::new();
            for path in fixture_files() {
                let source = fs::read_to_string(&path).unwrap();
                let rel = relative(&path);
                for d in linter.lint(&source, &path).unwrap() {
                    found.push((
                        (rel.clone(), d.line, d.rule_id.to_string()),
                        normalise_message(&d.message),
                    ));
                }
            }
            found
        })
        .unwrap()
        .join()
        .unwrap()
}
