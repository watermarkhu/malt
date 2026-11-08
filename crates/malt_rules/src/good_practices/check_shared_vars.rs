//! Shared-variable confusion check (SHVAU).
//!
//! When a name is *used* inside a nested function and also *assigned* in the
//! enclosing function *after* the nested function definition, MATLAB cannot
//! tell whether the nested function's use refers to the shared variable or to
//! a separate local variable. The check fires on the nested function's use of
//! the name, pointing at both lines.
//!
//! Conservative rule: fire only when a `NestedFunction` scope exists in the
//! symbol table, the name is used there, and the parent scope assigns the same
//! name at a line strictly after the nested function's start line. Assignments
//! made *before* the nested function definition are unambiguous (the name is a
//! regular shared variable) and do not fire.

use super::*;
use crate::analysis::symbols::{DefKind, ScopeKind};

impl GoodPracticesEngine {
    /// SHVAU: a name assigned in the parent after a nested function definition
    /// is also used inside that nested function — ambiguous shared variable.
    pub(crate) fn check_shared_vars(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Diagnostic> {
        if !self.is_check_enabled("SHVAU") {
            return Vec::new();
        }

        let sym = SymbolTable::build(tree, source);
        let mut diagnostics = Vec::new();

        for nested in sym.scopes.iter() {
            if nested.kind != ScopeKind::NestedFunction {
                continue;
            }
            let Some(parent_idx) = nested.parent else {
                continue;
            };
            let nested_start_line = nested.line;
            let parent = &sym.scopes[parent_idx];

            for use_ in &nested.uses {
                // Parent assignment of the same name AFTER the nested function
                // definition makes the reference ambiguous.
                let parent_def = parent
                    .defs
                    .iter()
                    .filter(|d| d.name == use_.name && d.kind == DefKind::Assignment)
                    .filter(|d| d.line > nested_start_line)
                    .min_by_key(|d| d.line);
                let Some(def) = parent_def else {
                    continue;
                };

                diagnostics.push(Diagnostic {
                    rule_id: "SHVAU",
                    message: format!(
                        "Confusing usage of name {} on lines {} and {}. Initialize {} before line \
                         {} to make it a shared variable or rename {} on line {} to disambiguate.",
                        use_.name, def.line, use_.line, use_.name, def.line, use_.name, use_.line
                    ),
                    severity: Severity::Warning,
                    byte_range: use_.byte_range.clone(),
                    line: use_.line,
                    column: use_.column,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shvau_ids(source: &str) -> Vec<&'static str> {
        let tree = parse(source);
        let eng = engine();
        eng.check_shared_vars(&tree, source)
            .iter()
            .map(|d| d.rule_id)
            .collect()
    }

    #[test]
    fn test_shvau_fires_when_parent_assigns_after_nested_function() {
        let source = "\
function outer()
    y = 1;
    function inner()
        disp(x);
    end
    x = 2;
end
";
        let ids = shvau_ids(source);
        assert_eq!(ids.len(), 1, "got: {:?}", shvau_ids(source));

        let tree = parse(source);
        let eng = engine();
        let diag = eng
            .check_shared_vars(&tree, source)
            .into_iter()
            .find(|d| d.rule_id == "SHVAU")
            .unwrap();
        assert!(diag.message.contains("x"), "got: {}", diag.message);
        assert!(
            diag.message.contains("lines 6 and 4"),
            "got: {}",
            diag.message
        );
    }

    #[test]
    fn test_shvau_silent_when_parent_assigns_before_nested_function() {
        let source = "\
function outer()
    x = 1;
    function inner()
        disp(x);
    end
end
";
        assert!(shvau_ids(source).is_empty());
    }

    #[test]
    fn test_shvau_silent_when_no_nested_function() {
        let source = "\
function outer()
    x = 2;
    disp(x);
end
";
        assert!(shvau_ids(source).is_empty());
    }

    #[test]
    fn test_shvau_silent_for_local_function_usage() {
        let source = "\
function outer()
    x = 2;
    inner();
end
function inner()
    disp(x);
end
";
        assert!(shvau_ids(source).is_empty());
    }

    #[test]
    fn test_shvau_respects_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec!["SHVAU".to_string()],
            },
        };
        let source = "\
function outer()
    y = 1;
    function inner()
        disp(x);
    end
    x = 2;
end
";
        let tree = parse(source);
        assert!(eng.check_shared_vars(&tree, source).is_empty());
    }
}
