use super::*;

use crate::analysis::control_flow::{analyze_definite_assignment, DefiniteAssignment};
use crate::analysis::maybe_defined::analyze_may_defined;
use crate::analysis::symbols::{DefKind, ScopeKind};

impl UnsetVariablesEngine {
    /// Run definite-assignment analysis on function scopes and emit diagnostics.
    pub(crate) fn check_function_scopes(
        &self,
        tree: &Tree,
        source: &str,
        symbol_table: &SymbolTable,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut functions = None;

        for scope in &symbol_table.scopes {
            // Only process function-like scopes.
            if !matches!(
                scope.kind,
                ScopeKind::Function
                    | ScopeKind::LocalFunction
                    | ScopeKind::NestedFunction
                    | ScopeKind::Method
            ) {
                continue;
            }

            // Find the corresponding function_definition node.
            let functions = functions.get_or_insert_with(|| function_nodes_by_range(tree));
            let range = (scope.byte_range.start, scope.byte_range.end);
            let Some(&func_node) = functions.get(&range) else {
                continue;
            };

            // Run definite-assignment analysis.
            let mut da = analyze_definite_assignment(func_node, source);

            // A name that is never defined in the function is a function call
            // (`p = inputParser;`), not an unset variable.
            da.possibly_unset.retain(|u| scope.is_defined(&u.name));

            // A nested function shares the workspace of the functions around it.
            if scope.kind == ScopeKind::NestedFunction {
                let mut outer = HashSet::new();
                let mut parent = scope.parent;
                while let Some(idx) = parent {
                    let p = &symbol_table.scopes[idx];
                    outer.extend(p.defs.iter().map(|d| d.name.clone()));
                    parent = p.parent;
                }
                da.possibly_unset.retain(|u| !outer.contains(&u.name));
            }

            // A use inside a nested function body shares the parent's workspace, so
            // MATLAB reports nothing for it here (probed: `function g(), disp(u); end`).
            da.possibly_unset.retain(|u| {
                !scope.children.iter().any(|&c| {
                    let child = &symbol_table.scopes[c];
                    child.kind == ScopeKind::NestedFunction
                        && child.byte_range.contains(&u.use_byte_range.start)
                })
            });

            // A `global` or `persistent` declaration makes the name a workspace
            // variable: MATLAB reports neither NODEF nor USENS for it (probed,
            // `disp(g)` before `global g`, and `p = p + 1` before `persistent p`).
            da.possibly_unset.retain(|u| {
                !scope.defs.iter().any(|d| {
                    d.name == u.name
                        && matches!(
                            d.kind,
                            crate::analysis::symbols::DefKind::Global
                                | crate::analysis::symbols::DefKind::Persistent
                        )
                })
            });

            // Emit NODEF/USENS for possibly-unset variables.
            // NODEF needs a definition later in the scope and none before the use.
            let nodef_applies = |name: &str, use_start: usize| {
                let assigns: Vec<usize> = scope
                    .defs
                    .iter()
                    .filter(|d| {
                        d.name == name && d.kind == crate::analysis::symbols::DefKind::Assignment
                    })
                    .map(|d| d.byte_range.start)
                    .collect();
                (!assigns.is_empty()
                    && assigns.iter().all(|&start| start > use_start)
                    && !Self::assigned_in_enclosing_loop(func_node, use_start, &assigns))
                    || Self::cleared_before(func_node, source, scope, name, use_start)
            };
            let cleared_input = |name: &str, use_start: usize| {
                crate::good_practices::is_cleared_parfor_input(
                    tree.root_node(),
                    source,
                    name,
                    use_start,
                )
            };
            self.emit_possibly_unset_diagnostics(
                &da,
                nodef_applies,
                cleared_input,
                &mut diagnostics,
            );

            // NODEF and SVNODEF: reads that no definition can reach. A nested function
            // reading a parent variable is SVNODEF when no parent definition is live
            // (probed: `if false, a = 1; end` in the parent, then `v = a` in the child).
            let may = analyze_may_defined(func_node, source);
            let mut outer_names: HashSet<String> = HashSet::new();
            let mut outer_live: HashSet<String> = HashSet::new();
            if scope.kind == ScopeKind::NestedFunction {
                let mut parent = scope.parent;
                while let Some(idx) = parent {
                    let p = &symbol_table.scopes[idx];
                    outer_names.extend(
                        p.defs
                            .iter()
                            .filter(|d| d.kind != DefKind::NestedFunction)
                            .map(|d| d.name.clone()),
                    );
                    if let Some(&outer_node) =
                        functions.get(&(p.byte_range.start, p.byte_range.end))
                    {
                        let outer = analyze_may_defined(outer_node, source);
                        outer_live.extend(outer.defined);
                        outer_live.extend(outer.arguments);
                    }
                    parent = p.parent;
                }
            }
            // A name that is only a nested function (`@report`) is not a variable.
            let mut reads = may.undefined_reads;
            reads.retain(|u| {
                scope
                    .defs
                    .iter()
                    .any(|d| d.name == u.name && d.kind != DefKind::NestedFunction)
                    || outer_names.contains(&u.name)
            });
            // A use inside a nested function body shares the parent's workspace, so it
            // is the nested function's to report.
            reads.retain(|u| {
                !scope.children.iter().any(|&c| {
                    let child = &symbol_table.scopes[c];
                    child.kind == ScopeKind::NestedFunction
                        && child.byte_range.contains(&u.use_byte_range.start)
                })
            });
            // A `global` or `persistent` name is a workspace variable (see below).
            reads.retain(|u| {
                !scope.defs.iter().any(|d| {
                    d.name == u.name && matches!(d.kind, DefKind::Global | DefKind::Persistent)
                })
            });
            // MATLAB reports one NODEF or SVNODEF per name, at its first such read.
            let mut reported: HashSet<String> = HashSet::new();
            for read in &reads {
                let name = read.name.as_str();
                if self.is_ignored(name) || reported.contains(name) {
                    continue;
                }
                // A nested function shares its parent's variables, even when it also
                // assigns them (probed: `calls = calls + 1` in a nested function).
                let (rule_id, message) = if outer_names.contains(name) {
                    if outer_live.contains(name) {
                        continue;
                    }
                    (
                        "SVNODEF",
                        format!(
                            "Variable '{name}' might not have a value, because variable definition may not be executed."
                        ),
                    )
                } else {
                    (
                        "NODEF",
                        format!("Variable '{name}' might be used before it is defined."),
                    )
                };
                // A parfor input cleared before the loop is PFIIN, not NODEF (probed).
                if cleared_input(name, read.use_byte_range.start) {
                    continue;
                }
                reported.insert(read.name.clone());
                diagnostics.push(Diagnostic {
                    rule_id,
                    message,
                    severity: Severity::Warning,
                    byte_range: read.use_byte_range.clone(),
                    line: read.use_line,
                    column: read.use_column,
                    fix: None,
                });
            }

            // Check output arguments: STOUT. A classdef constructor returns the
            // object itself, so its unassigned output is not reported (probed).
            if !Self::is_classdef_constructor(func_node, source, &scope.name) {
                // An output that is also an input is defined on entry.
                let mut defined = may.defined.clone();
                defined.extend(
                    scope
                        .defs
                        .iter()
                        .filter(|d| d.kind == DefKind::InputArg)
                        .map(|d| d.name.clone()),
                );
                self.check_output_args(scope, &defined, &mut diagnostics);
            }
        }

        diagnostics
    }

