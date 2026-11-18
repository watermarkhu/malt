use super::*;

impl GoodPracticesEngine {
    /// NBRAK1: square brackets used to group an expression for precedence.
    ///
    /// MATLAB reports this only when the single-element brackets are an operand
    /// of a larger expression (`[x]+1`, `2*[x]`, `[x]'`). Brackets that are the
    /// whole right-hand side or a function argument are NBRAK2 instead.
    pub(crate) fn check_nbrak1(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("NBRAK1") || node.kind() != "matrix" {
            return Vec::new();
        }

        let Some(element) = single_matrix_element(node) else {
            return Vec::new();
        };

        // `[5]` or `['abc']` are legitimate 1x1 array literals, but a number that
        // is an operand is a precedence group (probed: `[2] == x`, `x == [2]`,
        // `[2] * x` are NBRAK1, while `if [1]` is NBRAK2).
        let number_operand = element.kind() == "number"
            && node
                .parent()
                .is_some_and(|p| matches!(p.kind(), "comparison_operator" | "binary_operator"));
        if (element.kind() == "number" && !number_operand) || element.kind() == "string" {
            return Vec::new();
        }
        // `[c{:}]` and `[s.f]` are comma-separated lists, not precedence groups.
        // `[obj.Value(end)]` is an indexed field and a single value (probed, plan 6.2).
        if (element.kind() == "field_expression" && !is_indexed_field(element))
            || source[element.byte_range()].contains('{')
        {
            return Vec::new();
        }
        let operand = node.parent().is_some_and(|p| {
            matches!(
                p.kind(),
                "binary_operator"
                    | "comparison_operator"
                    | "boolean_operator"
                    | "unary_operator"
                    | "postfix_operator"
            )
        });
        if !operand {
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
            severity: Severity::Warning,
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
    fn test_nbrak1_fires_on_indexed_field_but_not_on_plain_field() {
        // Probed (plan 6.2): `[obj.Value( end ) ]* n` is NBRAK1, `[obj.a] * n` is silent.
        for (source, count) in [
            ("r = [obj.Value( end ) ]* n;\n", 1),
            ("r = [obj.Value(1)] * n;\n", 1),
            ("r = [obj.a] * n;\n", 0),
        ] {
            let tree = parse(source);
            let matrix = find_descendant_of_kind(tree.root_node(), "matrix").unwrap();
            assert_eq!(
                engine().check_nbrak1(matrix, source).len(),
                count,
                "{source}"
            );
        }
    }

    #[test]
    fn test_nbrak1_fires_on_single_expression_in_brackets() {
        let source = "x = [a + b] * 2;\n";
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
        let source = "x = [a] + 1;\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let matrix = find_descendant_of_kind(root, "matrix").unwrap();
        let diags = eng.check_nbrak1(matrix, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "NBRAK1");
    }

    #[test]
    fn test_nbrak1_silent_on_whole_right_hand_side() {
        // MATLAB R2026b reports `x = [a];` and `x = [a + b];` as NBRAK2 (probe),
        // so NBRAK1 must not fire on a bracket that is not an operand.
        for source in ["x = [a];\n", "x = [a + b];\n", "disp([a]);\n"] {
            let tree = parse(source);
            let matrix = find_descendant_of_kind(tree.root_node(), "matrix").unwrap();
            assert!(engine().check_nbrak1(matrix, source).is_empty(), "{source}");
        }
    }

    #[test]
    fn test_nbrak1_silent_on_comma_separated_list_operand() {
        // `[c{1}]` and `[s.f]` are lists, not precedence groups (probe: silent).
        for source in ["y = [c{1}] + 1;\n", "y = [s.f] + 1;\n"] {
            let tree = parse(source);
            let matrix = find_descendant_of_kind(tree.root_node(), "matrix").unwrap();
            assert!(engine().check_nbrak1(matrix, source).is_empty(), "{source}");
        }
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
        let d = nbrak1("x = [a + b] * 2;\n");
        assert!(d[0].fix.is_some());
        let d = nbrak1("x = [a]' + 1;\n");
        assert!(d[0].fix.is_some());
    }

    #[test]
    fn leaves_lists_handles_and_stray_separators_alone() {
        // `(c{:})` would no longer concatenate the list; the rest would become
        // invalid or mean something else in parentheses.
        for source in [
            "x = [c{:}] + 1;\n",
            "x = [s.f] + 1;\n",
            "x = [@sin] + 1;\n",
            "x = [a;] + 1;\n",
            "x = [a,] + 1;\n",
        ] {
            for d in nbrak1(source) {
                assert!(d.fix.is_none(), "{source:?} must not be rewritten");
            }
        }
    }
}

#[cfg(test)]
mod number_operand_tests {
    use super::*;

    fn nbrak1(source: &str) -> usize {
        let tree = parse(source);
        let mut count = 0;
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            count += engine().check_nbrak1(n, source).len();
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        count
    }

    #[test]
    fn fires_on_a_number_that_is_an_operand() {
        // Probed: `[2] == x`, `x == [2]`, `[2] * x` and `a = [2] == x` are NBRAK1.
        for source in [
            "if [2] == x\nend\n",
            "if x == [2]\nend\n",
            "a = [2] == x;\n",
            "b = [2] * x;\n",
        ] {
            assert_eq!(nbrak1(source), 1, "{source}");
        }
    }

    #[test]
    fn silent_on_a_bare_number_or_a_multi_element_operand() {
        // Probed: `if [1]` is NBRAK2 (not reported here); `x == [2 3]` is a list.
        for source in ["if [1]\nend\n", "c = x == [2 3];\n", "x = [5];\n"] {
            assert_eq!(nbrak1(source), 0, "{source}");
        }
    }
}
