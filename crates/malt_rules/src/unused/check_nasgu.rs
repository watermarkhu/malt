//! NASGU check: value assigned to a variable might be unused.
//!
//! An assignment is reported when no path from it reads the variable before
//! another full assignment or `clear` replaces it. Scripts keep their final
//! values in the workspace; a function reports a value that reaches its end
//! unread, except for its output arguments. The analysis is in
//! [`crate::analysis::liveness`], which reports nothing for a scope whose
//! control flow it cannot see through.

use super::*;
use crate::analysis::liveness::dead_stores;

impl UnusedEngine {
    /// Run NASGU check: value assigned to a variable might be unused.
    pub(crate) fn check_nasgu(
        &self,
        table: &SymbolTable,
        tree: &tree_sitter::Tree,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if self.is_check_disabled("NASGU") {
            return;
        }

        for index in 0..table.scopes.len() {
            let Some(dead) = dead_stores(table, index, tree, source) else {
                continue;
            };
            let scope = &table.scopes[index];
            for def_index in dead {
                let def = &scope.defs[def_index];
                if self.should_ignore_name(&def.name) {
                    continue;
                }
                diagnostics.push(Diagnostic {
                    rule_id: "NASGU",
                    message: "Value assigned to variable might be unused.".to_string(),
                    severity: Severity::Warning,
                    byte_range: def.byte_range.clone(),
                    line: def.line,
                    column: def.column,
                    fix: None,
                });
            }
        }
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
    fn nasgu_script_eval_is_not_analysed() {
        // MATLAB reports line 1 here; eval may read `a`, so nothing is reported.
        assert!(nasgu_lines("a = 1;\neval('disp(2)');\na = 2;\n").is_empty());
    }

    // -- Probes: functions ---------------------------------------------------

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
    fn nasgu_function_eval_is_not_analysed() {
        let source = "function f13()\n    a = 1;\n    eval('disp(a)');\nend\n";
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
