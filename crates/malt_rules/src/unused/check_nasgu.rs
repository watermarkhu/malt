//! NASGU check: value assigned to a variable might be unused.
//!
//! An assignment is reported when no path from it reads the variable before
//! another full assignment or `clear` replaces it. Scripts keep their final
//! values in the workspace; a function reports a value that reaches its end
//! unread, except for its output arguments. The analysis is in
//! [`crate::analysis::liveness`], which reports nothing for a scope whose
//! control flow it cannot see through.
//!
//! One element of a `[a, b] = f()` output list that is dead is ASGLU, not NASGU
//! (probed: MATLAB replaces the variable with `~`). An indexed partial store
//! (`x(i) = v`, `c{i} = v`) that is dead is NASGU. A field store (`s.f = v`) is
//! STRNU, reported by the good-practices engine.

use super::*;
use crate::analysis::liveness::{dead_stores, is_field_store};
use malt_core::Fix;

impl UnusedEngine {
    /// Run NASGU and ASGLU checks: a value assigned to a variable might be unused.
    pub(crate) fn check_nasgu(
        &self,
        table: &SymbolTable,
        tree: &tree_sitter::Tree,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let nasgu_off = self.is_check_disabled("NASGU");
        let asglu_off = self.is_check_disabled("ASGLU");
        if nasgu_off && asglu_off {
            return;
        }

        for index in 0..table.scopes.len() {
            let Some(dead) = dead_stores(table, index, tree, source) else {
                continue;
            };
            let scope = &table.scopes[index];
            for def_index in dead.full {
                let def = &scope.defs[def_index];
                if self.should_ignore_name(&def.name) {
                    continue;
                }
                if def.multi_output {
                    if !asglu_off {
                        diagnostics.push(Diagnostic {
                            rule_id: "ASGLU",
                            message: "Value assigned to variable might be unused. Consider replacing the variable with ~ instead.".to_string(),
                            severity: Severity::Warning,
                            byte_range: def.byte_range.clone(),
                            line: def.line,
                            column: def.column,
                            fix: Some(Fix::new(def.byte_range.clone(), "~")),
                        });
                    }
                } else if !nasgu_off {
                    diagnostics.push(nasgu_diagnostic(def));
                }
            }
            for def_index in dead.partial {
                let def = &scope.defs[def_index];
                // Field stores are STRNU; a dead output of a multi-output store
                // has no official probe, so it is left alone.
                if nasgu_off
                    || def.multi_output
                    || self.should_ignore_name(&def.name)
                    || is_field_store(tree, def.byte_range.clone())
                {
                    continue;
                }
                diagnostics.push(nasgu_diagnostic(def));
            }
        }
    }
}

