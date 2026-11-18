//! Node-level check for `comparison_operator` nodes: ISMT (`length(x) == 0`) and ISCL (`length(x) == 1`).

use super::*;

impl PerformanceEngine {
    /// Check `comparison_operator` for ISMT/ISCL patterns.
    pub(crate) fn check_comparison<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // Pattern: length(x) == 0 → isempty(x)
        // Pattern: length(x) == 1 → isscalar(x)
        if let Some((func, val)) = extract_length_comparison(node, source) {
            let op = node.child(1).map(|op| node_text(op, source));
            if func == "length" && self.is_enabled("ISMT") && val == "0" {
                // MATLAB's fix rewrites the whole comparison: `length(x) == 0`
                // becomes `isempty(x)`, `0 == length(x)` becomes `isempty(x)`
                // and `length(x) ~= 0` becomes `~isempty(x)` (probed on all
                // three). Unsafe: a user class can override `length` or
                // `isempty`, and the rewrite keeps neither the argument's
                // evaluation order nor its class guarantee for such a class.
                diags.push(make_diag_with_fix(
                    "ISMT",
                    node,
                    comparison_arg(node, source).map(|arg| {
                        let negation = if op == Some("~=") { "~" } else { "" };
                        Fix::new(
                            node.start_byte()..node.end_byte(),
                            format!("{negation}isempty({arg})"),
                        )
                        .unsafe_fix()
                    }),
                ));
            }
            let is_equality = op == Some("==");
            if self.is_enabled("ISCL") && is_equality && val == "1" {
                // `length(x) == 1` and `numel(x) == 1`, either order, become
                // `isscalar(x)` (probed). Unsafe for the same reason as ISMT:
                // a class that overrides `length`/`numel` can disagree with
                // `isscalar`.
                diags.push(make_diag_with_fix(
                    "ISCL",
                    node,
                    comparison_arg(node, source)
                        .map(|arg| Fix::new(node.byte_range(), format!("isscalar({arg})")))
                        .map(Fix::unsafe_fix),
                ));
            }
        }

        // Probed: `length(x) > 0` is ISMT too; `>= 1`, `< 1` and `0 < length(x)` are not.
        if self.is_enabled("ISMT") && is_length_greater_than_zero(node, source) {
            // MATLAB's fix: `length(x) > 0` becomes `~isempty(x)` (probed).
            diags.push(make_diag_with_fix(
                "ISMT",
                node,
                comparison_arg(node, source).map(|arg| {
                    Fix::new(node.byte_range(), format!("~isempty({arg})")).unsafe_fix()
                }),
            ));
        }

        diags
    }
}

impl PerformanceEngine {
    /// ISMT for a `length(x)` call used as the truth value of an `if`, `while` or
    /// `elseif` condition, directly or as an operand of `&`, `|`, `&&`, `||` or `~`.
    /// Probed (MATLAB R2026b): `if length(a)`, `if length(a) & b`, `while ~length(a) | b`
    /// are ISMT; `x = length(a) & b`, `length(a) > 1` and `numel(a) & b` are not.
    pub(crate) fn check_condition_length<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        if !self.is_enabled("ISMT") {
            return Vec::new();
        }
        let condition = (0..node.named_child_count())
            .filter_map(|i| node.named_child(i))
            .find(|c| {
                !matches!(
                    c.kind(),
                    "block" | "comment" | "else_clause" | "elseif_clause"
                )
            });
        let mut diags = Vec::new();
        if let Some(condition) = condition {
            collect_truth_length(condition, source, &mut diags);
        }
        diags
    }
}

