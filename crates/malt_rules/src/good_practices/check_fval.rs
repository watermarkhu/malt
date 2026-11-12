use super::*;
use crate::analysis::reaching::{reaching, Binding};

impl GoodPracticesEngine {
    /// FVAL: `feval` with a literal function name or handle, which can be a
    /// direct call: `feval('sin', x)` is `sin(x)`.
    pub(crate) fn check_fval(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("FVAL") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if node.kind() == "function_call" && is_literal_feval(node, source) {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "FVAL",
                    message: "Calling functions using 'feval' is usually not necessary. Call the function directly instead."
                        .to_string(),
                    severity: Severity::Warning,
                    byte_range: node.byte_range(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }
        diagnostics.sort_by_key(|d| d.byte_range.start);
        diagnostics
    }
}

/// `feval(<char literal | @name>, ...)`: the callee is known statically.
fn is_literal_feval(call: Node, source: &str) -> bool {
    let is_feval = call
        .child_by_field_name("name")
        .is_some_and(|n| &source[n.byte_range()] == "feval");
    if !is_feval {
        return false;
    }
    let mut cursor = call.walk();
    let first = call
        .children(&mut cursor)
        .find(|c| c.kind() == "arguments")
        .and_then(|args| args.named_child(0));
    let Some(first) = first else {
        return false;
    };
    match first.kind() {
        "string" | "handle_operator" => true,
        // Probed: `f = @sin; feval(f, 2)` and `f = @(x) ...; feval(f, 2)` are FVAL;
        // a function parameter is not.
        "identifier" => {
            let name = &source[first.byte_range()];
            matches!(
                reaching(first, source, name).binding,
                Binding::Value(v) if matches!(v.kind(), "handle_operator" | "lambda")
            )
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn fval(src: &str) -> bool {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        has_id(&lint_file(&*e, src), "FVAL")
    }

    #[test]
    fn fval_fires_on_literal_callee() {
        assert!(fval("r = feval('sin', 1);\n"));
        assert!(fval("r = feval(@sin, 1);\n"));
    }

    #[test]
    fn fval_ignores_dynamic_callee_and_unused_outputs() {
        assert!(!fval("r = feval(fh, 1);\n"));
        assert!(!fval("function r = f()\nend\n"));
    }
}

#[cfg(test)]
mod handle_variable_tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};

    fn fires(src: &str) -> bool {
        let e = GoodPracticesEngine::from_config(
            &malt_core::Config::from_toml("[lint]\npreset = \"all\"\n").unwrap(),
        );
        has_id(&lint_file(&*e, src), "FVAL")
    }

    #[test]
    fn fval_fires_on_variable_bound_to_handle() {
        // Probed: `f = @sin; feval(f, 2)` is FVAL.
        assert!(fires("f = @sin;\ny = feval(f, 2) + 1;\n"));
    }

    #[test]
    fn fval_silent_on_function_parameter_handle() {
        // Probed silent: a handle passed in as a parameter.
        assert!(!fires(
            "function a = g(f)\nfor k = 1:3\n    a = feval(f, k) + 2;\nend\nend\n"
        ));
    }
}
