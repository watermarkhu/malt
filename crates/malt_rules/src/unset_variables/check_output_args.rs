use super::*;

use crate::analysis::control_flow::DefiniteAssignment;
use crate::analysis::symbols::DefKind;

impl UnsetVariablesEngine {
    /// Check that all output arguments are definitely assigned (STOUT).
    pub(crate) fn check_output_args(
        &self,
        scope: &crate::analysis::symbols::Scope,
        _da: &DefiniteAssignment,
        dead: &[std::ops::Range<usize>],
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

            // The output arg name is pre-added to the assigned set by
            // collect_function_args, so we need to check if there is an actual
            // definition (not just the output declaration itself) that can run.
            // Definitions in dead code do not count (probed).
            if !self.has_live_non_output_def(scope, &def.name, dead) {
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

    /// Whether a variable has a non-OutputArg definition outside the dead regions.
    fn has_live_non_output_def(
        &self,
        scope: &crate::analysis::symbols::Scope,
        name: &str,
        dead: &[std::ops::Range<usize>],
    ) -> bool {
        scope.defs.iter().any(|d| {
            d.name == name
                && d.kind != DefKind::OutputArg
                && !dead.iter().any(|r| r.contains(&d.byte_range.start))
        })
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

    #[test]
    fn stout_when_the_only_assignment_is_dead() {
        // Probed (o08, o10): an assignment in dead code does not set the output.
        let diags = lint_file(
            &*engine(),
            "function r = foo()\nif false\n    r = 1;\nend\nend\n",
        );
        assert!(diags.iter().any(|d| d.rule_id == "STOUT"), "got: {diags:?}");
        let diags = lint_file(&*engine(), "function r = foo()\nreturn;\nr = 1;\nend\n");
        assert!(diags.iter().any(|d| d.rule_id == "STOUT"), "got: {diags:?}");
    }

    #[test]
    fn stout_silent_when_the_assignment_runs_before_an_error() {
        // Probed (o09): the assignment before `error` runs.
        let diags = lint_file(&*engine(), "function r = foo()\nr = 1;\nerror('x');\nend\n");
        assert!(
            !diags.iter().any(|d| d.rule_id == "STOUT"),
            "got: {diags:?}"
        );
    }
}