/// Pushes ISMT for each `length(...)` call in a boolean position of `node`.
fn collect_truth_length(node: tree_sitter::Node, source: &str, diags: &mut Vec<Diagnostic>) {
    match node.kind() {
        "function_call" if call_path(node, source, Dotted::Skip) == Some("length") => {
            // MATLAB's fix rewrites the call itself (`if length(a)` becomes
            // `if ~isempty(a)`; under an existing `~` it keeps the outer
            // negation, giving `~~isempty(a)`, probed). Unsafe for the same
            // class-override reason as the comparison form.
            let fix = boolean_position_fix(node, source, "~isempty");
            diags.push(make_diag_with_fix("ISMT", node, fix));
        }
        "parenthesis" => {
            if let Some(inner) = node.named_child(0) {
                collect_truth_length(inner, source, diags);
            }
        }
        "not_operator" => {
            if let Some(operand) = node.named_child(0) {
                collect_truth_length(operand, source, diags);
            }
        }
        // The grammar binds `&` and `|` tighter than comparisons, so `length(a) == 0 & b`
        // is `length(a) == (0 & b)`; MATLAB reads it as `(length(a) == 0) & b` (probed).
        "comparison_operator" => {
            let right = node.child(2);
            let Some(logical) = right.filter(|r| r.kind() == "binary_operator") else {
                return;
            };
            let (Some(left), Some(op), Some(zero), Some(rest)) = (
                node.child(0),
                node.child(1),
                logical.child(0),
                logical.child(2),
            ) else {
                return;
            };
            let logical_op = logical.child(1).map(|o| node_text(o, source));
            if left.kind() == "function_call"
                && call_path(left, source, Dotted::Skip) == Some("length")
                && matches!(node_text(op, source), "==" | "~=" | ">")
                && node_text(zero, source).trim() == "0"
                && matches!(logical_op, Some("&" | "|"))
            {
                // Only `length(a) == 0` is rewritten; `& b` stays (`if
                // isempty(a) & b`, probed). Unsafe like the other forms.
                let fix = call_args_text(left, source).map(|arg| {
                    let negation = if node_text(op, source) == "==" {
                        ""
                    } else {
                        "~"
                    };
                    Fix::new(
                        node.start_byte()..zero.end_byte(),
                        format!("{negation}isempty({arg})"),
                    )
                    .unsafe_fix()
                });
                diags.push(make_diag_with_fix("ISMT", node, fix));
            }
            collect_truth_length(rest, source, diags);
        }
        "boolean_operator" | "binary_operator" => {
            let is_logical = node
                .child(1)
                .is_some_and(|op| matches!(node_text(op, source), "&" | "|" | "&&" | "||"));
            if is_logical {
                for i in [0, 2] {
                    if let Some(operand) = node.child(i) {
                        collect_truth_length(operand, source, diags);
                    }
                }
            }
        }
        _ => {}
    }
}

/// The text of a call's arguments, verbatim.
fn call_args_text<'a>(call: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    let args = find_arguments(call)?;
    let mut cursor = args.walk();
    let arg_nodes: Vec<_> = args.named_children(&mut cursor).collect();
    let first = *arg_nodes.first()?;
    let last = *arg_nodes.last()?;
    Some(&source[first.start_byte()..last.end_byte()])
}

/// The `<name>(<args>)` rewrite of a call, verbatim arguments.
fn boolean_position_fix(call: tree_sitter::Node, source: &str, name: &str) -> Option<Fix> {
    let arg = call_args_text(call, source)?;
    Some(Fix::new(call.byte_range(), format!("{name}({arg})")).unsafe_fix())
}

/// `length(x) > 0`, with the call on the left.
fn is_length_greater_than_zero(node: tree_sitter::Node, source: &str) -> bool {
    let (Some(left), Some(op), Some(right)) = (node.child(0), node.child(1), node.child(2)) else {
        return false;
    };
    node_text(op, source) == ">"
        && left.kind() == "function_call"
        && call_path(left, source, Dotted::Skip) == Some("length")
        && node_text(right, source).trim() == "0"
}