/// The NASGU diagnostic for a dead assignment `def`.
fn nasgu_diagnostic(def: &crate::analysis::symbols::VarDef) -> Diagnostic {
    Diagnostic {
        rule_id: "NASGU",
        message: "Value assigned to variable might be unused.".to_string(),
        severity: Severity::Warning,
        byte_range: def.byte_range.clone(),
        line: def.line,
        column: def.column,
        fix: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    /// The lines NASGU reports for `source`, sorted.
    fn nasgu_lines(source: &str) -> Vec<usize> {
        let mut lines: Vec<usize> = lint_file(&*engine(), source)
            .iter()
            .filter(|d| d.rule_id == "NASGU")
            .map(|d| d.line)
            .collect();
        lines.sort_unstable();
        lines
    }

    // -- Probes: MATLAB R2026b checkcode, scripts ----------------------------

    #[test]
    fn nasgu_script_overwritten_before_use() {
        assert_eq!(nasgu_lines("a = 1;\na = 2;\ndisp(a)\n"), vec![1]);
    }

    #[test]
    fn nasgu_script_cleared_before_use() {
        assert_eq!(nasgu_lines("e = 7;\nclear e\n"), vec![1]);
    }

    #[test]
    fn nasgu_script_final_value_stays_in_workspace() {
        assert!(nasgu_lines("x = 3;\n").is_empty());
    }

    #[test]
    fn nasgu_script_loop_store_overwritten_in_same_pass() {
        let source = "for i = 1:3\n    t = i;\n    t = i * 2;\n    disp(t)\nend\n";
        assert_eq!(nasgu_lines(source), vec![2]);
    }

    #[test]
    fn nasgu_script_chain_of_overwrites() {
        assert_eq!(nasgu_lines("x = 1;\nx = 2;\nx = 3;\ndisp(x)\n"), vec![1, 2]);
    }

    #[test]
    fn nasgu_script_clear_then_store() {
        assert_eq!(nasgu_lines("x = 1;\nclear x\nx = 2;\ndisp(x)\n"), vec![1]);
    }

    #[test]
    fn nasgu_script_break_keeps_value_in_workspace() {
        let source = "for i = 1:3\n    t = i;\n    if i == 2, break; end\n    t = 0;\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_script_while_may_skip_store() {
        let source = "k = 2;\na = 1;\nwhile k > 0\n    a = 2;\n    k = k - 1;\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_script_conditional_store_does_not_kill() {
        let source = "k = rand > 0.5;\na = 1;\nif k, a = 2; end\ndisp(a)\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_script_both_branches_overwrite() {
        let source = "k = rand > 0.5;\na = 1;\nif k, a = 2; else, a = 3; end\ndisp(a)\n";
        assert_eq!(nasgu_lines(source), vec![2]);
    }

    #[test]
    fn nasgu_script_while_body_store_not_dead() {
        assert!(nasgu_lines("a = 1;\nwhile rand > 0.5, a = 2; end\ndisp(a)\n").is_empty());
    }

    #[test]
    fn nasgu_script_loop_final_store_is_workspace_value() {
        assert!(nasgu_lines("for i = 1:3\n    a = i;\nend\n").is_empty());
    }

    #[test]
    fn nasgu_script_clear_in_branch_kills() {
        let source = "a = 1;\nif rand > 0.5, clear a; end\na = 2;\ndisp(a)\n";
        assert_eq!(nasgu_lines(source), vec![1]);
    }

    #[test]
    fn nasgu_script_read_on_one_path_keeps_value() {
        let source = "a = 1;\nif rand > 0.5, disp(a); end\na = 2;\ndisp(a)\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_script_try_body_store_read_by_nothing() {
        // The catch body does not read `a`, and the store after the try overwrites it.
        let source = "try\n    a = 1;\ncatch\nend\na = 2;\ndisp(a)\n";
        assert_eq!(nasgu_lines(source), vec![2]);
    }

    #[test]
    fn nasgu_one_line_if_body_reads_value() {
        // A single-line `if` body is a statement of its own, not a block.
        assert!(nasgu_lines("a = 1;\nk = true;\nif k, disp(a), end\n").is_empty());
    }

    #[test]
    fn nasgu_one_line_if_store_read_later() {
        let source = "function y = f(tol)\n    if nargin < 3, tol = 1e-8; end\n    y = tol;\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_if_condition_is_a_read() {
        let source = "function f()\n    a = 1;\n    if a > 0\n        disp(2);\n    end\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_while_condition_is_a_read() {
        let source =
            "function f()\n    n = 3;\n    while n > 0\n        n = n - 1;\n    end\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_script_eval_does_not_stop_analysis() {
        // Probed: MATLAB reports line 1; eval does not stop the analysis.
        assert_eq!(nasgu_lines("a = 1;\neval('disp(2)');\na = 2;\n"), vec![1]);
    }

    // -- Probes: functions ---------------------------------------------------

    /// The ASGLU diagnostics for `source`, as (line, column), sorted.
    fn asglu_positions(source: &str) -> Vec<(usize, usize)> {
        let mut found: Vec<(usize, usize)> = lint_file(&*engine(), source)
            .iter()
            .filter(|d| d.rule_id == "ASGLU")
            .map(|d| (d.line, d.column))
            .collect();
        found.sort_unstable();
        found
    }

    // -- ASGLU: an output of a multi-output call that is dead ----------------

    #[test]
    fn asglu_fires_on_unused_output_element() {
        let src = "function foo(x)\n    [a, b] = size(x);\n    disp(a);\nend\n";
        assert_eq!(asglu_positions(src), vec![(2, 9)]);
        let diags = lint_file(&*engine(), src);
        let d: Vec<_> = diags.iter().filter(|d| d.rule_id == "ASGLU").collect();
        assert_eq!(&src[d[0].byte_range.clone()], "b");
        assert_eq!(d[0].fix.as_ref().unwrap().replacement, "~");
        assert!(!has_id(&diags, "NASGU"), "disjoint from NASGU: {diags:?}");
    }

    #[test]
    fn asglu_message_is_the_official_one() {
        let diags = lint_file(
            &*engine(),
            "function foo(x)\n    [a, b] = size(x);\n    disp(a);\nend\n",
        );
        let d = diags.iter().find(|d| d.rule_id == "ASGLU").unwrap();
        assert_eq!(
            d.message,
            "Value assigned to variable might be unused. Consider replacing the variable with ~ instead."
        );
    }

    #[test]
    fn asglu_both_outputs_unused_at_function_end() {
        assert_eq!(
            asglu_positions("function f(x)\n    [a, b] = size(x);\nend\n"),
            vec![(2, 6), (2, 9)]
        );
    }

    #[test]
    fn asglu_fires_on_first_element_overwritten_before_read() {
        // Probed: MATLAB reports the first `a` and the first `b`, not the last stores.
        assert_eq!(
            asglu_positions(
                "function f(x)\n    [a, b] = size(x);\n    a = 1;\n    disp(a + b);\nend\n"
            ),
            vec![(2, 6)]
        );
        assert_eq!(
            asglu_positions(
                "function f(x)\n    [a, b] = size(x);\n    disp(a);\n    b = 2;\nend\n"
            ),
            vec![(2, 9)]
        );
    }

    #[test]
    fn asglu_silent_in_a_script_when_value_stays_in_workspace() {
        // Probed: MATLAB keeps the final values of a script in the workspace.
        assert!(asglu_positions("[a, b] = size(1);\n").is_empty());
        assert!(asglu_positions("[a, b] = size(1);\ndisp(a)\n").is_empty());
    }

    #[test]
    fn asglu_script_overwritten_before_read() {
        assert_eq!(
            asglu_positions("[a, b] = size(1);\nb = 3;\ndisp(a + b)\n"),
            vec![(1, 5)]
        );
    }

    #[test]
    fn asglu_silent_for_tilde_and_used_outputs() {
        assert!(
            asglu_positions("function f(x)\n    [~, b] = size(x);\n    disp(b);\nend\n").is_empty()
        );
        assert!(
            asglu_positions("function f(x)\n    [a, b] = size(x);\n    disp(a + b);\nend\n")
                .is_empty()
        );
    }

    #[test]
    fn asglu_dynamic_feval_call_is_a_multi_output_call() {
        assert_eq!(
            asglu_positions("function f(x)\n    [a, b] = feval('size', x);\n    disp(a);\nend\n"),
            vec![(2, 9)]
        );
    }

    #[test]
    fn asglu_reported_when_eval_is_in_scope() {
        // Probed: MATLAB reports `b` (eval does not stop the analysis).
        assert_eq!(
            asglu_positions(
                "function f(x)\n    [a, b] = size(x);\n    eval('1');\n    disp(a);\nend\n"
            ),
            vec![(2, 9)]
        );
    }

    // -- Partial stores: dead indexed stores are NASGU -----------------------

    #[test]
    fn nasgu_fires_on_dead_indexed_store_at_function_end() {
        // Probed: `x(3) = 1` and `c{1} = 2` are NASGU, not STRNU.
        assert_eq!(
            nasgu_lines("function f()\n    x = zeros(1, 3);\n    x(2) = 1;\nend\n"),
            vec![3]
        );
        assert_eq!(nasgu_lines("function f()\n    c{1} = 2;\nend\n"), vec![2]);
    }

    #[test]
    fn nasgu_fires_on_indexed_store_overwritten_as_a_whole() {
        assert_eq!(nasgu_lines("x(3) = 1;\nx = 2;\ndisp(x)\n"), vec![1]);
    }

    #[test]
    fn nasgu_partial_store_read_later_is_not_dead() {
        assert!(
            nasgu_lines("function f()\n    x = 1;\n    x(2) = 2;\n    disp(x)\nend\n").is_empty()
        );
    }

    #[test]
    fn nasgu_ignores_field_stores() {
        // Field stores are STRNU (check_strnu), so NASGU stays silent on them.
        assert!(nasgu_lines("function f()\n    s.a = 1;\nend\n").is_empty());
    }

    #[test]
    fn nasgu_fires_on_unused_assignment() {
        let diags = lint_file(&*engine(), "function foo()\n    x = 1;\nend\n");
        assert!(has_id(&diags, "NASGU"), "got: {diags:?}");
    }

    #[test]
    fn nasgu_function_store_at_end_is_dead() {
        assert_eq!(nasgu_lines("function f01()\n    b = 3;\nend\n"), vec![2]);
    }

    #[test]
    fn nasgu_function_overwritten_before_use() {
        let source = "function f02()\n    b = 3;\n    b = 4;\n    disp(b)\nend\n";
        assert_eq!(nasgu_lines(source), vec![2]);
    }

    #[test]
    fn nasgu_function_output_overwritten_only_earlier_store() {
        let source = "function y = f03()\n    y = 3;\n    y = 4;\nend\n";
        assert_eq!(nasgu_lines(source), vec![2]);
    }

    #[test]
    fn nasgu_function_output_value_is_read_by_caller() {
        assert!(nasgu_lines("function y = f04()\n    y = 3;\nend\n").is_empty());
    }

    #[test]
    fn nasgu_function_final_store_after_read() {
        let source = "function f05()\n    a = 1;\n    disp(a);\n    a = 2;\nend\n";
        assert_eq!(nasgu_lines(source), vec![4]);
    }

    #[test]
    fn nasgu_function_conditional_store_at_end() {
        let source = "function f06(k)\n    c = 5;\n    if k, c = 6; end\nend\n";
        assert_eq!(nasgu_lines(source), vec![2, 3]);
    }

    #[test]
    fn nasgu_function_self_reference_is_read() {
        // `x = x + 1` reads the old value; the new value is never read.
        let source = "function f07()\n    x = 1;\n    x = x + 1;\nend\n";
        assert_eq!(nasgu_lines(source), vec![3]);
    }

    #[test]
    fn nasgu_function_loop_accumulator_is_read_next_pass() {
        let source =
            "function f08()\n    s = 0;\n    for i = 1:3\n        s = s + i;\n    end\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_function_loop_store_unread_at_end() {
        let source = "function f09()\n    for i = 1:2\n        t = i;\n    end\nend\n";
        assert_eq!(nasgu_lines(source), vec![3]);
    }

    #[test]
    fn nasgu_function_clear_kills() {
        assert_eq!(
            nasgu_lines("function f10()\n    a = 1;\n    clear a;\nend\n"),
            vec![2]
        );
    }

    #[test]
    fn nasgu_function_two_stores_unread() {
        let source = "function f11()\n    a = 1;\n    a = 2;\nend\n";
        assert_eq!(nasgu_lines(source), vec![2, 3]);
    }

    #[test]
    fn nasgu_function_global_is_not_reported() {
        assert!(nasgu_lines("function f12()\n    global g\n    g = 1;\nend\n").is_empty());
    }

    #[test]
    fn nasgu_function_eval_does_not_stop_analysis() {
        // Probed: MATLAB reports the store that only eval reads.
        let source = "function f13()\n    a = 1;\n    eval('disp(a)');\nend\n";
        assert_eq!(nasgu_lines(source), vec![2]);
    }

    #[test]
    fn nasgu_function_constant_true_if_kills_earlier_store() {
        // Probed: `if 1` always runs its body, so the store before it is dead.
        let source = "function r = f(x)\nr = 0;\nif 1\n  r = 1;\nend\nend\n";
        assert_eq!(nasgu_lines(source), vec![2]);
    }

    #[test]
    fn nasgu_function_constant_variable_condition() {
        // Probed: `k = 1; if k` runs its body; the store before it is dead.
        let source = "function r = f(x)\nr = 0;\nk = 1;\nif k\n  r = 1;\nend\nend\n";
        assert_eq!(nasgu_lines(source), vec![2]);
    }

    #[test]
    fn nasgu_function_unknown_variable_condition_keeps_store() {
        // Probed: `k` is assigned under a condition, so it is not constant.
        let source =
            "function r = f(x)\nr = 0;\nk = 1;\nif x\n  k = 0;\nend\nif k\n  r = 1;\nend\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_function_constant_false_branch_is_not_reported() {
        // Probed: the store in `if 0` is unreachable, and so reports nothing.
        assert!(nasgu_lines("function r = f(x)\nr = 0;\nif 0\n  r = 1;\nend\nend\n").is_empty());
    }

    #[test]
    fn nasgu_function_comparison_is_not_folded() {
        // Probed: `1 == 1` is not evaluated, so the store before it stays live.
        assert!(
            nasgu_lines("function r = f(x)\nr = 0;\nif 1 == 1\n  r = 1;\nend\nend\n").is_empty()
        );
    }

    #[test]
    fn nasgu_function_constant_true_else_is_dead() {
        // Probed: the `else` of `if 1` never runs, so both stores before the end are kept only by the body.
        let source = "function r = f(x)\nr = 0;\nif 1\n  r = 1;\nelse\n  r = 2;\nend\nend\n";
        assert_eq!(nasgu_lines(source), vec![2]);
    }

    #[test]
    fn nasgu_function_while_true_store_before_break() {
        // Probed: a `while 1` body always runs, so the store before it is dead.
        let source = "function r = f(x)\nr = 0;\nwhile 1\n  r = 1;\n  break;\nend\nend\n";
        assert_eq!(nasgu_lines(source), vec![2]);
    }

    #[test]
    fn nasgu_function_on_cleanup_is_not_reported() {
        // Probed: MATLAB does not report a dead `onCleanup` object.
        assert!(nasgu_lines("function f()\n    c = onCleanup(@() disp(1));\nend\n").is_empty());
    }

    #[test]
    fn nasgu_function_arguments_block_does_not_stop_analysis() {
        // Probed (codegen fixture): MATLAB reports the dead store after an `arguments` block.
        let source = "function r = f2(a)\narguments\n    a (1,1) double\nend\np = mobileRobotPRM;\nr = a;\nend\n";
        assert_eq!(nasgu_lines(source), vec![5]);
    }

    #[test]
    fn nasgu_function_method_named_save_is_not_dynamic() {
        // Probed (codegen fixture): `obj.save(1)` is a method call, not the function `save`.
        let source = "function f()\n    y = 1;\n    z = obj.save(1);\nend\n";
        assert_eq!(nasgu_lines(source), vec![2, 3]);
    }

    #[test]
    fn nasgu_function_after_infinite_loop_is_unreachable() {
        // Probed: MATLAB reports nothing after a `while true` without `break`.
        let source =
            "function f()\n    k = 1;\n    while true\n        disp(k)\n    end\n    a = 2;\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_function_try_catch_keeps_value() {
        let source = "function y = f14(k)\n    a = 1;\n    try\n        a = 2;\n    catch\n    end\n    y = a;\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_function_constant_if_both_stores() {
        let source = "function f15()\n    a = 1;\n    if true\n        a = 2;\n    end\nend\n";
        assert_eq!(nasgu_lines(source), vec![2, 4]);
    }

    #[test]
    fn nasgu_function_partial_store_is_not_reported() {
        // The partial store reads `x`, so the later full store is still read by disp.
        let source = "function f16()\n    x.a = 1;\n    x = 2;\n    disp(x)\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_function_read_before_later_overwrite() {
        let source = "function f17()\n    a = 1;\n    b = a;\n    a = 2;\n    disp(b)\nend\n";
        assert_eq!(nasgu_lines(source), vec![4]);
    }

    #[test]
    fn nasgu_function_lambda_capture_is_not_reported() {
        let source = "function f18()\n    a = 1;\n    g = @() a;\n    a = 2;\n    disp(g())\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_function_assignin_is_not_analysed() {
        let source = "function f19()\n    a = 1;\n    assignin('base', 'z', 1);\n    a = 2;\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_function_persistent_is_not_reported() {
        assert!(nasgu_lines("function f20()\n    persistent p\n    p = 1;\nend\n").is_empty());
    }

    #[test]
    fn nasgu_function_evalin_is_not_analysed() {
        let source =
            "function f21()\n    a = 1;\n    evalin('base', 'disp(1)');\n    a = 2;\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_function_output_overwritten_before_return() {
        let source = "function y = f22()\n    y = 1;\n    y = 2;\n    return;\nend\n";
        assert_eq!(nasgu_lines(source), vec![2]);
    }

    #[test]
    fn nasgu_function_return_path_reads_value() {
        let source =
            "function f23()\n    a = 1;\n    if rand > 0.5, return; end\n    disp(a)\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_function_return_path_leaves_value_unread() {
        let source =
            "function f24()\n    a = 1;\n    if rand > 0.5, return; end\n    a = 2;\nend\n";
        assert_eq!(nasgu_lines(source), vec![2, 4]);
    }

    #[test]
    fn nasgu_function_break_leaves_loop_value_unread() {
        let source = "function f25()\n    for i = 1:3\n        t = i;\n        if i == 2, break; end\n    end\nend\n";
        assert_eq!(nasgu_lines(source), vec![3]);
    }

    #[test]
    fn nasgu_function_nested_function_is_not_analysed() {
        let source = "function f26()\n    a = 1;\n    g();\n    a = 2;\n    disp(a)\n    function g()\n        disp(a)\n    end\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_function_input_overwritten_unread() {
        assert_eq!(nasgu_lines("function f27(x)\n    x = 1;\nend\n"), vec![2]);
    }

    #[test]
    fn nasgu_function_try_body_read_in_catch_keeps_value() {
        let source = "function f28()\n    a = 1;\n    try\n        disp(a);\n    catch\n    end\n    a = 2;\n    disp(a)\nend\n";
        assert!(nasgu_lines(source).is_empty());
    }

    #[test]
    fn nasgu_ignores_script_workspace_variables() {
        // The values of a script remain in the workspace afterwards.
        let diags = lint_file(&*engine(), "x = linspace(0, 1, 10);\ny = x * 2;\n");
        assert!(!has_id(&diags, "NASGU"), "got: {diags:?}");
    }

    #[test]
    fn nasgu_ok_when_used() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    x = 1;\n    disp(x);\nend\n",
        );
        assert!(!has_id(&diags, "NASGU"), "got: {diags:?}");
    }
}
