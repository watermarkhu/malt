//! BDSCA1: an `if` or `while` condition that is itself a non-scalar literal,
//! such as `if [1 0]`, `while [1 2 3]`, `if []` or `if 1:3`. Probed: all fire at
//! the start of the literal. Silent: a parenthesised literal, `[1]`, a variable
//! condition, `x = []; if x`, `if ~[]`, `if [] & true` and `if zeros(0, 1)`.

use super::check_iseq::{condition_of, single_row_len};
use super::*;

const BDSCA1_MSG: &str = "Unexpected use of non-scalar value in a context which expects a scalar value. Use 'all' or 'any' to convert this value into a scalar value.";

impl GoodPracticesEngine {
    /// BDSCA1: an `if` or `while` condition that is a range or a matrix literal
    /// with zero or two or more elements (`[]` is probed as non-scalar).
    pub(crate) fn check_bdsca1(&self, node: Node, _source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("BDSCA1") {
            return Vec::new();
        }
        // Probed: an `elseif` condition is reported like an `if` condition.
        let condition = if node.kind() == "elseif_clause" {
            (0..node.named_child_count())
                .filter_map(|i| node.named_child(i))
                .find(|c| !matches!(c.kind(), "block" | "comment"))
        } else {
            condition_of(node)
        };
        let Some(condition) = condition else {
            return Vec::new();
        };
        let non_scalar = match condition.kind() {
            "range" => true,
            "matrix" => single_row_len(condition).is_some_and(|n| n != 1),
            _ => false,
        };
        if !non_scalar {
            return Vec::new();
        }

        let pos = condition.start_position();
        vec![Diagnostic {
            rule_id: "BDSCA1",
            message: BDSCA1_MSG.to_string(),
            severity: Severity::Warning,
            byte_range: condition.start_byte()..condition.end_byte(),
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

    /// Run BDSCA1 on every node of `src`.
    fn bdsca1_diags(src: &str) -> Vec<Diagnostic> {
        let tree = parse(src);
        let eng = engine();
        let mut out = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            out.extend(eng.check_bdsca1(n, src));
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        out
    }

    #[test]
    fn bdsca1_fires_on_matrix_and_range_conditions() {
        let diags = bdsca1_diags("if [1 0]\n    disp(1);\nend\n");
        assert!(has_id(&diags, "BDSCA1"), "got: {diags:?}");
        assert_eq!((diags[0].line, diags[0].column), (1, 4));

        let diags = bdsca1_diags("while [1 2 3]\n    break;\nend\n");
        assert_eq!((diags[0].line, diags[0].column), (1, 7));

        assert!(has_id(&bdsca1_diags("if 1:3\nend\n"), "BDSCA1"));

        // Probed: an empty matrix literal is non-scalar in `if` and `while`.
        let diags = bdsca1_diags("if []\n    disp(1);\nend\n");
        assert_eq!((diags[0].line, diags[0].column), (1, 4));
        let diags = bdsca1_diags("while []\n    disp(1);\nend\n");
        assert_eq!((diags[0].line, diags[0].column), (1, 7));
    }

    #[test]
    fn bdsca1_silent_on_scalar_parenthesised_and_variable_conditions() {
        for src in [
            "if [1]\nend\n",
            "if ([1 2 3])\nend\n",
            "a = [1 2 3];\nif a\nend\n",
            "if [1 2] && true\nend\n",
            "if rand(3)\nend\n",
            "if ~[]\nend\n",
            "if [] & true\nend\n",
            "x = [];\nif x\nend\n",
            "if zeros(0, 1)\nend\n",
        ] {
            assert!(!has_id(&bdsca1_diags(src), "BDSCA1"), "{src}");
        }
    }
}

#[cfg(test)]
mod elseif_tests {
    use super::*;
    use crate::test_util::has_id;

    #[test]
    fn bdsca1_fires_on_an_elseif_condition() {
        // Probed: `elseif [1 0]` and `elseif [1 2 3]` are reported like `if`.
        let source = "if x\nelseif [1 0]\n    disp(1);\nend\n";
        let tree = parse(source);
        let mut out = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            out.extend(engine().check_bdsca1(n, source));
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        assert!(has_id(&out, "BDSCA1"), "got: {out:?}");
        assert_eq!((out[0].line, out[0].column), (2, 8));
    }
}