/// The argument text of the `length`/`numel` call on one side of a comparison
/// matched by [`extract_length_comparison`], verbatim.
fn comparison_arg<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    let call = [node.child(0)?, node.child(2)?].into_iter().find(|c| {
        c.kind() == "function_call"
            && matches!(
                call_path(*c, source, Dotted::Skip),
                Some("length" | "numel")
            )
    })?;
    call_args_text(call, source)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- ISMT / ISCL ----------------------------------------------------------

    #[test]
    fn ismt_fires_on_length_zero() {
        let src = "y = (length(x) == 0);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "ISMT"), "got: {diags:?}");
    }

    #[test]
    fn ismt_fires_on_length_not_equal_zero() {
        // Probed: `(length(x) ~= 0)` is ISMT; ISCL is for `==` only.
        let diags = lint_nodes(&*engine(), "y = (length(x) ~= 0);\n");
        assert!(has_id(&diags, "ISMT"), "got: {diags:?}");
        let diags = lint_nodes(&*engine(), "y = (length(x) ~= 1);\n");
        assert!(!has_id(&diags, "ISCL"), "got: {diags:?}");
    }

    #[test]
    fn ismt_not_fire_on_other_length() {
        let src = "y = (length(x) == 2);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "ISMT"), "got: {diags:?}");
    }

    #[test]
    fn iscl_fires_on_length_one() {
        let src = "y = (length(x) == 1);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "ISCL"), "got: {diags:?}");
    }

    #[test]
    fn iscl_fires_on_numel_one() {
        let src = "y = (numel(x) == 1);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "ISCL"), "got: {diags:?}");
    }

    #[test]
    fn ismt_not_fire_on_numel_zero() {
        let src = "y = (numel(x) == 0);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "ISMT"), "got: {diags:?}");
    }

    #[test]
    fn iscl_not_fire_on_other_length() {
        let src = "y = (length(x) == 2);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "ISCL"), "got: {diags:?}");
    }
}

#[cfg(test)]
mod ismt_greater_tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    #[test]
    fn ismt_fires_on_length_greater_than_zero() {
        // Probed: `length(x) > 0` in a while condition is ISMT (undefined and constant x).
        for src in [
            "while length(stack) > 0\n    stack(end) = [];\nend\n",
            "y = (length(x) > 0);\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, "ISMT"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn ismt_silent_on_other_length_comparisons() {
        // Probed silent: `>= 1`, `< 1`, `0 < length(x)`, `numel(x) > 0`.
        for src in [
            "y = (length(x) >= 1);\n",
            "y = (length(x) < 1);\n",
            "y = (0 < length(x));\n",
            "y = (numel(x) > 0);\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "ISMT"), "{src}: got: {diags:?}");
        }
    }
}

#[cfg(test)]
mod ismt_condition_tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    #[test]
    fn ismt_fires_on_length_as_a_condition_truth_value() {
        // Probed (si1, si2, si3, si5, si6, si7, si12, zf_ismt1): reported at `length`.
        for src in [
            "if length(a)\n    disp(1);\nend\n",
            "if length(a) && b\n    disp(1);\nend\n",
            "if length(a) | b\n    disp(1);\nend\n",
            "while length(a) & b\n    disp(1);\nend\n",
            "if ~length(a) & b\n    disp(1);\nend\n",
            "if (length(a)) & b\n    disp(1);\nend\n",
            "if length(a) & a\n    disp(1);\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, "ISMT"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn ismt_silent_on_length_outside_a_condition_or_in_other_comparisons() {
        // Probed silent (si10, si11, si13, zf_ismt2, zf_ismt4): not a condition, numel,
        // comparison with 1.
        for src in [
            "x = length(a) & b;\n",
            "c = length(a) & b;\nif c\n    disp(1);\nend\n",
            "if numel(a) & b\n    disp(1);\nend\n",
            "if length(a) > 1 & b\n    disp(1);\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "ISMT"), "{src}: got: {diags:?}");
        }
    }
}
