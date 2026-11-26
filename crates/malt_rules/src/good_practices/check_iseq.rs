//! ISEQ: `==` or `~=` as the whole `if`/`while` condition, with a non-scalar
//! literal operand. MATLAB expects a scalar there, so `isequal` is suggested.
//! Probed: `if a == [1 2]`, `if [1 2] == a`, `if a == []`, `if a == 1:3`,
//! `while a ~= [1 2]`, and the parenthesised `if (a == [1 2])` all fire at the
//! operator. Silent: variable operands, a single-element literal `[2]`, and a
//! comparison outside `if`/`while`.

use super::*;

const ISEQ_MSG: &str = "The context surrounding this expression expects a scalar value. Consider using 'isequal', which returns a scalar value, instead of '==' or '~='.";

/// The condition expression of an `if` or `while` statement, as written.
pub(crate) fn condition_of(stmt: Node) -> Option<Node> {
    if !matches!(stmt.kind(), "if_statement" | "while_statement") {
        return None;
    }
    (0..stmt.named_child_count())
        .filter_map(|i| stmt.named_child(i as u32))
        .find(|c| {
            !matches!(
                c.kind(),
                "block" | "elseif_clause" | "else_clause" | "comment"
            )
        })
}

/// Number of elements of a single-row matrix literal (`[1 2]` is 2, `[]` is 0).
/// Multi-row literals return `None`: MATLAB's behaviour for them is unprobed.
pub(crate) fn single_row_len(matrix: Node) -> Option<usize> {
    let mut cursor = matrix.walk();
    let parts: Vec<Node> = matrix
        .named_children(&mut cursor)
        .filter(|c| c.kind() != "comment")
        .collect();
    match parts.as_slice() {
        [] => Some(0),
        [row] if row.kind() == "row" => Some(
            (0..row.named_child_count())
                .filter_map(|i| row.named_child(i as u32))
                .filter(|c| c.kind() != "comment")
                .count(),
        ),
        _ => None,
    }
}

/// Strips the parentheses around a condition, as in `if (a == [1 2])`.
fn unwrap_parens(mut node: Node) -> Node {
    while node.kind() == "parenthesis" {
        match node.named_child(0) {
            Some(inner) => node = inner,
            None => break,
        }
    }
    node
}

impl GoodPracticesEngine {
    /// ISEQ: an `==` or `~=` comparison that is the whole `if`/`while` condition
    /// and has a non-scalar literal (a range or a matrix other than one element).
    pub(crate) fn check_iseq(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("ISEQ") {
            return Vec::new();
        }
        let Some(condition) = condition_of(node).map(unwrap_parens) else {
            return Vec::new();
        };
        if condition.kind() != "comparison_operator" {
            return Vec::new();
        }
        let Some(op) = (0..condition.child_count())
            .filter_map(|i| condition.child(i))
            .find(|c| !c.is_named() && matches!(node_text(*c, source), "==" | "~="))
        else {
            return Vec::new();
        };
        let has_nonscalar_literal = (0..condition.named_child_count())
            .filter_map(|i| condition.named_child(i as u32))
            .any(|operand| match operand.kind() {
                "range" => true,
                "matrix" => single_row_len(operand).is_some_and(|n| n != 1),
                _ => false,
            });
        if !has_nonscalar_literal {
            return Vec::new();
        }

        let pos = op.start_position();
        vec![Diagnostic {
            rule_id: "ISEQ",
            message: ISEQ_MSG.to_string(),
            severity: Severity::Warning,
            byte_range: op.start_byte()..op.end_byte(),
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

    /// Run ISEQ on every node of `src`.
    fn iseq_diags(src: &str) -> Vec<Diagnostic> {
        let tree = parse(src);
        let eng = engine();
        let mut out = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            out.extend(eng.check_iseq(n, src));
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        out
    }

    #[test]
    fn iseq_fires_on_matrix_literal_operand() {
        let diags = iseq_diags("if a == [1 2]\n    disp(1);\nend\n");
        assert!(has_id(&diags, "ISEQ"), "got: {diags:?}");
        assert_eq!((diags[0].line, diags[0].column), (1, 6));
    }

    #[test]
    fn iseq_fires_when_literal_is_left_operand() {
        let diags = iseq_diags("a = 2;\nif [1 2] == a\nend\n");
        assert_eq!((diags[0].line, diags[0].column), (2, 10));
    }

    #[test]
    fn iseq_fires_on_empty_matrix_range_and_while() {
        for src in [
            "if a == []\nend\n",
            "if a == 1:3\nend\n",
            "while a ~= [1 2]\nend\n",
            "if (a == [1 2])\nend\n",
        ] {
            assert!(has_id(&iseq_diags(src), "ISEQ"), "{src}");
        }
    }

    #[test]
    fn iseq_silent_on_variables_scalar_literal_and_non_condition() {
        for src in [
            "if a == b\nend\n",
            "if a == [2]\nend\n",
            "x = a == [1 2];\n",
            "if s == 'abc'\nend\n",
            "if isequal(a, [1 2])\nend\n",
        ] {
            assert!(!has_id(&iseq_diags(src), "ISEQ"), "{src}");
        }
    }
}
