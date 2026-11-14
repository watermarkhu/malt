//! Regression tests for the per-file caches. The symbol table, the dead-store
//! results, the reaching-binding index and the word pre-filters are built once
//! per parse and shared by every check of that file. A memo that outlived its
//! file would change the diagnostics of the next one, so these tests lint
//! several sources through one `Linter` and compare every result with a fresh
//! linter's. They also keep files with many top-level statements and functions
//! from going back to quadratic time, and check that running the file checks
//! of a large file on several threads changes nothing.

use std::path::Path;
use std::time::Instant;

use malt_core::{Config, Linter, RuleRegistry};

/// The diagnostics of one lint as comparable values, in report order.
type Report = Vec<(String, usize, usize, String)>;

fn linter() -> Linter {
    let config = Config::from_toml("[lint]\npreset = \"all\"\n").unwrap();
    Linter::new(RuleRegistry::new(
        malt_rules::active_rules(&config),
        &config,
    ))
}

fn report(linter: &mut Linter, source: &str) -> Report {
    linter
        .lint(source, Path::new("shared.m"))
        .expect("lint should succeed")
        .into_iter()
        .map(|d| (d.rule_id.to_string(), d.line, d.column, d.message))
        .collect()
}

/// A script whose checks fire on unused, undefined and grown variables.
const SCRIPT: &str = "unusedA = 1;\nq = 2;\ndisp(w);\nfor k = 1:3\n  s(k) = k;\nend\n";
/// A function file with an unused local and an unused input.
const FUNCTION: &str = "function r = f(a, b)\n  c = a;\n  r = a + 1;\nend\n";
/// A script that preallocates, grows and reads one array.
const OTHER: &str = "x = zeros(1, 3);\nfor i = 1:3\n  x(i) = i;\nend\ny = x;\n";
/// Checks that read the binding reaching a use: constant conditions, a value
/// that shadows a function, a fixed-point variable assigned directly.
const REACHING: &str = "k = 0;\nif k || true\n  disp(1);\nend\nmax = 3;\n[~, idx] = max(1);\ny = fi(0, 1, 16);\ny = 2;\n";
/// The same names bound differently, so a stale index would answer wrongly.
const REACHING_OTHER: &str = "k = input('');\nif k || true\n  disp(1);\nend\nmax = @(v) v;\n[~, idx] = max(1);\ny = 2;\ny = fi(0, 1, 16);\n";

#[test]
fn interleaved_lints_match_fresh_linters() {
    let sources = [SCRIPT, FUNCTION, OTHER, REACHING, REACHING_OTHER];
    let expected: Vec<Report> = sources
        .iter()
        .map(|source| report(&mut linter(), source))
        .collect();
    assert!(
        expected.iter().all(|report| !report.is_empty()),
        "each sample should trigger checks"
    );

    let mut shared = linter();
    for round in 0..2 {
        for (source, want) in sources.iter().zip(&expected) {
            assert_eq!(&report(&mut shared, source), want, "round {round}");
        }
    }
}

#[test]
fn same_length_source_in_a_reused_buffer_is_not_served_a_stale_table() {
    // Both sources are 20 bytes, and clearing the buffer keeps its allocation, so
    // the second lint sees the same address and length as the first.
    let first = "unusedA = 1;\nq = 2;\n";
    let second = "q = 3;\nunusedA = 4;\n";
    assert_eq!(first.len(), second.len());

    let mut buffer = String::from(first);
    let mut shared = linter();
    let _ = report(&mut shared, &buffer);
    buffer.clear();
    buffer.push_str(second);

    assert_eq!(
        report(&mut shared, &buffer),
        report(&mut linter(), second),
        "the second source must be linted on its own"
    );
}

#[test]
fn many_top_level_statements_and_functions_lint_in_time() {
    let mut source = String::new();
    for i in 0..3000 {
        source.push_str(&format!("v{i} = {i} + 1;\n"));
    }
    for i in 0..300 {
        source.push_str(&format!("function r = f{i}(a)\n  r = a + {i};\nend\n"));
    }

    // The default configuration, as users run it: the `all` preset enables far
    // more checks and is timed by the release benchmarks instead.
    let config = Config::default();
    let mut linter = Linter::new(RuleRegistry::new(
        malt_rules::active_rules(&config),
        &config,
    ));
    let start = Instant::now();
    let diagnostics = report(&mut linter, &source);
    let elapsed = start.elapsed();

    assert!(
        !diagnostics.is_empty(),
        "the unused variables should be reported"
    );
    assert!(
        elapsed.as_secs_f64() < 10.0,
        "3,000 statements and 300 functions took {elapsed:?}"
    );
}

/// A file large enough for the file checks to run on several threads.
fn large_source() -> String {
    let mut source = String::new();
    for i in 0..600 {
        source.push_str(&format!(
            "v{i} = {i};\nif v{i} || true\n  s{i}(2) = v{i};\nend\nw{i} = q + v{i}\n"
        ));
    }
    for i in 0..60 {
        source.push_str(&format!(
            "function r = f{i}(a, b)\n  c = a;\n  for k = 1:3\n    g(k) = k;\n  end\n  r = a + {i};\nend\n"
        ));
    }
    source
}

#[test]
fn file_checks_on_several_threads_match_one_thread() {
    let source = large_source();
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_matlab::LANGUAGE.into())
        .unwrap();
    let nodes = parser
        .parse(&source, None)
        .unwrap()
        .root_node()
        .descendant_count();
    // The linter only spreads the file checks of trees this large.
    assert!(nodes >= 20_000, "{nodes} nodes");
    let expected = report(&mut linter(), &source);
    assert!(expected.len() > 1_000, "got {} diagnostics", expected.len());

    let mut parallel = linter();
    parallel.set_file_check_threads(4, 64 * 1024 * 1024);
    for round in 0..2 {
        assert_eq!(report(&mut parallel, &source), expected, "round {round}");
        // A small file in between, through the same linter.
        assert_eq!(
            report(&mut parallel, SCRIPT),
            report(&mut linter(), SCRIPT),
            "round {round}"
        );
    }
}