    /// Whether a loop around the use at `use_start` contains one of `assigns`: the
    /// assignment reaches the use on the next iteration, so MATLAB reports nothing
    /// (probed: `while q` with `q = 1` in the body, `q` read before `q = i` in a
    /// `for` body).
    fn assigned_in_enclosing_loop(
        func: tree_sitter::Node,
        use_start: usize,
        assigns: &[usize],
    ) -> bool {
        let Some(mut node) = func.descendant_for_byte_range(use_start, use_start + 1) else {
            return false;
        };
        loop {
            if matches!(node.kind(), "for_statement" | "while_statement")
                && assigns.iter().any(|a| node.byte_range().contains(a))
            {
                return true;
            }
            match node.parent() {
                Some(parent) => node = parent,
                None => return false,
            }
        }
    }

    /// Whether a plain `clear name` statement of the function body comes before
    /// the use, and no assignment of `name` comes between the last such clear and
    /// the use: no definition reaches the use, so MATLAB reports NODEF.
    fn cleared_before(
        func: tree_sitter::Node,
        source: &str,
        scope: &crate::analysis::symbols::Scope,
        name: &str,
        use_start: usize,
    ) -> bool {
        let mut func_cursor = func.walk();
        let Some(block) = func
            .named_children(&mut func_cursor)
            .find(|n| n.kind() == "block")
        else {
            return false;
        };
        let mut cursor = block.walk();
        let last_clear = block
            .named_children(&mut cursor)
            .filter(|s| s.kind() == "command" && s.end_byte() <= use_start)
            .filter(|s| {
                crate::analysis::liveness::clear_names(&source[s.byte_range()])
                    .is_some_and(|names| names.iter().any(|n| n == name))
            })
            .map(|s| s.end_byte())
            .max();
        last_clear.is_some_and(|clear| {
            !scope.defs.iter().any(|d| {
                d.name == name
                    && d.kind == crate::analysis::symbols::DefKind::Assignment
                    && d.byte_range.start >= clear
                    && d.byte_range.start < use_start
            })
        })
    }

