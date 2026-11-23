use super::*;
use crate::analysis::callee::{is_shadowed, Visibility};
use crate::analysis::reaching::resolved_values;
use crate::analysis::symbols::SymbolTable;
use malt_core::Fix;

impl GoodPracticesEngine {
    /// FVAL: `feval` with a literal function name or handle, which can be a
    /// direct call: `feval('sin', x)` is `sin(x)`.
    pub(crate) fn check_fval(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("FVAL") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let table = SymbolTable::shared(|| tree.root_node(), source);
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
                    fix: fval_fix(node, source, &table),
                });
            }
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }
        diagnostics.sort_by_key(|d| d.byte_range.start);
        diagnostics
    }
}

/// The FVAL fix: `feval('f', x)` and `feval(@f, x)` become `f(x)`; the callee
/// name replaces `feval` and the first argument goes with the comma and the
/// blanks (or continuation) up to the next argument. This is the text MATLAB's
/// own fix writes (probed: `feval( @sin , 1 )` gives `sin( 1 )`, `feval(@sin)`
/// gives `sin()`, `feval(@sin, ...\n 1)` gives `sin(1)`, `@pkg.fn` and
/// `"a.b"` give `pkg.fn(...)` and `a.b(...)`; a variable or an anonymous
/// function as the first argument gets no fix).
///
/// Safe because the call is the same call: `feval` resolves the name when it
/// runs, as the direct call does, the arguments are evaluated in the same order
/// and passed unchanged (`varargin{:}` included), the number of outputs is
/// whatever the context asks for either way, and the callee's `inputname`
/// sees the same names (probed). What could differ is only a name that means
/// something else as a direct call: a variable called `f` (indexed instead of
/// called) or a keyword. The fix is not offered when the name, or the first
/// segment of a dotted name, is a variable or a function defined in the file,
/// when `feval` itself is shadowed, or when the written name is not a plain
/// (dotted) identifier.
fn fval_fix(call: Node, source: &str, table: &SymbolTable) -> Option<Fix> {
    let name_node = call.child_by_field_name("name")?;
    let mut cursor = call.walk();
    let args = call
        .children(&mut cursor)
        .find(|c| c.kind() == "arguments")?;
    let first = args.named_child(0)?;
    let text = &source[first.byte_range()];
    let callee = match first.kind() {
        "string" => {
            let inner = text.get(1..text.len().checked_sub(1)?)?;
            if !(text.starts_with('\'') && text.ends_with('\'')
                || text.starts_with('"') && text.ends_with('"'))
            {
                return None;
            }
            inner
        }
        "handle_operator" => text.strip_prefix('@')?,
        _ => return None,
    };
    if !is_plain_dotted_name(callee) || is_keyword(callee) {
        return None;
    }
    let head = callee.split('.').next()?;
    if is_shadowed(call, head, table, Visibility::WithScript)
        || is_shadowed(call, "feval", table, Visibility::WithScript)
    {
        return None;
    }
    // Delete from the first argument to the start of the next one, or just the
    // first argument when it is alone.
    let delete_end = args
        .named_child(1)
        .map_or(first.end_byte(), |second| second.start_byte());
    Some(Fix::with_additional(
        name_node.byte_range(),
        callee.to_string(),
        vec![Fix::new(first.start_byte()..delete_end, "")],
    ))
}

/// `name` or `pkg.name` with identifier segments only.
fn is_plain_dotted_name(text: &str) -> bool {
    !text.is_empty()
        && text.split('.').all(|seg| {
            let mut chars = seg.chars();
            chars.next().is_some_and(|c| c.is_ascii_alphabetic())
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

/// MATLAB's reserved words, which cannot be called.
fn is_keyword(text: &str) -> bool {
    matches!(
        text,
        "break"
            | "case"
            | "catch"
            | "classdef"
            | "continue"
            | "else"
            | "elseif"
            | "end"
            | "for"
            | "function"
            | "global"
            | "if"
            | "otherwise"
            | "parfor"
            | "persistent"
            | "return"
            | "spmd"
            | "switch"
            | "try"
            | "while"
            | "arguments"
            | "enumeration"
            | "events"
            | "methods"
            | "properties"
    )
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
        "identifier" if depth < 4 => resolved_values(node, source).is_some_and(|values| {
            values
                .iter()
                .all(|v| matches!(v.kind(), "handle_operator" | "lambda" | "string"))
        }),
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

#[cfg(test)]
mod variable_tests {
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn fval(src: &str) -> bool {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        has_id(&lint_file(&*e, src), "FVAL")
    }

    // Probed: FVAL for a handle or a name that reaches the call through copies (a
    // self-copy included), a literal name, and a handle set in one branch.
    #[test]
    fn fval_follows_copies_and_self_copies() {
        assert!(fval("h = @sin; h = h; y = feval(h, 1);\n"));
        assert!(fval("h = @sin; g = h; y = feval(g, 1);\n"));
        assert!(fval("h = 'sin'; y = feval(h, 1);\n"));
        assert!(fval(
            "h = @sin;\nc = rand > 0.5;\nif c\n    h = @cos;\nend\ny = feval(h, 1);\n"
        ));
    }

    // Probed: silent for a value that is not a function and for a cleared name.
    #[test]
    fn fval_is_silent_for_a_non_function_or_a_cleared_name() {
        assert!(!fval("h = @sin; h = 3; y = feval(h, 1);\n"));
        assert!(!fval("h = @sin; clear h; y = feval(h, 1);\n"));
    }
}
