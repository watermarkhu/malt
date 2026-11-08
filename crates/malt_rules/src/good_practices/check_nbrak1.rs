use super::*;

impl GoodPracticesEngine {
    /// NBRAK1: square brackets used to group an expression for precedence.
    ///
    /// `[...]` concatenates into a matrix; when a single non-literal expression
    /// is wrapped in brackets the intent is usually precedence grouping, which
    /// should use parentheses instead.
    pub(crate) fn check_nbrak1(&self, node: Node, _source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("NBRAK1") || node.kind() != "matrix" {
            return Vec::new();
        }

        let Some(element) = single_matrix_element(node) else {
            return Vec::new();
        };

        // `[5]` or `['abc']` are legitimate 1x1 array literals.
        if element.kind() == "number" || element.kind() == "string" {
            return Vec::new();
        }

        let start = node.start_byte();
        let end = node.end_byte();
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "NBRAK1",
            message: "If you intend to specify expression precedence, use parentheses () instead of brackets [].".to_string(),
            severity: Severity::Info,
            byte_range: start..end,
            line: pos.row + 1,
            column: pos.column + 1,
            fix: Some(malt_core::Fix::with_additional(
                start..start + 1,
                "(",
                vec![malt_core::Fix::new(end - 1..end, ")")],
            )),
        }]
    }
}

/// Return the single element of a `matrix` node when it contains exactly one
/// non-trivial element (a single `row` with one expression).
fn single_matrix_element<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut elements = Vec::new();
    collect_matrix_elements(node, &mut elements);
    if elements.len() == 1 {
        Some(elements[0])
    } else {
        None
    }
}

/// Collect named elements inside a `matrix`, flattening `row` nodes and
/// skipping comments and line continuations.
fn collect_matrix_elements<'a>(node: Node<'a>, out: &mut Vec<Node<'a>>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "row" => collect_matrix_elements(child, out),
            "comment" | "line_continuation" => {}
            _ if child.is_named() => out.push(child),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nbrak1_fires_on_single_expression_in_brackets() {
        let source = "x = [a + b];\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let matrix = find_descendant_of_kind(root, "matrix").unwrap();
        let diags = eng.check_nbrak1(matrix, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "NBRAK1");
    }

    #[test]
    fn test_nbrak1_fires_on_single_identifier_in_brackets() {
        let source = "x = [a];\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let matrix = find_descendant_of_kind(root, "matrix").unwrap();
        let diags = eng.check_nbrak1(matrix, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "NBRAK1");
    }

    #[test]
    fn test_nbrak1_silent_on_multi_element_matrix() {
        let source = "x = [1 2 3];\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let matrix = find_descendant_of_kind(root, "matrix").unwrap();
        let diags = eng.check_nbrak1(matrix, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_nbrak1_silent_on_scalar_literal() {
        let source = "x = [5];\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let matrix = find_descendant_of_kind(root, "matrix").unwrap();
        let diags = eng.check_nbrak1(matrix, source);
        assert!(diags.is_empty());
    }
}
