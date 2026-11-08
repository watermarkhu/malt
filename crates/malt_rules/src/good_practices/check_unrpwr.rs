use super::*;

impl GoodPracticesEngine {
    /// UNRPWR: Power of negative base may produce complex result.
    pub(crate) fn check_unrpwr(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("UNRPWR") || node.kind() != "binary_operator" {
            return Vec::new();
        }

        let text = node_text(node, source);
        if !text.contains('^') {
            return Vec::new();
        }

        // Check if the left operand is a negative literal or unary negation.
        let left = node.child_by_field_name("left").or_else(|| node.child(0));
        let has_negative_base = left
            .map(|l| {
                if l.kind() == "unary_operator" {
                    let lt = node_text(l, source);
                    return lt.starts_with('-');
                }
                if l.kind() == "number" {
                    let lt = node_text(l, source);
                    return lt.starts_with('-');
                }
                false
            })
            .unwrap_or(false);

        if !has_negative_base {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "UNRPWR",
            message: "Consider using parentheses to explicitly specify operator precedence."
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
