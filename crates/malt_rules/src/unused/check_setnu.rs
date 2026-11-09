//! SETNU check: variable is assigned but its value is never subsequently used.

use super::*;

impl UnusedEngine {
    /// Run SETNU check: a variable that a function assigns and that a nested
    /// function also refers to, but that is never read anywhere in either.
    ///
    /// Probes against MATLAB R2026b: SETNU fires only for such shared
    /// variables, once per variable, at the first assignment in the enclosing
    /// function. It did not fire for partial assignments (`A(2) = 5`,
    /// `s.f = 1`) or loop variables in functions without nested functions;
    /// those are STRNU, ASGLU or NASGU. A variable only the nested function
    /// assigns is NASGU's.
    pub(crate) fn check_setnu(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("SETNU") {
            return;
        }

        for (index, scope) in table.scopes.iter().enumerate() {
            for def in &scope.defs {
                if def.kind != DefKind::Assignment || def.partial {
                    continue;
                }
                let name = &def.name;
                if self.should_ignore_name(name) {
                    continue;
                }
                // Inputs, outputs, globals and persistents are used outside.
                if scope
                    .defs
                    .iter()
                    .any(|d| d.name == *name && d.kind != DefKind::Assignment)
                {
                    continue;
                }
                // Report once per variable, at its first assignment in this scope.
                let first = scope
                    .defs
                    .iter()
                    .filter(|d| d.kind == DefKind::Assignment && d.name == *name)
                    .min_by_key(|d| d.byte_range.start);
                if first.is_none_or(|d| d.byte_range != def.byte_range) {
                    continue;
                }

                let shared = scope
                    .children
                    .iter()
                    .any(|&c| nested_refers_to(table, c, name));
                if shared && !is_read_below(table, index, name) {
                    diagnostics.push(Diagnostic {
                        rule_id: "SETNU",
                        message: "Variable is set, but might be unused.".to_string(),
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
}

/// `true` if the nested function at `index`, or a function nested in it,
/// refers to `name` (assigns or reads it).
fn nested_refers_to(table: &SymbolTable, index: usize, name: &str) -> bool {
    let scope = &table.scopes[index];
    if scope.kind != ScopeKind::NestedFunction {
        return false;
    }
    scope.defs.iter().any(|d| d.name == name)
        || scope.uses.iter().any(|u| u.name == name)
        || scope
            .children
            .iter()
            .any(|&c| nested_refers_to(table, c, name))
}

/// `true` if `name` is read in the scope at `index` or in any scope below it.
fn is_read_below(table: &SymbolTable, index: usize, name: &str) -> bool {
    let scope = &table.scopes[index];
    scope.uses.iter().any(|u| u.name == name)
        || scope
            .children
            .iter()
            .any(|&c| is_read_below(table, c, name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    // -- SETNU: shared variable set but never read --------------------------

    #[test]
    fn setnu_fires_on_shared_variable_never_read() {
        // Probe: `n` is set here and in the nested function, never read.
        let diags = lint_file(
            &*engine(),
            "function foo()\n    n = 0;\n    inner();\n    function inner()\n        n = 1;\n    end\nend\n",
        );
        let lines: Vec<usize> = diags
            .iter()
            .filter(|d| d.rule_id == "SETNU")
            .map(|d| d.line)
            .collect();
        assert_eq!(lines, vec![2], "got: {diags:?}");
    }

    #[test]
    fn setnu_reports_first_assignment_once() {
        // Probe: one report per variable, at the first assignment.
        let diags = lint_file(
            &*engine(),
            "function foo()\n    n = 0;\n    n = 2;\n    inner();\n    function inner()\n        n = 1;\n    end\nend\n",
        );
        let n = diags.iter().filter(|d| d.rule_id == "SETNU").count();
        assert_eq!(n, 1, "got: {diags:?}");
    }

    #[test]
    fn setnu_ok_when_nested_function_reads() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    n = 0;\n    inner();\n    function inner()\n        disp(n);\n    end\nend\n",
        );
        assert!(!has_id(&diags, "SETNU"), "got: {diags:?}");
    }

    #[test]
    fn setnu_ok_when_parent_reads() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    n = 0;\n    inner();\n    disp(n);\n    function inner()\n        n = 1;\n    end\nend\n",
        );
        assert!(!has_id(&diags, "SETNU"), "got: {diags:?}");
    }

    #[test]
    fn setnu_ignores_partial_assignment() {
        // Probe: MATLAB does not report partial assignments as SETNU.
        let diags = lint_file(
            &*engine(),
            "function foo()\n    A(2) = 5;\n    A(3) = 6;\nend\n",
        );
        assert!(!has_id(&diags, "SETNU"), "got: {diags:?}");
    }

    #[test]
    fn setnu_ok_without_nested_function() {
        // Probe: an unread variable in a plain function is NASGU's.
        let diags = lint_file(&*engine(), "function foo()\n    x = 5;\nend\n");
        assert!(!has_id(&diags, "SETNU"), "got: {diags:?}");
    }

    #[test]
    fn setnu_ok_when_value_used() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    x = 1;\n    disp(x);\nend\n",
        );
        assert!(!has_id(&diags, "SETNU"), "got: {diags:?}");
    }

    #[test]
    fn setnu_ok_on_output_arg() {
        let diags = lint_file(&*engine(), "function y = foo()\n    y = 1;\nend\n");
        assert!(!has_id(&diags, "SETNU"), "got: {diags:?}");
    }
}
