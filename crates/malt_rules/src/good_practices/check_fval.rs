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
    callee_is_literal(first, source, 0)
}

/// Whether a `feval` callee is a known function: a character vector, a handle or
/// an anonymous function, written literally or bound to a variable. Probed: FVAL
/// for `f = @sin; feval(f, 2)`, `f = 'sin'; feval(f, 1)`, `k = h` chains and a
/// lambda literal; a function parameter is not.
fn callee_is_literal(node: Node, source: &str, depth: usize) -> bool {
    match node.kind() {
        "string" | "handle_operator" | "lambda" => true,
        "identifier" if depth < 4 => {
            let name = &source[node.byte_range()];
            match reaching(node, source, name).binding {
                Binding::Value(v) if v.kind() == "identifier" => {
                    callee_is_literal(v, source, depth + 1)
                }
                Binding::Value(v) => matches!(v.kind(), "handle_operator" | "lambda" | "string"),
                _ => false,
            }
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
    fn fval_fires_on_probed_callee_forms() {
        // Probed FVAL: a lambda literal, a character variable, a handle copied through
        // another variable.
        assert!(fires("r = feval(@(x) x+1, 2);\n"));
        assert!(fires("h = 'sin';\nh = 'cos';\nr = feval(h, 1);\n"));
        assert!(fires("h = @sin;\nk = h;\nr = feval(k, 2);\n"));
    }

    #[test]
    fn fval_silent_on_function_parameter_handle() {
        // Probed silent: a handle passed in as a parameter.
        assert!(!fires(
            "function a = g(f)\nfor k = 1:3\n    a = feval(f, k) + 2;\nend\nend\n"
        ));
    }
}
