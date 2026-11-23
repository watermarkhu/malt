use super::*;
use crate::analysis::elementwise::operator_token;

/// True for a binary `^` or `.^` node.
fn is_power(node: Node, source: &str) -> bool {
    if node.kind() != "binary_operator" {
        return false;
    }
    (0..node.child_count())
        .filter_map(|i| node.child(i))
        .any(|c| !c.is_named() && matches!(node_text(c, source), "^" | ".^"))
}

impl GoodPracticesEngine {
    /// UNRPWR: a unary operator combined with a power without parentheses, as in
    /// `2^-a^2` (parsed as `2^((-a)^2)`) or `2^~a^b`.
    pub(crate) fn check_unrpwr(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("UNRPWR") || !is_power(node, source) {
            return Vec::new();
        }
        let unary_left = node
            .child_by_field_name("left")
            .is_some_and(|l| matches!(l.kind(), "unary_operator" | "not_operator"));
        let right_of_power = node.parent().is_some_and(|p| {
            is_power(p, source)
                && p.child_by_field_name("right")
                    .is_some_and(|r| r.id() == node.id())
        });
        // `2^~a^b` may parse as `2^(~(a^b))`: a unary right operand over a power.
        let unary_over_power = node.child_by_field_name("right").is_some_and(|r| {
            matches!(r.kind(), "unary_operator" | "not_operator")
                && r.named_child(0).is_some_and(|o| is_power(o, source))
        });
        if !((unary_left && right_of_power) || unary_over_power) {
            return Vec::new();
        }

        // MATLAB points at the power operator before the sign (`2^-a^2` at the `^` after
        // the 2, `2 .^ -a` at the `.`), not at the sign itself (probed).
        let anchor = if unary_over_power {
            node
        } else {
            node.parent().unwrap_or(node)
        };
        let pos = operator_token(anchor).unwrap_or(anchor).start_position();
        vec![Diagnostic {
            rule_id: "UNRPWR",
            message: "Combining exponentiation operators with unary operators (such as logical negation) without parentheses will not be supported in a future release. Use parentheses to clarify the evaluation order.".to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod dump_tests {
    use super::*;
    fn dump(src: &str) {
        let tree = parse(src);
        fn rec(n: tree_sitter::Node, depth: usize, src: &str) {
            let ind = "  ".repeat(depth);
            let txt = &src[n.start_byte()..n.end_byte()];
            println!("{}{}: {:?}", ind, n.kind(), txt);
            let mut c = n.walk();
            for ch in n.children(&mut c) {
                rec(ch, depth + 1, src);
            }
        }
        println!("===== {} =====", src.escape_debug());
        rec(tree.root_node(), 0, src);
    }
    #[test]
    fn dump_trees() {
        dump("-x^2;\n");
        dump("(-x)^2;\n");
        dump("x = -2^2;\n");
        dump("try\ncatch\nend\n");
        dump("try\ncatch ME\nend\n");
        dump("warning('');\n");
        dump("warning(ME);\n");
        dump("eval('load(filename)');\n");
        dump("x = 1; y = 2;\n");
        dump("x = 1, y = 2;\n");
        dump("[x];\n");
        dump("[x+y];\n");
        dump("[1 2 3];\n");
        dump("s = struct('a', 1);\n");
        dump("a = [1, 2, % comment\n3];\n");
        dump("disp(x);\n");
    }
}
#[cfg(test)]
mod dump2_tests {
    use super::*;
    fn dump(src: &str) {
        let tree = parse(src);
        fn rec(n: tree_sitter::Node, depth: usize, src: &str) {
            let ind = "  ".repeat(depth);
            let txt = &src[n.start_byte()..n.end_byte()];
            println!("{}{}: {:?}", ind, n.kind(), txt);
            let mut c = n.walk();
            for ch in n.children(&mut c) {
                rec(ch, depth + 1, src);
            }
        }
        println!("===== {} =====", src.escape_debug());
        rec(tree.root_node(), 0, src);
    }
    #[test]
    fn dump_trees() {
        dump("disp a disp b\n");
        dump("disp a  disp b\n");
        dump("x = 1  y = 2\n");
        dump("x = 1 y = 2\n");
        dump("if x\n  a = 1  b = 2\nend\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    /// Run UNRPWR on every node of `src`.
    fn unrpwr_diags(src: &str) -> Vec<Diagnostic> {
        let tree = parse(src);
        let eng = engine();
        let mut out = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            out.extend(eng.check_unrpwr(n, src));
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        out
    }

    #[test]
    fn unrpwr_fires_on_unary_inside_chained_power() {
        // MATLAB reports UNRPWR for these (probe): 2^-a^2, 2^~a^b, a.^-b.^c, 2^-3^2.
        for src in [
            "y = 2^-a^2;\n",
            "y = 2^~a^b;\n",
            "y = a.^-b.^c;\n",
            "y = 2^-3^2;\n",
        ] {
            let diags = unrpwr_diags(src);
            assert!(has_id(&diags, "UNRPWR"), "{src}: got {diags:?}");
        }
    }

    #[test]
    fn unrpwr_silent_without_chained_power() {
        // MATLAB does not report these (probe): a single power with a unary operand,
        // a leading unary operand that is not the right operand of a power, and a
        // left-chained `~a^b^c` (XPORD only).
        for src in ["y = 2^-a;\n", "y = -a^-b;\n", "y = ~a^b^c;\n"] {
            let diags = unrpwr_diags(src);
            assert!(!has_id(&diags, "UNRPWR"), "{src}: got {diags:?}");
        }
    }
}

#[cfg(test)]
mod column_tests {
    use super::super::engine;
    use crate::test_util::lint_nodes;

    #[test]
    fn unrpwr_column_is_the_power_operator_before_the_sign() {
        // Probed: MATLAB reports UNRPWR at the `^` (or `.^`) before the sign: column 6 for
        // `y = 2^-a^2;`, 7 for `y = 3 ^ -a ^ 2;` and `y = 2 .^ -a .^ 2;`, not at the sign.
        for (src, column) in [
            ("a = 2;\ny = 2^-a^2;\n", 6),
            ("a = 2;\ny = 3 ^ -a ^ 2;\n", 7),
            ("a = 2;\ny = 2 .^ -a .^ 2;\n", 7),
            ("a = 2;\nb = 3;\ny = 2^~a^b;\n", 6),
        ] {
            let diags = lint_nodes(&engine(), src);
            let found: Vec<usize> = diags
                .iter()
                .filter(|d| d.rule_id == "UNRPWR")
                .map(|d| d.column)
                .collect();
            assert_eq!(found, vec![column], "{src}: got: {diags:?}");
        }
    }
}
