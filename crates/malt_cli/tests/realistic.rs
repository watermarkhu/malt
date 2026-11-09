//! Ordinary MATLAB code should not trigger false positives with the default
//! preset. Each file below is idiomatic code; the listed IDs are the only
//! findings it is expected to produce.

use std::path::Path;

use malt_core::{Config, Linter, RuleRegistry};

fn ids(file: &str) -> Vec<&'static str> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/realistic")
        .join(file);
    let source = std::fs::read_to_string(&path).unwrap();
    let config = Config::default();
    let registry = RuleRegistry::new(malt_rules::active_rules(&config), &config);
    let mut linter = Linter::new(registry);
    // Fixture names are not the class name in `cls1.m`'s directory layout, so
    // lint it under its own file name.
    let mut ids: Vec<&'static str> = linter
        .lint(&source, &path)
        .unwrap()
        .into_iter()
        .map(|d| d.rule_id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

#[test]
fn script_has_only_the_clear_finding() {
    assert_eq!(ids("script1.m"), vec!["CLEAR0ARGS"]);
}

#[test]
fn function_file_is_clean() {
    assert_eq!(ids("func1.m"), Vec::<&str>::new());
}

#[test]
fn class_file_only_has_release_gate_findings() {
    assert_eq!(ids("cls1.m"), vec!["FCDQS", "FCFAV", "FVINR"]);
}

#[test]
fn function_with_arguments_block_parfor_and_switch() {
    // PFNF is right: `helper` is a nested function called inside parfor.
    assert_eq!(ids("func2.m"), vec!["FCDQS", "FCFAV", "FVINR", "PFNF"]);
}

#[test]
fn script_findings_are_real_dead_stores_and_a_missing_semicolon() {
    assert_eq!(ids("script2.m"), vec!["NASGU", "PRTCAL"]);
}

#[test]
fn unit_test_class_is_clean() {
    assert_eq!(ids("test1.m"), Vec::<&str>::new());
}
