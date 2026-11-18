use super::check_comnot::logical_call_literal;
use super::*;

impl GoodPracticesEngine {
    /// COMPNOP: `f(...) == 1` or `f(...) ~= 0` for a logical call `f` (`any`,
    /// `all`, `isequal`, `isreal`) simplifies to `f(...)`.
    pub(crate) fn check_compnop(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("COMPNOP") {
            return Vec::new();
        }
        let Some((call_node, op, value)) = logical_call_literal(node, source) else {
            return Vec::new();
        };
        let is_compnop = (op == "==" && value == 1.0) || (op == "~=" && value == 0.0);
        if !is_compnop {
            return Vec::new();
        }

        let call_text = node_text(call_node, source).to_string();
        let func_name = get_function_call_name(call_node, source).unwrap_or("function");
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "COMPNOP",
            message: format!(
                "This logical comparison simplifies to {func_name}(...). Did you mean to use {op} to evaluate function argument: {func_name}(...{op}...)?"
            ),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            // `f(x) == 1` is `f(x)` only when f returns a logical.
            fix: Some(Fix::new(node.start_byte()..node.end_byte(), call_text).unsafe_fix()),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// COMPNOP diagnostics for every comparison in `source`.
    fn compnop(source: &str) -> Vec<Diagnostic> {
        let tree = parse(source);
        let eng = engine();
        let mut out = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            out.extend(eng.check_compnop(n, source));
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        out
    }

    #[test]
    fn test_compnop_fires_on_logical_call_eq_one_or_neq_zero() {
        // Probed: `any(x) == 1`, `any(x) ~= 0` and `isequal(x, s) ~= 0`.
        for (source, replacement) in [
            ("a = any(x) == 1;\n", "any(x)"),
            ("a = any(x) ~= 0;\n", "any(x)"),
            ("a = isequal(x, s) ~= 0;\n", "isequal(x, s)"),
        ] {
            let diags = compnop(source);
            assert_eq!(diags.len(), 1, "{source}");
            assert_eq!(diags[0].rule_id, "COMPNOP");
            assert_eq!(diags[0].fix.as_ref().unwrap().replacement, replacement);
        }
    }

    #[test]
    fn test_compnop_message_matches_matlab() {
        let diags = compnop("if any(x) ~= 0\nend\n");
        assert_eq!(
            diags[0].message,
            "This logical comparison simplifies to any(...). Did you mean to use ~= to evaluate function argument: any(...~=...)?"
        );
    }

    #[test]
    fn test_compnop_silent_on_zero_eq_and_logical_literals() {
        // `== 0` and `~= 1` are COMPNOT; `true`/`false` literals and
        // non-logical functions are silent (probed).
        for source in [
            "a = any(x) == 0;\n",
            "a = any(x) ~= 1;\n",
            "a = any(x) == true;\n",
            "a = isa(x, 'double') == true;\n",
            "a = isempty(x) == 1;\n",
        ] {
            assert!(compnop(source).is_empty(), "{source}");
        }
    }
}
