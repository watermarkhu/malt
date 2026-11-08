//! ASGLU check: an element of `[a, b] = f()` that is never used.

use super::*;
use malt_core::Fix;

impl UnusedEngine {
    /// Run ASGLU check: an output of a multi-value call that is never read can
    /// be replaced by `~`, which skips it. Equivalent, so the fix is safe.
    pub(crate) fn check_asglu(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("ASGLU") {
            return;
        }

        for scope in &table.scopes {
            for def in &scope.defs {
                if !def.multi_output || def.kind != DefKind::Assignment {
                    continue;
                }
                let name = &def.name;
                if self.should_ignore_name(name) || scope.is_used(name) {
                    continue;
                }
                // Outputs of the enclosing function are used by the caller.
                if scope
                    .defs
                    .iter()
                    .any(|d| d.kind == DefKind::OutputArg && d.name == *name)
                {
                    continue;
                }
                diagnostics.push(Diagnostic {
                    rule_id: "ASGLU",
                    message: format!(
                        "Value assigned to variable '{name}' might be unused. Consider replacing the variable with ~ instead."
                    ),
                    severity: Severity::Warning,
                    byte_range: def.byte_range.clone(),
                    line: def.line,
                    column: def.column,
                    fix: Some(Fix::new(def.byte_range.clone(), "~")),
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

    #[test]
    fn asglu_fires_on_unused_output_element() {
        let src = "function foo()\n    [a, b] = size(1);\n    disp(a);\nend\n";
        let diags = lint_file(&*engine(), src);
        let d: Vec<_> = diags.iter().filter(|d| d.rule_id == "ASGLU").collect();
        assert_eq!(d.len(), 1, "got: {diags:?}");
        assert_eq!(&src[d[0].byte_range.clone()], "b");
        assert_eq!(d[0].fix.as_ref().unwrap().replacement, "~");
        assert!(!has_id(&diags, "NASGU"), "disjoint from NASGU: {diags:?}");
    }

    #[test]
    fn asglu_ok_when_every_element_is_used() {
        let src = "function foo()\n    [a, b] = size(1);\n    disp(a + b);\nend\n";
        assert!(!has_id(&lint_file(&*engine(), src), "ASGLU"));
    }

    #[test]
    fn asglu_ignores_plain_overwrites() {
        let src = "function foo()\n    x = 1;\n    x = 2;\nend\n";
        assert!(!has_id(&lint_file(&*engine(), src), "ASGLU"));
    }
}
