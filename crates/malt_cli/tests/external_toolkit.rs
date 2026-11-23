//! Conformance on real third-party code, measured without vendoring it.
//!
//! The MathWorks `matlab-agentic-toolkit` is licensed for use with MathWorks products
//! only, so it is not copied into this repository. Set `MALT_EXTERNAL_TOOLKIT` to a
//! clone at commit `b58be8124631cccb03a67aa79dcef1c9720243e2` to run this test;
//! without it the test is skipped. `toolkit_expected.tsv` holds only facts (relative
//! path, line, check ID) of what MATLAB's `checkcode` reports there.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use malt_core::{Config, Linter, RuleRegistry};

type Triple = (String, usize, String);

fn mfiles(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            mfiles(&path, out);
        } else if path.extension().is_some_and(|e| e == "m") {
            out.push(path);
        }
    }
}

fn run(root: PathBuf) -> BTreeSet<(String, Triple)> {
    let expected: BTreeSet<Triple> = include_str!("toolkit_expected.tsv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            (c[0].to_string(), c[1].parse().unwrap(), c[2].to_string())
        })
        .collect();
    let mut files = Vec::new();
    mfiles(&root, &mut files);
    files.sort();
    let config = Config::default();
    let registry = RuleRegistry::new(malt_rules::active_rules(&config), &config);
    let mut linter = Linter::new(registry);
    let mut found = BTreeSet::new();
    for path in files {
        let source = fs::read_to_string(&path).unwrap();
        let rel = path
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        for d in linter.lint(&source, &path).unwrap() {
            found.insert((rel.clone(), d.line, d.rule_id.to_string()));
        }
    }
    let mut diff = BTreeSet::new();
    for t in expected.difference(&found) {
        diff.insert(("matlab-only".to_string(), t.clone()));
    }
    for t in found.difference(&expected) {
        diff.insert(("malt-only".to_string(), t.clone()));
    }
    diff
}

#[test]
fn no_differences_with_matlab_on_toolkit() {
    let Ok(dir) = std::env::var("MALT_EXTERNAL_TOOLKIT") else {
        eprintln!("MALT_EXTERNAL_TOOLKIT not set; skipping");
        return;
    };
    let root = PathBuf::from(dir);
    let diff = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || run(root))
        .unwrap()
        .join()
        .unwrap();
    assert!(
        diff.is_empty(),
        "{} difference(s):\n{}",
        diff.len(),
        diff.iter()
            .take(40)
            .map(|(k, (p, l, i))| format!("  {k}\t{p}:{l}\t{i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
