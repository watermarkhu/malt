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
            // `varargout` is in the builtin ignore list for reads, but MATLAB
            // reports an unassigned `varargout` output (probed).
            if self.is_ignored(&def.name) && def.name != "varargout" {
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
    fn stout_classdef_constructor_not_reported() {
        // Probed (r7_U): the constructor named like its class returns the object,
        // so its unassigned output is not STOUT.
        let source =
            "classdef Cls\n    methods\n        function r = Cls(x)\n        end\n    end\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(
            !diags.iter().any(|d| d.rule_id == "STOUT"),
            "got: {diags:?}"
        );
    }

    #[test]
    fn stout_classdef_method_still_reported() {
        // Probed (r7_U): only the constructor is exempt; another method still gets STOUT.
        let source =
            "classdef Cls\n    methods\n        function r = m(obj)\n        end\n    end\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(diags.iter().any(|d| d.rule_id == "STOUT"), "got: {diags:?}");
    }

    #[test]
    fn stout_varargout_not_assigned() {
        // Probed: MATLAB reports STOUT for an unassigned `varargout` output and
        // not for `varargout{1} = ...` assigned on one path only.
        let source = "function varargout = foo(x)\n    disp(x);\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(diags.iter().any(|d| d.rule_id == "STOUT"), "got: {diags:?}");
        let source =
            "function varargout = foo(x)\n    if x\n        varargout{1} = 1;\n    end\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(
            !diags.iter().any(|d| d.rule_id == "STOUT"),
            "got: {diags:?}"
        );
    }

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
