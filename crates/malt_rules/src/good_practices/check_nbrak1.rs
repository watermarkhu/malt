use super::*;

impl GoodPracticesEngine {
    /// NBRAK1: square brackets used to group an expression for precedence.
    ///
    /// `[...]` concatenates into a matrix; when a single non-literal expression
    /// is wrapped in brackets the intent is usually precedence grouping, which
    /// should use parentheses instead.
    pub(crate) fn check_nbrak1(&self, node: Node, source: &str) -> Vec<Diagnostic> {
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
        // `(x)` equals `[x]` only for a plain expression. A comma-separated list
        // (`[c{:}]`, `[s.f]`) would stop being concatenated, a handle or an
        // indexed result would not be valid in parentheses, and a stray
        // separator (`[a,]`) would not survive the swap. Report those, but do
        // not rewrite them.
        let plain = matches!(
            element.kind(),
            "identifier"
                | "binary_operator"
                | "unary_operator"
                | "comparison_operator"
                | "boolean_operator"
                | "postfix_operator"
                | "parenthesis"
        );
        let padded = source[start + 1..element.start_byte()].trim().is_empty()
            && source[element.end_byte()..end - 1].trim().is_empty();
        let indexed_after = node.parent().is_some_and(|p| {
            matches!(
                p.kind(),
                "function_call" | "field_expression" | "cell_index"
            )
        });
        let fix = (plain && padded && !indexed_after).then(|| {
            malt_core::Fix::with_additional(
                start..start + 1,
                "(",
                vec![malt_core::Fix::new(end - 1..end, ")")],
            )
        });
        vec![Diagnostic {
            rule_id: "NBRAK1",
            message: "If you intend to specify expression precedence, use parentheses () instead of brackets [].".to_string(),
            severity: Severity::Info,
            byte_range: start..end,
            line: pos.row + 1,
            column: pos.column + 1,
            fix,
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

#[cfg(test)]
mod fix_tests {
    use super::*;

    fn nbrak1(source: &str) -> Vec<Diagnostic> {
        let tree = parse(source);
        let matrix = find_descendant_of_kind(tree.root_node(), "matrix").unwrap();
        engine().check_nbrak1(matrix, source)
    }

    #[test]
    fn rewrites_plain_expressions_only() {
        let d = nbrak1("x = [a + b];\n");
        assert!(d[0].fix.is_some());
        let d = nbrak1("x = [a'];\n");
        assert!(d[0].fix.is_some());
    }

    #[test]
    fn leaves_lists_handles_and_stray_separators_alone() {
        // `(c{:})` would no longer concatenate the list; the rest would become
        // invalid or mean something else in parentheses.
        for source in [
            "x = [c{:}];\n",
            "x = [s.f];\n",
            "x = [@sin];\n",
            "x = [a;];\n",
            "x = [a,];\n",
        ] {
            for d in nbrak1(source) {
                assert!(d.fix.is_none(), "{source:?} must not be rewritten");
            }
        }
    }
}
