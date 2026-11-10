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
fn script_with_clear_and_clc_is_clean() {
    // MATLAB reports CLEAR0ARGS inside functions only.
    assert_eq!(ids("script1.m"), Vec::<&str>::new());
}

#[test]
fn function_file_is_clean() {
    assert_eq!(ids("func1.m"), Vec::<&str>::new());
}

#[test]
fn class_file_only_has_the_arguments_block_finding() {
    // The FC* release gates are off by default, as in MATLAB's factory
    // configuration (checked against checkcode in R2026b).
    // FVINR needs an `arguments (Output)` block next to the unattributed one;
    // a lone block is silent in MATLAB (probe fv1).
    assert_eq!(ids("cls1.m"), Vec::<&str>::new());
}

#[test]
fn function_with_arguments_block_parfor_and_switch() {
    // PFNF is right: `helper` is a nested function called inside parfor.
    // FVINR is silent: the arguments block is the only one (probe fv1).
    assert_eq!(ids("func2.m"), vec!["PFNF"]);
}

#[test]
fn script_workspace_variables_are_not_dead_stores() {
    // The values a script leaves in the workspace are its results: MATLAB
    // reports no NASGU for them.
    assert_eq!(ids("script2.m"), Vec::<&str>::new());
}

#[test]
fn unit_test_class_is_clean() {
    assert_eq!(ids("test1.m"), Vec::<&str>::new());
}

#[test]
fn guide_file_has_only_unused_callback_arguments_and_callback() {
    // No DEFNU: gui1.m calls str2func, and MATLAB reports no DEFNU in a file
    // that calls eval, feval or str2func (probed; the callbacks are not reported).
    // VANUS is off under the mathworks preset: MATLAB never reports it.
    assert_eq!(ids("gui1.m"), vec!["INUSD"]);
}

#[test]
fn legacy_globals_nargin_varargin_and_eval() {
    // No VALST (varargin is last), no FVAL (no feval), no STRNU on the global,
    // no TLEV on a top-level eval.
    // `out = eval(expr)` and the two-argument eval are not EVLCS in MATLAB.
    // ERTAG is off under the mathworks preset: MATLAB never reports it.
    // GVMIS: MATLAB reports the `global` declaration itself (legacy1.m:2).
    assert_eq!(ids("legacy1.m"), vec!["AGROW", "GVMIS"]);
}

#[test]
fn legacy_string_handling() {
    // No OR2: MATLAB is silent on `strcmp(...) | strcmp(...)` (a call operand).
    assert_eq!(ids("legacy2.m"), vec!["AGROW", "NASGU"]);
}

#[test]
fn numeric_solver_has_no_hermitian_false_positive() {
    // `r' * r` is already Hermitian; only `x' * A * x` chains are MHERM's.
    assert_eq!(ids("num1.m"), Vec::<&str>::new());
}

#[test]
fn numeric_script_findings_are_real() {
    // rand(N, 2) is not the 'seed' form (RAND); the script's workspace values
    // are not dead stores.
    // No STRNU on the table field (MATLAB is silent) and no LOGSUM (off by default:
    // MATLAB reports it for `sum(x(:) <= 1)` but not for this comparison).
    assert_eq!(ids("num2.m"), Vec::<&str>::new());
}

#[test]
fn coder_inline_is_not_the_removed_inline_function() {
    assert_eq!(ids("coder1.m"), Vec::<&str>::new());
}
