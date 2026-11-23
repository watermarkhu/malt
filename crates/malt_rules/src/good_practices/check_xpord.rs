//! XPORD: chained exponents without parentheses. Probed: `2^3^2`, `a.^b.^c`,
//! `a^b^c^d`, `-a^b^c` and `~a^b^c` fire once per chain, at its first operand
//! (for `-a^b^c` and `~a^b^c` that is the `a`). Silent: `a^(b^c)`, `(a^b)^c`,
//! `a^b*c^b`, `a^b`, `a^-b^c` and `2^-a^2` (those report UNRPWR instead).
//! Tree-sitter nests a chain to the right, so the check reports the outermost
//! link of each chain.

use super::*;

/// True for a binary `^` or `.^` node.
fn is_power(node: Node, source: &str) -> bool {
    if node.kind() != "binary_operator" {
        return false;
    }
    (0..node.child_count())
        .filter_map(|i| node.child(i))
        .any(|c| !c.is_named() && matches!(node_text(c, source), "^" | ".^"))
}

/// True for a unary (`-a`, `~a`) operator node.
fn is_unary(node: Node) -> bool {
    matches!(node.kind(), "unary_operator" | "not_operator")
}

impl GoodPracticesEngine {
    /// XPORD: a power chain without parentheses, reported once at its first operand.
    pub(crate) fn check_xpord(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("XPORD") || !is_power(node, source) {
            return Vec::new();
        }
        // Tree-sitter nests a chain to the right (`a^b^c` is `a^(b^c)`), so the
        // inner links are right children of a power and the outermost one reports.
        let inner_link = node.parent().is_some_and(|p| {
            is_power(p, source)
                && p.child_by_field_name("right")
                    .is_some_and(|r| r.id() == node.id())
        });
        if inner_link {
            return Vec::new();
        }
        let Some(right) = node
            .child_by_field_name("right")
            .filter(|r| is_power(*r, source))
        else {
            return Vec::new();
        };
        // `a^-b^c` and `2^-a^2` put a unary operator under the inner power; MATLAB
        // reports those as UNRPWR instead.
        if right.child_by_field_name("left").is_some_and(is_unary) {
            return Vec::new();
        }

        let start = match node.child_by_field_name("left") {
            // `-a^b^c` and `~a^b^c` parse as `(~a) ^ (b^c)`: the operand starts the chain.
            Some(l) if is_unary(l) => l.named_child(0),
            Some(l) => Some(l),
            None => None,
        };
        let Some(start) = start else {
            return Vec::new();
        };

        let pos = start.start_position();
        vec![Diagnostic {
            rule_id: "XPORD",
            message: "Use parentheses to clarify the evaluation order of chained exponents."
                .to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    /// Run XPORD on every node of `src`.
    fn xpord_diags(src: &str) -> Vec<Diagnostic> {
        let tree = parse(src);
        let eng = engine();
        let mut out = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            out.extend(eng.check_xpord(n, src));
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        out
    }

    #[test]
    fn xpord_fires_on_chained_power_at_first_operand() {
        let diags = xpord_diags("x = 2^3^2;\n");
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!((diags[0].line, diags[0].column), (1, 5));

        let diags = xpord_diags("z = -a^b^c;\n");
        assert_eq!((diags[0].line, diags[0].column), (1, 6));

        let diags = xpord_diags("z = ~a^b^c;\n");
        assert!(has_id(&diags, "XPORD"), "got: {diags:?}");
        assert_eq!((diags[0].line, diags[0].column), (1, 6));

        assert!(has_id(&xpord_diags("y = a.^b.^c;\n"), "XPORD"));
    }

    #[test]
    fn xpord_reports_one_diagnostic_per_chain() {
        let diags = xpord_diags("y = a^b^c^d;\n");
        assert_eq!(diags.len(), 1, "got: {diags:?}");
    }

    #[test]
    fn xpord_silent_with_parentheses_or_no_chain() {
        for src in [
            "y = a^(b^c);\n",
            "y = (a^b)^c;\n",
            "w = a^b*c^b;\n",
            "v = a^b;\n",
            "y = a^-b^c;\n",
            "y = 2^-a^2;\n",
        ] {
            assert!(!has_id(&xpord_diags(src), "XPORD"), "{src}");
        }
    }
}