    /// Whether `func` is the constructor of the classdef it sits in: a method named
    /// like its class.
    fn is_classdef_constructor(func: tree_sitter::Node, source: &str, name: &str) -> bool {
        let mut current = func.parent();
        while let Some(node) = current {
            if node.kind() == "class_definition" {
                return node
                    .child_by_field_name("name")
                    .is_some_and(|n| &source[n.byte_range()] == name);
            }
            current = node.parent();
        }
        false
    }

    /// Emit USENS for possibly-unset variables from definite-assignment results.
    /// NODEF and SVNODEF come from the reaching-definition analysis instead.
    fn emit_possibly_unset_diagnostics(
        &self,
        da: &DefiniteAssignment,
        nodef_applies: impl Fn(&str, usize) -> bool,
        cleared_input: impl Fn(&str, usize) -> bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // USENS is off under the `mathworks` preset: MATLAB never reported it
        // in the probes or the fixtures.
        if !self.usens_on {
            return;
        }
        // Track which variable names have already been reported to avoid duplicates.
        let mut reported: HashSet<String> = HashSet::new();

        for unset_var in &da.possibly_unset {
            if self.is_ignored(&unset_var.name) || reported.contains(&unset_var.name) {
                continue;
            }
            if cleared_input(&unset_var.name, unset_var.use_byte_range.start) {
                continue;
            }
            // Used but set on some paths only, and not a use before every assignment.
            if !da.definitely_assigned.contains(&unset_var.name)
                || nodef_applies(&unset_var.name, unset_var.use_byte_range.start)
            {
                continue;
            }
            reported.insert(unset_var.name.clone());
            diagnostics.push(Diagnostic {
                rule_id: "USENS",
                message: format!(
                    "Explicitly initialize variable '{}' to avoid a potential uninitialized variable, or use a valid syntax for function call on line {}.",
                    unset_var.name, unset_var.use_line
                ),
                severity: Severity::Warning,
                byte_range: unset_var.use_byte_range.clone(),
                line: unset_var.use_line,
                column: unset_var.use_column,
                fix: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::lint_file;
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnsetVariablesEngine::from_config(&Config::default())
    }

    // -- NODEF tests --------------------------------------------------------

    #[test]
    fn nodef_silent_for_parfor_input_cleared_before_loop() {
        // Probed (e_ii01): `clear k` before a parfor loop that reads `k` is PFIIN at the
        // read, not NODEF.
        let source = "function r = f(n)\nk = 1;\nclear k;\nr = zeros(1, n);\nparfor i = 1:n\n    r(i) = k;\nend\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(diags.iter().all(|d| d.rule_id != "NODEF"), "got: {diags:?}");
    }

    #[test]
    fn nodef_use_before_assign() {
        let source = "\
function foo()
    y = x + 1;
    x = 5;
end
";
        let diags = lint_file(&*engine(), source);
        // x is assigned later in the function, so it is "used but not set
        // in all code paths" (USENS) rather than completely undefined.
        assert!(
            diags
                .iter()
                .any(|d| d.rule_id == "NODEF" || d.rule_id == "USENS"),
            "expected NODEF or USENS for 'x', got: {diags:?}"
        );
    }

    #[test]
    fn nodef_name_value_argument_name_is_not_a_read() {
        // Probed: `NumClasses = NaN` in `struct(...)` names an argument, it does not read
        // the variable; the assignment after the call leaves MATLAB silent.
        let source = "\
function s = foo()
    s = struct(NumClasses = NaN, Counts = []);
    NumClasses = 3;
    disp(NumClasses)
end
";
        let diags = lint_file(&*engine(), source);
        assert!(
            !diags
                .iter()
                .any(|d| d.rule_id == "NODEF" || d.rule_id == "USENS"),
            "expected no NODEF/USENS for a name-value argument name, got: {diags:?}"
        );
    }

    #[test]
    fn nodef_name_value_argument_value_is_a_read() {
        // Probed: the value `y` of `A = y` is a read, so NODEF fires there when `y` is
        // assigned later.
        let source = "\
function s = foo()
    s = struct(A = y);
    y = 1;
    disp(s)
end
";
        let diags = lint_file(&*engine(), source);
        let nodef: Vec<_> = diags.iter().filter(|d| d.rule_id == "NODEF").collect();
        assert_eq!(nodef.len(), 1, "got: {diags:?}");
        assert_eq!((nodef[0].line, nodef[0].column), (2, 20));
    }

    #[test]
    fn never_defined_name_is_a_function_call() {
        let source = "\
function foo()
    y = unknown_var + 1;
end
";
        let diags = lint_file(&*engine(), source);
        // Not defined anywhere in the function: MATLAB resolves it as a call.
        assert!(
            !diags.iter().any(|d| d.rule_id == "NODEF"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn nodef_fires_when_defined_only_later() {
        let src = "function foo()\n    y = x + 1;\n    x = 2;\n    disp(y + x);\nend\n";
        assert!(lint_file(&*engine(), src)
            .iter()
            .any(|d| d.rule_id == "NODEF" || d.rule_id == "USENS"));
    }

    #[test]
    fn field_names_are_not_variables() {
        let src = "function foo(p)\n    s = p.Results.Scale;\n    disp(s);\nend\n";
        assert!(!lint_file(&*engine(), src)
            .iter()
            .any(|d| d.rule_id == "NODEF"));
    }

    #[test]
    fn nodef_not_fired_when_assigned() {
        let source = "\
function foo()
    x = 5;
    y = x + 1;
end
";
        let diags = lint_file(&*engine(), source);
        assert!(
            !diags.iter().any(|d| d.rule_id == "NODEF"),
            "should not fire NODEF for 'x' when it is assigned before use"
        );
    }

    #[test]
    fn global_or_persistent_declaration_suppresses_unset_reports() {
        // Probed: MATLAB reports only GPFST and GVMIS for these, never NODEF or USENS.
        assert!(nodef_ids("function g()\n    disp(g)\n    global g\nend\n").is_empty());
        assert!(nodef_ids("function g()\n    p = p + 1;\n    persistent p\nend\n").is_empty());
    }

    #[test]
    fn use_in_nested_function_shares_the_parent_workspace() {
        // Probed: MATLAB reports nothing for `u` used in a nested function before
        // the parent assigns it.
        let src = "function f()\n    function g(), disp(u); end\n    u = 1; g();\nend\n";
        assert!(nodef_ids(src).is_empty(), "got: {:?}", nodef_ids(src));
    }

    // -- USENS tests --------------------------------------------------------

    #[test]
    fn usens_variable_not_set_in_all_paths() {
        let source = "\
function y = foo(x)
    if x > 0
        z = 1;
    end
    y = z;
end
";
        let diags = lint_file(&*engine(), source);
        // MATLAB is silent on a conditional definition followed by a use (probe q16).
        let has_relevant = diags
            .iter()
            .any(|d| d.rule_id == "NODEF" || d.rule_id == "USENS");
        assert!(
            !has_relevant,
            "expected no NODEF or USENS for 'z', got: {diags:?}"
        );
    }

    fn nodef_ids(src: &str) -> Vec<&'static str> {
        lint_file(&*engine(), src)
            .iter()
            .map(|d| d.rule_id)
            .filter(|id| *id == "NODEF" || *id == "USENS")
            .collect()
    }

    #[test]
    fn first_indexed_assignment_creates_the_variable() {
        assert!(nodef_ids(
            "function foo()\n    A(2) = 5;\n    s.f = 1;\n    disp(A); disp(s);\nend\n"
        )
        .is_empty());
    }

    #[test]
    fn partial_assignment_to_an_input_is_fine() {
        assert!(nodef_ids("function foo(A)\n    A(2) = 5;\n    disp(A);\nend\n").is_empty());
    }

    #[test]
    fn lambda_parameters_are_bound() {
        assert!(
            nodef_ids("function foo()\n    h = @(x) x + 1;\n    disp(h(1));\nend\n").is_empty()
        );
    }

    #[test]
    fn nested_function_sees_the_parent_workspace() {
        let src = "function r = foo()\n    r = 1;\n    function inner()\n        disp(r);\n    end\nend\n";
        assert!(nodef_ids(src).is_empty());
    }

    #[test]
    fn a_branch_that_errors_does_not_leave_the_variable_unset() {
        let src = "function r = foo(m)\n    switch m\n        case 1\n            s = 1;\n        otherwise\n            error('bad');\n    end\n    r = s;\nend\n";
        assert!(nodef_ids(src).is_empty(), "{:?}", nodef_ids(src));
        let src = "function r = foo(c)\n    if c\n        s = 1;\n    else\n        return\n    end\n    r = s;\nend\n";
        assert!(nodef_ids(src).is_empty());
    }

    fn nodef_lines(src: &str) -> Vec<(usize, usize)> {
        lint_file(&*engine(), src)
            .iter()
            .filter(|d| d.rule_id == "NODEF")
            .map(|d| (d.line, d.column))
            .collect()
    }

    #[test]
    fn nodef_in_if_and_while_condition() {
        // Probe (custom__a, o, p): a condition read before the later assignment.
        assert_eq!(
            nodef_lines("function foo(x)\nif q\n    x = 1;\nend\nq = 2;\nend\n"),
            vec![(2, 4)]
        );
        assert_eq!(
            nodef_lines("function foo(x)\nif x && q\n    y = 1;\nend\nq = 2;\nend\n"),
            vec![(2, 9)]
        );
        assert_eq!(
            nodef_lines("function foo()\nwhile q\n    x = 1;\nend\nq = 2;\nend\n"),
            vec![(2, 7)]
        );
    }

    #[test]
    fn nodef_silent_when_a_loop_assigns_the_name() {
        // Probe (v12, m): the assignment in the loop body reaches the read on the
        // next iteration.
        assert!(nodef_lines("function f()\nwhile q\n    q = 1;\nend\nend\n").is_empty());
        assert!(nodef_lines(
            "function f()\nfor i = 1:2\n    if i > 1, y = q; end\n    q = i;\nend\nend\n"
        )
        .is_empty());
    }

    #[test]
    fn nodef_for_bare_command_name_before_assignment() {
        // Probe (v4, gp-a__t10 `keyboard;`): `q;` is a read of `q` (NODEF and VUNUS).
        assert_eq!(nodef_lines("function f()\nq;\nq = 2;\nend\n"), vec![(2, 1)]);
        // A name the function never assigns is a function call (probe v9).
        assert!(nodef_lines("function f()\nq;\nend\n").is_empty());
    }

    #[test]
    fn nodef_for_ans_only_when_assigned_later() {
        // Probe (gp-b__t2, f): `ans` read before the function assigns it.
        assert_eq!(
            nodef_lines("function f()\nans\nans = 5;\nend\n"),
            vec![(2, 1)]
        );
        // Probe (v11): with no assignment, `ans` is NOANS only.
        assert!(nodef_lines("function f()\ny = ans;\nend\n").is_empty());
    }

    /// The NODEF, SVNODEF and STOUT findings of `src`, with their lines.
    fn undefined_lines(src: &str) -> Vec<(&'static str, usize)> {
        lint_file(&*engine(), src)
            .iter()
            .filter(|d| matches!(d.rule_id, "NODEF" | "SVNODEF" | "STOUT"))
            .map(|d| (d.rule_id, d.line))
            .collect()
    }

    #[test]
    fn parfor_names_are_not_available_after_the_loop() {
        // Probe (yc_f14, yc_f15, yc_u14, yc_u23): the body's temporary and the loop
        // variable have no definition after the loop.
        let src = "function r = f(n)\nr = 0;\nparfor k = 1:n, t = k; end\nr = t + k;\nend\n";
        assert_eq!(undefined_lines(src), vec![("NODEF", 4), ("NODEF", 4)]);
        let src = "function r = f(n)\nparfor k = 1:n\n    t = k;\nend\np = t;\nr = p;\nend\n";
        assert_eq!(undefined_lines(src), vec![("NODEF", 5)]);
    }

    #[test]
    fn parfor_body_reads_are_not_nodef() {
        // Probe (yc_p01, yc_p04): a read inside the body is PFIIN, never NODEF.
        let src = "function r = f(n)\nr = 0;\nparfor k = 1:n\n    r = r + q;\nend\nq = 1;\nend\n";
        assert!(
            undefined_lines(src).is_empty(),
            "{:?}",
            undefined_lines(src)
        );
    }

    #[test]
    fn parfor_sliced_assignment_is_available_after_the_loop() {
        // Probe (yc_u20): `x(k) = k` in a parfor body leaves `x` available.
        let src = "function r = f(n)\nparfor k = 1:n\n    x(k) = k;\nend\nr = x;\nend\n";
        assert!(undefined_lines(src).is_empty());
    }

    #[test]
    fn dead_constant_branch_defines_nothing() {
        // Probes (yc_f24, yc_u01, yc_u02): `if false`, `if 0` and `while false` bodies
        // never run, so the read after them is NODEF.
        for src in [
            "function r = f(c)\nif false, a = 1; end\nr = a;\nend\n",
            "function r = f(c)\nif 0, a = 1; end\nr = a;\nend\n",
            "function r = f(c)\nwhile false, a = 1; end\nr = a;\nend\n",
        ] {
            assert_eq!(undefined_lines(src), vec![("NODEF", 3)], "{src}");
        }
        // Probe (yc_f23): a constant-true `if` always runs its body.
        assert!(
            undefined_lines("function r = f(c)\nif true, a = 1; end\nr = a;\nend\n").is_empty()
        );
    }

    #[test]
    fn conditional_definition_and_for_loop_are_silent() {
        // Probes (yc_f01, yc_f16): MATLAB is silent when a definition may run.
        assert!(undefined_lines("function r = f(c)\nif c, a = 1; end\nr = a;\nend\n").is_empty());
        assert!(
            undefined_lines("function r = f(n)\nfor k = 1:n, t = k; end\nr = t;\nend\n").is_empty()
        );
    }

    #[test]
    fn clear_removes_the_value_unless_a_later_definition_may_run() {
        // Probes (yc_u26, yc_u25): a plain `clear` leaves the read unset; a conditional
        // definition after it is silent.
        assert_eq!(
            undefined_lines("function r = f(c)\na = 1;\nclear a\nr = a;\nend\n"),
            vec![("NODEF", 4)]
        );
        let src = "function r = f(c)\nif c, a = 1; end\nclear a\nif c, a = 2; end\nr = a;\nend\n";
        assert!(undefined_lines(src).is_empty());
    }

    #[test]
    fn nested_read_of_a_dead_parent_definition_is_svnodef() {
        // Probes (yc_n03, yc_n13): the parent's only definition never runs.
        let src = "function r = f(c)\nif false, a = 1; end\nr = g();\n    function v = g()\n        v = a;\n    end\nend\n";
        assert_eq!(undefined_lines(src), vec![("SVNODEF", 5)]);
        let src = "function r = f(c)\ntry\n    error('x')\n    a = 1;\ncatch\nend\nr = g();\n    function v = g()\n        v = a;\n    end\nend\n";
        assert_eq!(undefined_lines(src), vec![("SVNODEF", 9)]);
    }

    #[test]
    fn nested_read_of_a_live_parent_definition_is_silent() {
        // Probes (yc_u44, yc_n04, yc_n02): a live definition, before or after the call
        // or cleared again, makes the parent variable silent in the nested function.
        let src = "function r = f(c)\nif c, a = 1; end\nr = g();\n    function v = g()\n        v = a;\n    end\nend\n";
        assert!(undefined_lines(src).is_empty());
        let src = "function r = f(c)\nr = g();\na = 1;\n    function v = g()\n        v = a;\n    end\nend\n";
        assert!(undefined_lines(src).is_empty());
    }

    #[test]
    fn nested_function_own_dead_definition_is_nodef() {
        // Probe (yc_u40): the nested function's own dead definition gives NODEF.
        let src = "function r = f(c)\nr = g(c);\n    function v = g(c)\n        if false, a = 1; end\n        v = a;\n    end\nend\n";
        assert_eq!(undefined_lines(src), vec![("NODEF", 5)]);
    }

    #[test]
    fn nested_function_assigning_the_parent_variable_is_silent() {
        // Probes (fix/unused__t3, corpus3/i/i_functional_style): the parent's `v` is
        // live before the nested function's `v = v + 1`.
        let src =
            "function t3()\n    v = 1;\n    function inner()\n        v = v + 1;\n    end\nend\n";
        assert!(undefined_lines(src).is_empty());
    }

    #[test]
    fn handle_to_a_nested_function_is_not_a_variable() {
        // Probe (corpus1/a/a_nested_funs): `@report` names a nested function.
        let src =
            "function f()\n    h = @report;\n    disp(h);\n    function report()\n    end\nend\n";
        assert!(undefined_lines(src).is_empty());
    }

    #[test]
    fn syms_and_named_load_define_variables_but_eval_does_not() {
        // Probes (corpus3/h/h_symbolic_math, yc_e04, yc_e07, fix/gp-a__t10): `syms`
        // and `load(file, 'x')` define; `eval` does not (MATLAB reports the read).
        assert!(
            undefined_lines("function f()\nsyms m k\nassume(m > 0)\nd = m + k;\nend\n").is_empty()
        );
        assert!(undefined_lines("function f()\nload('a.mat', 'q');\nr = q;\nend\n").is_empty());
        assert_eq!(
            undefined_lines("function f()\neval('q = 1;');\nr = q;\nq = 2;\nend\n"),
            vec![("NODEF", 3)]
        );
    }

    #[test]
    fn output_defined_by_a_multi_output_list_or_an_input_is_not_stout() {
        // Probes (corpus1/b/b_app_callbacks, corpus2/f/f_overload_ops,
        // skills-catalog/describeFeatures): `[varargout{1:n}] = ...` and an output that is
        // also an input.
        let src =
            "function varargout = f(varargin)\n[varargout{1:nargout}] = g(varargin{:});\nend\n";
        assert!(undefined_lines(src).iter().all(|(id, _)| *id != "STOUT"));
        let src = "function d = f(d)\nif ~istable(d)\n    error('x');\nend\nend\n";
        assert!(undefined_lines(src).iter().all(|(id, _)| *id != "STOUT"));
    }

    #[test]
    fn output_after_a_leading_return_is_stout() {
        // Probe (yc_u33): code after `return` never assigns the output.
        let src = "function r = f(c)\nreturn\na = 1;\nr = a;\nend\n";
        assert_eq!(undefined_lines(src), vec![("STOUT", 1)]);
    }

    #[test]
    fn spmd_body_runs_in_sequence() {
        // Probe (yc_u22): a definition in an spmd body is defined after it.
        assert!(
            undefined_lines("function r = f(n)\nspmd\n    t = 1;\nend\nr = t;\nend\n").is_empty()
        );
    }
}
