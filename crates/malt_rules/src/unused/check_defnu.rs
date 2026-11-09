//! DEFNU check: local function defined but never called.

use super::*;

/// Calls that reach functions by name at run time.
const DYNAMIC_CALLS: &[&str] = &["eval", "feval", "str2func"];

impl UnusedEngine {
    /// Run DEFNU check: local function defined but never called.
    pub(crate) fn check_defnu(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("DEFNU") {
            return;
        }

        // Collect all uses across all scopes to check if a local function is called.
        let all_uses: HashSet<&str> = table
            .scopes
            .iter()
            .flat_map(|s| s.uses.iter().map(|u| u.name.as_str()))
            .collect();

        // Probe: once the file calls eval, feval or str2func, MATLAB cannot tell
        // which local function is reached, and reports no DEFNU in the file.
        if DYNAMIC_CALLS.iter().any(|c| all_uses.contains(c)) {
            return;
        }

        for scope in &table.scopes {
            if scope.kind != ScopeKind::LocalFunction {
                continue;
            }
            let name = &scope.name;
            if name.is_empty() {
                continue;
            }
            if self.should_ignore_name(name) {
                continue;
            }
            if !all_uses.contains(name.as_str()) {
                diagnostics.push(Diagnostic {
                    rule_id: "DEFNU",
                    message: "Function might be unused.".to_string(),
                    severity: Severity::Warning,
                    byte_range: scope.byte_range.clone(),
                    line: scope.line,
                    column: 1,
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

    // -- DEFNU: local function never called ---------------------------------

    #[test]
    fn defnu_fires_on_uncalled_local_function() {
        let diags = lint_file(
            &*engine(),
            "function main()\n    disp(1);\nend\n\nfunction helper()\n    disp(2);\nend\n",
        );
        assert!(has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_ok_when_local_function_called() {
        let diags = lint_file(
            &*engine(),
            "function main()\n    helper();\nend\n\nfunction helper()\n    disp(2);\nend\n",
        );
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_ok_when_function_handle_taken() {
        // Probe: a function handle counts as a use.
        let diags = lint_file(
            &*engine(),
            "function main()\n    h = @helper;\n    h();\nend\n\nfunction helper()\n    disp(2);\nend\n",
        );
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_silent_when_file_calls_feval() {
        // Probe: `dead` is not reported once the file calls feval, even though
        // the feval argument is a handle or a variable.
        let diags = lint_file(
            &*engine(),
            "function r = main(name)\n    r = feval(name);\nend\n\nfunction r = dead()\n    r = 2;\nend\n",
        );
        assert!(!has_id(&diags, "DEFNU"), "got: {diags:?}");
    }

    #[test]
    fn defnu_silent_when_file_calls_eval_or_str2func() {
        // Probe: the same holds for eval and str2func.
        for call in ["eval('x = 1;');", "g = str2func('helper');"] {
            let src =
                format!("function main()\n    {call}\nend\n\nfunction dead()\n    disp(3);\nend\n");
            let diags = lint_file(&*engine(), &src);
            assert!(!has_id(&diags, "DEFNU"), "{call}: got: {diags:?}");
        }
    }
}
