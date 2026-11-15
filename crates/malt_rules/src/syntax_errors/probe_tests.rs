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

#[test]
fn dotted_name_followed_by_a_word_is_syner_at_the_word_and_stays_on_its_line() {
    // MATLAB (probes c2, c3, c4): SYNER at the word (`arg1` column 18, `arg2` column 22,
    // `baz` column 9) and the statements on the next lines are analysed as usual: the
    // handle on line 3 is a statement of its own, not part of the broken one.
    let source = "function f(x)\ncomm.AWGNChannel arg1\nh = @comm.gpu.AWGNChannel;\ncomm.gpu.AWGNChannel arg2\nfoo.bar baz\ny = x\nend\n";
    let diags = lint_file(&*engine(), source);
    assert_eq!(positions(&diags, "SYNER"), vec![(2, 18), (4, 22), (5, 9)]);
}

#[test]
fn dotted_names_that_are_expressions_are_not_syner() {
    // `x.y` alone, with an operator, with a call and a one-line `if` with a comma are valid.
    let source = "x.y\nx.y.z;\nx.y - 1;\nx.y (1);\nif c, a.b, end\n";
    assert!(!has_id(&lint_file(&*engine(), source), "SYNER"));
}

#[test]
fn end_as_a_value_inside_brackets_is_reswd_at_the_end() {
    // MATLAB: RESWD at the `end` (column 6) for `[end + 1]`, `{end}` and `[end 1]`.
    let source = "y = [end + 1];\nz = {end};\nw = [end 1];\n";
    let diags = lint_file(&*engine(), source);
    assert_eq!(positions(&diags, "RESWD"), vec![(1, 6), (2, 6), (3, 6)]);
}
