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
            let da = analyze_definite_assignment(func_node, source);

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
                        "Explicitly initialize this variable to avoid a potential uninitialized variable, or use a valid syntax for function call on line {}.",
                        unset_var.use_line
                    ),
                )
            } else {
                (
                    "NODEF",
                    "Variable might be used before it is defined.".to_string(),
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
    fn nodef_never_defined() {
        let source = "\
function foo()
    y = unknown_var + 1;
end
";
        let diags = lint_file(&*engine(), source);
        assert!(
            diags.iter().any(|d| d.rule_id == "NODEF"),
            "expected NODEF for 'unknown_var', got: {diags:?}"
        );
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
}
