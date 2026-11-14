//! Tests from the MATLAB R2026b probes of the syntax items (`checkcode` output): an
//! op-assign on `i` or `j`, underscore names, quoted name-value names, and the
//! column of M3COL.

use super::*;
use crate::test_util::{has_id, lint_file};
use malt_core::Config;

fn engine() -> Box<dyn Rule> {
    SyntaxErrorsEngine::from_config(&Config::default())
}

fn positions(diags: &[Diagnostic], id: &str) -> Vec<(usize, usize)> {
    diags
        .iter()
        .filter(|d| d.rule_id == id)
        .map(|d| (d.line, d.column))
        .collect()
}

#[test]
fn op_assign_on_imaginary_unit_is_syner_in_scripts_and_functions() {
    // MATLAB: SYNER at the `=` (column 4), in a script and in a function.
    let script = lint_file(&*engine(), "i = 1;\ni += 1;\n");
    assert_eq!(positions(&script, "SYNER"), vec![(2, 4)]);
    let function = lint_file(&*engine(), "function f()\ni += 1;\nend\n");
    assert_eq!(positions(&function, "SYNER"), vec![(2, 4)]);
}

#[test]
fn op_assign_on_other_names_is_not_syner() {
    // MATLAB: silent for an unset `x`, and REDEF (not SYNER) once `x` is assigned.
    assert!(!has_id(&lint_file(&*engine(), "x += 1;\n"), "SYNER"));
    assert!(!has_id(
        &lint_file(&*engine(), "x = 1;\nx += 1;\n"),
        "SYNER"
    ));
}

#[test]
fn leading_underscore_names_are_badch() {
    // MATLAB: BADCH at the `_` of `_a` (column 9). An ordinary name with an
    // inner underscore is fine.
    let diags = lint_file(&*engine(), "x = 1;\ny = x + _a;\n");
    assert_eq!(positions(&diags, "BADCH"), vec![(2, 9)]);
    assert!(!has_id(&lint_file(&*engine(), "x_1 = 1;\n"), "BADCH"));
}

#[test]
fn quoted_name_value_names_are_fvacs_or_fvami() {
    // MATLAB: a quoted valid identifier is FVACS; a quoted invalid one is FVAMI.
    let fvacs = lint_file(&*engine(), "t = s('a' = 1, 2);\n");
    assert_eq!(positions(&fvacs, "FVACS"), vec![(1, 7)]);
    assert!(!has_id(&fvacs, "FVAMI"));
    let fvami = lint_file(&*engine(), "f('Bad Name' = 1);\n");
    assert_eq!(positions(&fvami, "FVAMI"), vec![(1, 3)]);
}
