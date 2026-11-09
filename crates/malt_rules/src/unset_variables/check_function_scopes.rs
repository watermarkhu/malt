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

            // Emit NODEF/USENS for possibly-unset variables.
            self.emit_possibly_unset_diagnostics(&da, &mut diagnostics);

            // Check output arguments: STOUT.
            self.check_output_args(scope, &da, &mut diagnostics);
        }

        diagnostics
    }

    /// Emit diagnostics for possibly-unset variables from definite-assignment results.
    fn emit_possibly_unset_diagnostics(
        &self,
        da: &DefiniteAssignment,
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
            let (rule_id, message) = if da.definitely_assigned.contains(&unset_var.name) {
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
        // z is assigned later on one path but used before being definitely assigned.
        let has_relevant = diags
            .iter()
            .any(|d| d.rule_id == "NODEF" || d.rule_id == "USENS");
        assert!(
            has_relevant,
            "expected NODEF or USENS for 'z', got: {diags:?}"
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
