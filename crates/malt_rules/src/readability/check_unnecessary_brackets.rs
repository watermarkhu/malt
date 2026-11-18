//! # NBRAK2: unnecessary brackets around a scalar expression
//!
//! `check_unnecessary_brackets` flags `[x]` where `x` is a single expression
//! (identifier, literal, operator expression, or call), when the brackets are
//! not an operand of a larger expression (that case is NBRAK1).

use super::*;

impl ReadabilityEngine {
    /// NBRAK2: Unnecessary brackets `[x]` for scalar expression.
    pub(crate) fn check_unnecessary_brackets(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("NBRAK2")
            || crate::syntax_errors::has_end_value_in_literal(node, source)
        {
            return;
        }

        // A matrix node with exactly one named child that is a "row" with
        // exactly one element is `[scalar]`.
        let mut cursor = node.walk();
        let elements: Vec<_> = node
            .named_children(&mut cursor)
            .filter(|c| c.kind() != "comment")
            .collect();
        if elements.len() != 1 {
            return;
        }

        let row = elements[0];

        // An operand of a larger expression (`[x]+1`) is NBRAK1, and an indexed
        // bracket (`[x](1)`) is not redundant. MATLAB reports neither as NBRAK2.
        let operand_or_indexed = node.parent().is_some_and(|p| {
            matches!(
                p.kind(),
                "binary_operator"
                    | "comparison_operator"
                    | "boolean_operator"
                    | "unary_operator"
                    | "postfix_operator"
            )
        }) || source[node.end_byte()..].starts_with('(');

        // The row should have exactly one named child (the scalar expression)
        if row.kind() == "row" && row.named_child_count() == 1 && !operand_or_indexed {
            if let Some(inner) = row.named_child(0) {
                let inner_text = &source[inner.start_byte()..inner.end_byte()];
                // Comma-separated lists (`[s.f]`, `[c{:}]`) can yield several
                // values, so the brackets are not redundant.
                // A cell literal `[{1}]` is one element; `c{:}` is a list.
                // `[obj.Value(end)]` is an indexed field, a single value (probed, plan 6.2).
                let list_like = (inner.kind() == "field_expression"
                    && !crate::good_practices::is_indexed_field(inner))
                    || (inner_text.contains('{') && inner.kind() != "cell");
                // Probed against MATLAB R2026b: single identifiers, literals,
                // operator expressions, calls, ranges, cells, handles, negations
                // and parenthesised expressions are all reported.
                let scalar = crate::good_practices::is_indexed_field(inner)
                    || matches!(
                        inner.kind(),
                        "identifier"
                            | "number"
                            | "string"
                            | "binary_operator"
                            | "unary_operator"
                            | "comparison_operator"
                            | "boolean_operator"
                            | "postfix_operator"
                            | "function_call"
                            | "range"
                            | "cell"
                            | "function_handle"
                            | "handle_operator"
                            | "not_operator"
                            | "parenthesis"
                    );
                if scalar && !list_like {
                    // MATLAB's own fix strips the two bracket tokens and keeps
                    // everything between them verbatim (probed on the fixtures:
                    // `[a(1) ]` becomes `y = a(1) ;`, and the comment inside
                    // `[a  % comment ...]` survives). Replacing the whole node
                    // range with the element text would drop such a comment.
                    results.push(self.diag(
                        "NBRAK2",
                        "Use of brackets [] is unnecessary.",
                        node,
                        Some(Fix::with_additional(
                            node.start_byte()..node.start_byte() + 1,
                            "",
                            vec![Fix::new(node.end_byte() - 1..node.end_byte(), "")],
                        )),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    // -- NBRAK2 --------------------------------------------------------------

    #[test]
    fn nbrak2_brackets_around_scalar_fires() {
        let diags = lint_nodes(&*engine(), "x = [5];\n");
        assert!(has_id(&diags, "NBRAK2"), "got: {diags:?}");
    }

    #[test]
    fn nbrak2_brackets_around_indexed_field_fires() {
        // Probed (plan 6.2): `[obj.Value(end)]` is NBRAK2, `[obj.a]` is not.
        assert!(has_id(
            &lint_nodes(&*engine(), "x = [obj.Value(end)];\n"),
            "NBRAK2"
        ));
        assert!(!has_id(&lint_nodes(&*engine(), "x = [obj.a];\n"), "NBRAK2"));
    }

    #[test]
    fn nbrak2_brackets_around_identifier_fires() {
        let diags = lint_nodes(&*engine(), "x = [v];\n");
        assert!(has_id(&diags, "NBRAK2"), "got: {diags:?}");
    }

    #[test]
    fn nbrak2_multi_element_matrix_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = [1 2];\n");
        assert!(!has_id(&diags, "NBRAK2"), "got: {diags:?}");
    }

    // Probed against MATLAB R2026b: each of these is NBRAK2.
    #[test]
    fn nbrak2_fires_on_whole_expression_and_call_argument() {
        for source in [
            "x = [a + b];\n",
            "x = [-x];\n",
            "x = [a == b];\n",
            "x = [f(x)];\n",
            "x = [1:3];\n",
            "x = [a,];\n",
            "disp([x]);\n",
            "y = [{1}];\n",
            "y = [(a && b)];\n",
            "y = [a  % comment\n];\n",
            "y = [~a];\n",
            "y = [@sin];\n",
        ] {
            let diags = lint_nodes(&*engine(), source);
            assert!(has_id(&diags, "NBRAK2"), "{source}: got: {diags:?}");
        }
    }

    #[test]
    fn nbrak2_fix_strips_only_the_bracket_tokens() {
        // Probed: MATLAB's own fix keeps everything between the brackets
        // verbatim, comments and spacing included, and malt now does too
        // (the old whole-node rewrite deleted the comment).
        for (source, fixed) in [
            ("x = [5];\n", "x = 5;\n"),
            ("y = [a(1) ];\n", "y = a(1) ;\n"),
            ("y = [a  % comment\n];\n", "y = a  % comment\n;\n"),
            ("y = [ a ];\n", "y =  a ;\n"),
        ] {
            let diags = lint_nodes(&*engine(), source);
            let d = diags
                .iter()
                .find(|d| d.rule_id == "NBRAK2")
                .expect("NBRAK2");
            let fix = d.fix.as_ref().expect("fix");
            // Back to front, as the applier does, so offsets stay valid.
            let mut edits: Vec<(std::ops::Range<usize>, String)> = fix
                .additional_edits
                .iter()
                .map(|e| (e.byte_range.clone(), e.replacement.clone()))
                .chain(std::iter::once((
                    fix.byte_range.clone(),
                    fix.replacement.clone(),
                )))
                .collect();
            edits.sort_by_key(|(r, _)| std::cmp::Reverse(r.start));
            let mut out = String::from(source);
            for (range, text) in edits {
                out.replace_range(range, &text);
            }
            assert_eq!(out, fixed, "{source}");
        }
    }

    // Probed against MATLAB R2026b: these are not NBRAK2. `[x] + 1` is NBRAK1
    // (operand of an operator), and the rest are lists or indexed brackets.
    #[test]
    fn nbrak2_silent_on_operand_list_and_indexed_brackets() {
        for source in [
            "y = [x] + 1;\n",
            "y = 2 * [x];\n",
            "y = [s.f];\n",
            "y = [c{:}];\n",
            "y = [x](1);\n",
        ] {
            let diags = lint_nodes(&*engine(), source);
            assert!(!has_id(&diags, "NBRAK2"), "{source}: got: {diags:?}");
        }
    }
}
