use super::*;

use crate::analysis::control_flow::DefiniteAssignment;
use crate::analysis::symbols::DefKind;

impl UnsetVariablesEngine {
    /// Check that all output arguments are definitely assigned (STOUT).
    pub(crate) fn check_output_args(
        &self,
        scope: &crate::analysis::symbols::Scope,
        _da: &DefiniteAssignment,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for def in &scope.defs {
            if def.kind != DefKind::OutputArg {
                continue;
            }
            if self.is_ignored(&def.name) {
                continue;
            }

            // Check if the output variable is definitely assigned at function end.
            // The output arg name is pre-added to the assigned set by
            // collect_function_args, so we need to check if there is an actual
            // assignment (not just the output declaration itself). We look for
            // any Assignment-kind def in the scope for this variable name.
            let has_assignment = scope
                .defs
                .iter()
                .any(|d| d.name == def.name && d.kind == DefKind::Assignment);

            // If there's no assignment and the name is not in the definitely-
            // assigned set from the analysis (excluding the initial output arg
            // declaration), emit STOUT.
            if !has_assignment && !self.has_non_output_def(scope, &def.name) {
                diagnostics.push(Diagnostic {
                    rule_id: "STOUT",
                    message: "Function return value might be unset.".to_string(),
                    severity: Severity::Warning,
                    byte_range: def.byte_range.clone(),
                    line: def.line,
                    column: def.column,
                    fix: None,
                });
            }
        }
    }

    /// Check whether a variable has any non-OutputArg definition in the scope.
    fn has_non_output_def(&self, scope: &crate::analysis::symbols::Scope, name: &str) -> bool {
        scope
            .defs
            .iter()
            .any(|d| d.name == name && d.kind != DefKind::OutputArg)
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

    // -- STOUT tests --------------------------------------------------------

    #[test]
    fn stout_output_not_assigned() {
        let source = "\
function y = foo(x)
    disp(x);
end
";
        let diags = lint_file(&*engine(), source);
        assert!(
            diags.iter().any(|d| d.rule_id == "STOUT"),
            "expected STOUT for 'y', got: {diags:?}"
        );
    }

    #[test]
    fn stout_not_fired_when_output_assigned() {
        let source = "\
function y = foo(x)
    y = x + 1;
end
";
        let diags = lint_file(&*engine(), source);
        assert!(
            !diags.iter().any(|d| d.rule_id == "STOUT"),
            "should not fire STOUT when output is assigned"
        );
    }
}
