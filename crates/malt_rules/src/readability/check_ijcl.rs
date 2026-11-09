//! # IJCL: `i`/`j` used as the imaginary unit
//!
//! `check_ijcl` flags a read of the identifier `i` or `j` that is not a
//! variable in scope, i.e. one that relies on the built-in imaginary unit.
//! `1i` / `1j` cannot be shadowed by a variable.

use super::*;

/// Parent node kinds in which a bare `i`/`j` is an expression operand.
fn is_operand_position(node: tree_sitter::Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        "binary_operator"
        | "unary_operator"
        | "postfix_operator"
        | "comparison_operator"
        | "boolean_operator"
        | "row"
        | "matrix"
        | "cell"
        | "parenthesized_expression"
        | "range"
        | "arguments" => true,
        "assignment" => parent
            .child_by_field_name("right")
            .is_some_and(|r| r.id() == node.id()),
        _ => false,
    }
}

impl ReadabilityEngine {
    /// IJCL: file-level walk over `identifier` nodes named `i` or `j`.
    pub(crate) fn check_ijcl(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("IJCL") {
            return;
        }
        let table = SymbolTable::build(tree, source);
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
            if node.kind() != "identifier" {
                continue;
            }
            let name = &source[node.start_byte()..node.end_byte()];
            if (name != "i" && name != "j")
                || !is_operand_position(node)
                || is_variable_in_scope(name, node.start_byte(), &table)
            {
                continue;
            }
            results.push(self.diag(
                "IJCL",
                "For improved robustness, consider replacing i and j by 1i.",
                node,
                Some(Fix::new(
                    node.start_byte()..node.end_byte(),
                    format!("1{name}"),
                )),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    #[test]
    fn ijcl_imaginary_unit_use_fires() {
        for src in ["z = 3 + 2*i;\n", "z = [1 j];\n", "z = exp(j*pi);\n"] {
            assert!(has_id(&lint_file(&*engine(), src), "IJCL"), "{src}");
        }
    }

    #[test]
    fn ijcl_fix_uses_the_literal() {
        let src = "z = 3 + 2*i;\n";
        let diags = lint_file(&*engine(), src);
        let d = diags.iter().find(|d| d.rule_id == "IJCL").expect("IJCL");
        let fix = d.fix.as_ref().expect("fix");
        assert_eq!(&source_after(src, fix), "z = 3 + 2*1i;\n");
    }

    fn source_after(src: &str, fix: &Fix) -> String {
        let mut out = src.to_string();
        out.replace_range(fix.byte_range.clone(), &fix.replacement);
        out
    }

    #[test]
    fn ijcl_assignment_and_loop_variable_do_not_fire() {
        for src in [
            "i = 5;\n",
            "j = zeros(3);\n",
            "for i = 1:3\n    y(i) = i;\nend\n",
            "i = 2;\nz = 3*i;\n",
            "function f(i)\n    z = 3*i;\nend\n",
            "k = 5;\n",
        ] {
            assert!(!has_id(&lint_file(&*engine(), src), "IJCL"), "{src}");
        }
    }
}
