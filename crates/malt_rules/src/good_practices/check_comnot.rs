use super::*;

/// Functions whose numeric 0/1 comparison MATLAB simplifies (`any(x) == 0` is
/// `~any(x)`). Probed: `any`, `all`, `isequal` and `isreal` are reported;
/// `isempty`, `islogical`, `xor` and `strcmp` are silent, and so is any
/// `true`/`false` literal (`any(x) == true`, `isa(x,'double') ~= true`).
const LOGICAL_CALLS: &[&str] = &["any", "all", "isequal", "isreal"];

/// A `==` or `~=` comparison of a call from [`LOGICAL_CALLS`] with a numeric
/// literal, on either side. Returns the call, the operator and the literal's value.
pub(crate) fn logical_call_literal<'a>(
    node: Node<'a>,
    source: &'a str,
) -> Option<(Node<'a>, String, f64)> {
    if node.kind() != "comparison_operator" {
        return None;
    }
    let op = find_operator_text(node, source);
    if op != "==" && op != "~=" {
        return None;
    }
    let lhs = node.child(0)?;
    let rhs = node.child(2)?;
    let (call, literal) = match (lhs.kind(), rhs.kind()) {
        ("function_call", "number") => (lhs, rhs),
        ("number", "function_call") => (rhs, lhs),
        _ => return None,
    };
    if !LOGICAL_CALLS.contains(&get_function_call_name(call, source)?) {
        return None;
    }
    let value: f64 = node_text(literal, source).trim().parse().ok()?;
    Some((call, op, value))
}

impl GoodPracticesEngine {
    /// COMPNOT: `f(...) == 0` or `f(...) ~= 1` for a logical call `f` (`any`,
    /// `all`, `isequal`, `isreal`) simplifies to `~f(...)`.
    pub(crate) fn check_comnot(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("COMPNOT") {
            return Vec::new();
        }
        let Some((call_node, op, value)) = logical_call_literal(node, source) else {
            return Vec::new();
        };
        let is_comnot = (op == "==" && value == 0.0) || (op == "~=" && value == 1.0);
        if !is_comnot {
            return Vec::new();
        }

        let call_text = node_text(call_node, source).to_string();
        let func_name = get_function_call_name(call_node, source).unwrap_or("function");
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "COMPNOT",
            message: format!(
                "This logical comparison simplifies to ~{func_name}(...). Did you mean to use {op} to evaluate function argument: {func_name}(...{op}...)?"
            ),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            // `f(x) == 0` is `~f(x)` only when f returns a logical.
            fix: Some(
                Fix::new(node.start_byte()..node.end_byte(), format!("~{call_text}")).unsafe_fix(),
            ),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// COMPNOT diagnostics for the first comparison in `source`.
    fn comnot(source: &str) -> Vec<Diagnostic> {
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            out.extend(eng.check_comnot(n, source));
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        out
    }

    #[test]
    fn test_comnot_fires_on_logical_call_eq_zero_or_neq_one() {
        // Probed: `any(x) == 0`, `all(x) ~= 1`, `any(x) == 0.0`, `isreal(x) == 0`
        // and `any(x) == 0 || 1` are reported (the fix is `~call`).
        for (source, replacement) in [
            ("a = any(x) == 0;\n", "~any(x)"),
            ("a = all(x) ~= 1;\n", "~all(x)"),
            ("a = any(x) == 0.0;\n", "~any(x)"),
            ("a = isreal(x) == 0;\n", "~isreal(x)"),
            ("a = 0 == any(x);\n", "~any(x)"),
        ] {
            let diags = comnot(source);
            assert_eq!(diags.len(), 1, "{source}");
            assert_eq!(diags[0].rule_id, "COMPNOT");
            assert_eq!(diags[0].fix.as_ref().unwrap().replacement, replacement);
        }
    }

    #[test]
    fn test_comnot_message_matches_matlab() {
        let diags = comnot("if any(x) ~= 1\nend\n");
        assert_eq!(
            diags[0].message,
            "This logical comparison simplifies to ~any(...). Did you mean to use ~= to evaluate function argument: any(...~=...)?"
        );
    }

    #[test]
    fn test_comnot_silent_on_neq_zero_eq_one_and_logical_literals() {
        // `== 1` and `~= 0` are COMPNOP. Probed: `true`/`false` literals, and
        // `isempty`, `islogical`, `strcmp`, `xor` and `isa` are silent.
        for source in [
            "a = any(x) == 1;\n",
            "a = any(x) ~= 0;\n",
            "a = any(x) == true;\n",
            "a = any(x) == false;\n",
            "a = isa(x, 'double') ~= true;\n",
            "a = isa(x, 'double') == false;\n",
            "a = isempty(x) == 0;\n",
            "a = islogical(x) == 0;\n",
            "a = strcmp(s, 'a') == 0;\n",
            "a = xor(p, q) == 0;\n",
            "a = ~any(x) == 0;\n",
        ] {
            assert!(comnot(source).is_empty(), "{source}");
        }
    }
}
