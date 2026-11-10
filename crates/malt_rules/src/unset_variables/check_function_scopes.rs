use super::*;

use crate::analysis::control_flow::{analyze_definite_assignment, DefiniteAssignment};
use crate::analysis::symbols::ScopeKind;

impl UnsetVariablesEngine {
    /// Run definite-assignment analysis on function scopes and emit diagnostics.
    pub(crate) fn check_function_scopes(
        &self,
        tree: &Tree,
        source: &str,
        symbol_table: &SymbolTable,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

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
            let Some(func_node) = find_function_node_at(tree, &scope.byte_range) else {
                continue;
            };

            // Run definite-assignment analysis.
            let mut da = analyze_definite_assignment(func_node, source);

            // A name that is never defined in the function is a function call
            // (`p = inputParser;`), not an unset variable.
            da.possibly_unset
                .retain(|u| scope.defs.iter().any(|d| d.name == u.name));

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
                !assigns.is_empty() && assigns.iter().all(|&start| start > use_start)
            };
            self.emit_possibly_unset_diagnostics(&da, nodef_applies, &mut diagnostics);

            // Check output arguments: STOUT. A classdef constructor returns the
            // object itself, so its unassigned output is not reported (probed).
            if !Self::is_classdef_constructor(func_node, source, &scope.name) {
                self.check_output_args(scope, &da, &mut diagnostics);
            }
        }

        diagnostics
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

    /// Emit diagnostics for possibly-unset variables from definite-assignment results.
    fn emit_possibly_unset_diagnostics(
        &self,
        da: &DefiniteAssignment,
        nodef_applies: impl Fn(&str, usize) -> bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Track which variable names have already been reported to avoid duplicates.
        let mut reported: HashSet<String> = HashSet::new();

        for unset_var in &da.possibly_unset {
            if self.is_ignored(&unset_var.name) {
                continue;
            }
            if reported.contains(&unset_var.name) {
                continue;
            }
            reported.insert(unset_var.name.clone());

            // If the variable is in the definitely-assigned set at function end
            // but was used before that point, it's USENS (used but not set in
            // all paths). Otherwise it's NODEF (never defined at all on some paths).
            if !da.definitely_assigned.contains(&unset_var.name)
                && !nodef_applies(&unset_var.name, unset_var.use_byte_range.start)
            {
                continue;
            }
            // Use before every assignment is NODEF (probed: `disp(u)`, then `u = 1`).
            let (rule_id, message) = if da.definitely_assigned.contains(&unset_var.name)
                && !nodef_applies(&unset_var.name, unset_var.use_byte_range.start)
            {
                (
                    "USENS",
                    format!(
                        "Explicitly initialize variable '{}' to avoid a potential uninitialized variable, or use a valid syntax for function call on line {}.",
                        unset_var.name, unset_var.use_line
                    ),
                )
            } else {
                (
                    "NODEF",
                    format!(
                        "Variable '{}' might be used before it is defined.",
                        unset_var.name
                    ),
                )
            };

            // USENS is off under the `mathworks` preset: MATLAB never reported it
            // in the probes or the fixtures.
            if rule_id == "USENS" && !self.usens_on {
                continue;
            }

            diagnostics.push(Diagnostic {
                rule_id,
                message,
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
}
