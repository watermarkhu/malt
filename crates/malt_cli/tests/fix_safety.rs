//! Safety net for autofixes.
//!
//! Every file in `tests/fixtures/fix/` triggers at least one fixable check. For
//! each of them, with every rule enabled and only safe fixes applied, a fix must
//!
//! 1. never turn valid MATLAB into code that does not parse,
//! 2. converge: fixing the fixed source changes nothing,
//! 3. never stack semicolons (`;;`) where the source had none, which is what two
//!    checks inserting the same `;` used to produce.
//!
//! Failures are collected over all fixtures and reported together, so one run
//! shows every broken fix instead of the first.

use std::fs;
use std::path::{Path, PathBuf};

use malt_core::{fix_source, Config, FixMode, Linter, RuleRegistry};

fn linter() -> Linter {
    let config = Config::from_toml("[lint]\npreset = \"all\"\n").unwrap();
    let registry = RuleRegistry::new(malt_rules::active_rules(&config), &config);
    let mut linter = Linter::new(registry);
    linter.set_inline_suppression(config.inline_suppression);
    linter
}

fn parses_cleanly(source: &str) -> bool {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_matlab::LANGUAGE.into())
        .unwrap();
    parser
        .parse(source, None)
        .is_some_and(|tree| !tree.root_node().has_error())
}

fn fixtures() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fix");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "m"))
        .collect();
    files.sort();
    assert!(files.len() > 100, "fixture corpus went missing");
    files
}

fn check(path: &Path, linter: &mut Linter) -> Vec<String> {
    let source = fs::read_to_string(path).unwrap();
    let name = path.file_name().unwrap().to_string_lossy();
    let mut problems = Vec::new();

    let first = fix_source(linter, &source, path, FixMode::Safe).unwrap();
    if first.source == source {
        return problems; // nothing safe to fix here
    }

    if parses_cleanly(&source) && !parses_cleanly(&first.source) {
        problems.push(format!("{name}: the fix produced code that does not parse"));
    }

    let again = fix_source(linter, &first.source, path, FixMode::Safe).unwrap();
    if again.source != first.source {
        problems.push(format!("{name}: fixing is not idempotent"));
    }

    if first.source.matches(";;").count() > source.matches(";;").count() {
        problems.push(format!("{name}: the fix stacked semicolons (;;)"));
    }
    problems
}

#[test]
fn safe_fixes_keep_code_valid_and_converge() {
    // Test threads get a 2 MiB stack, less than the 8 MiB of a main thread: run
    // on a thread sized like the one the CLI lints on.
    let problems: Vec<String> = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(|| {
            let mut linter = linter();
            fixtures()
                .iter()
                .flat_map(|path| check(path, &mut linter))
                .collect()
        })
        .unwrap()
        .join()
        .unwrap();
    assert!(
        problems.is_empty(),
        "{} unsafe fix result(s):\n{}",
        problems.len(),
        problems.join("\n")
    );
}
