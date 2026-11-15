//! Node-level check for elementwise `&` / `|` in boolean contexts: AND2 and OR2.

use super::*;
use crate::analysis::elementwise::{
    call_name, elementwise_operands, is_known_logical, operator_token,
};

impl PerformanceEngine {
    /// Check `binary_operator` for AND2/OR2 patterns.
    ///
    /// In tree-sitter-matlab, `&&` and `||` parse as `boolean_operator`, while
    /// `&` and `|` parse as `binary_operator`. This check fires on
    /// `binary_operator` nodes with `&` or `|` that are inside boolean
    /// contexts (if/while conditions). The operands are the ones MATLAB sees
    /// (see `analysis::elementwise`), and the diagnostic sits on the operator.
    pub(crate) fn check_boolean_context_binary<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let op = extract_operator(node, source);

        // AND2: & in boolean context
        if self.is_enabled("AND2")
            && op == "&"
            && and2_applies(node)
            && !has_settled_operand(node, source)
        {
            diags.push(make_diag_with_fix(
                "AND2",
                operator_token(node).unwrap_or(node),
                fix_replace_operator(node, source, "&", "&&").map(|f| f.unsafe_fix()),
            ));
        }

        // OR2: | in boolean context
        if self.is_enabled("OR2")
            && op == "|"
            && is_in_condition(node)
            && or2_applies(node, source)
            && !has_settled_operand(node, source)
        {
            diags.push(make_diag_with_fix(
                "OR2",
                operator_token(node).unwrap_or(node),
                fix_replace_operator(node, source, "|", "||").map(|f| f.unsafe_fix()),
            ));
        }

        diags
    }
}

/// Whether an AND2 candidate `&` is a condition operand, possibly parenthesised.
/// Probed and in the fixtures: MATLAB fires on `if a & b`, `if a > 1 & b < 2`,
/// `if (x & ok) || r`; it is silent inside call arguments or indexing
/// (`isempty(a & b)`, `x(a & b)`).
fn and2_applies(node: tree_sitter::Node) -> bool {
    let mut current = node;
    loop {
        let Some(parent) = current.parent() else {
            return false;
        };
        match parent.kind() {
            "if_statement" | "while_statement" | "elseif_clause" => return true,
            "binary_operator" | "comparison_operator" | "boolean_operator" | "parenthesis" => {
                current = parent
            }
            _ => return false,
        }
    }
}

/// Whether an operand of `&` or `|` is a value MATLAB knows to be logical: a
/// comparison of known values (`L > 1` with `L = 0.75`, `isempty(q) == 0`,
/// `numel(x) > 3`), `true`/`false`, or a variable holding one of these.
/// Probed: MATLAB reports neither AND2 nor OR2 when one operand is such a value.
/// It does report `&`/`|` between bare numbers, between undefined names, and
/// between comparisons of values it does not know (`input`, function results,
/// indexed arrays that grow, struct fields, cell elements).
fn has_settled_operand(node: tree_sitter::Node, source: &str) -> bool {
    elementwise_operands(node, source).is_some_and(|(left, right)| {
        is_known_logical(&left, source) || is_known_logical(&right, source)
    })
}

/// Calls that MATLAB reports as a bare OR2 operand (probed). A bare call not
/// in this list is left alone: MATLAB's answer for it is not known.
const OR2_CALLS: &[&str] = &[
    "numel",
    "length",
    "size",
    "isnan",
    "any",
    "mean",
    "max",
    "min",
    "isfield",
    "isreal",
    "isvector",
    "isscalar",
    "ismember",
    "all",
    "exist",
    "strncmp",
    "contains",
    "startsWith",
    "isfile",
    "isfolder",
    "xor",
    "isinf",
    "isfinite",
    "isrow",
    "iscolumn",
    "ismatrix",
    "not",
    "and",
    "or",
    "isprop",
    "isvarname",
    "isspace",
];

/// Whether an OR2 candidate `|` is a plain condition operand. Probed: MATLAB is
/// silent inside parentheses or call arguments (`(a | b) == 1`, `any(a | b)`)
/// and when a bare operand is a call outside `OR2_CALLS` (`strcmp(a, 'x') | b`).
/// Comparisons with calls inside them (`mean(x) > 1`) are not bare operands.
fn or2_applies(node: tree_sitter::Node, source: &str) -> bool {
    let mut current = node;
    loop {
        let Some(parent) = current.parent() else {
            return false;
        };
        match parent.kind() {
            "if_statement" | "while_statement" | "elseif_clause" => break,
            "binary_operator" | "comparison_operator" | "boolean_operator" => current = parent,
            _ => return false,
        }
    }
    elementwise_operands(node, source).is_some_and(|(left, right)| {
        [left, right]
            .iter()
            .all(|operand| match operand.leaves.as_slice() {
                [leaf] if leaf.kind() == "function_call" => {
                    call_name(*leaf, source).is_some_and(|name| OR2_CALLS.contains(&name))
                }
                _ => true,
            })
    })
}

#[cfg(test)]
mod tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    // -- AND2 / OR2 ----------------------------------------------------------

    #[test]
    fn and2_silent_inside_call_or_index_argument() {
        // Probed: MATLAB reports AND2 for `if a & b` and `if a > 1 & b < 2` only.
        for src in [
            "if isempty(a & b)\n    c = 1;\nend\n",
            "if x(a & b)\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "AND2"), "{src}: got: {diags:?}");
        }
        for src in [
            "if a > 1 & b < 2\n    c = 1;\nend\n",
            "if (a & b)\n    c = 1;\nend\n",
            "if (x & ok) || r\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, "AND2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn or2_silent_on_call_operand_or_parenthesised_operand() {
        // Probed: MATLAB reports no OR2 for `strcmp(...) | b`, `(a | b) == 1`, `any(a | b)`.
        for src in [
            "if strcmp(a, 'x') | b\n    c = 1;\nend\n",
            "if (a | b) == 1\n    c = 1;\nend\n",
            "if any(a | b)\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "OR2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn and2_or2_silent_on_parenthesised_known_logical_operands() {
        // Probed (E1, E2, E3): MATLAB is silent when a parenthesised operand is a known
        // logical predicate (`islogical`, `isempty`, `isnumeric`, `ischar`).
        for src in [
            "function r = e1(x, y)\nr = 0;\nif (islogical(x)) & (y)\n    r = 1;\nend\nend\n",
            "function r = e2(x, y)\nr = 0;\nif (isempty(x)) | (y > 1)\n    r = 1;\nend\nend\n",
            "function r = e3(x, y)\nr = 0;\nif (isnumeric(x)) & (ischar(y))\n    r = 1;\nend\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "AND2"), "{src}: got: {diags:?}");
            assert!(!has_id(&diags, "OR2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn and2_or2_fire_on_parenthesised_comparison_and_plain_operands() {
        // Probed (E4, E6, E7): MATLAB reports AND2 for `(x > 1) & (y < 2)` and `(x) & (y)`,
        // and OR2 for `(x > 1) | (y < 0)`; a comparison of inputs is not a known logical.
        for (src, id) in [
            (
                "function r = e4(x, y)\nr = 0;\nif (x > 1) & (y < 2)\n    r = 1;\nend\nend\n",
                "AND2",
            ),
            (
                "function r = e6(x, y)\nr = 0;\nif (x) & (y)\n    r = 1;\nend\nend\n",
                "AND2",
            ),
            (
                "function r = e7(x, y)\nr = 0;\nif (x > 1) | (y < 0)\n    r = 1;\nend\nend\n",
                "OR2",
            ),
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, id), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn or2_fires_on_plain_condition_operands() {
        let diags = lint_nodes(&*engine(), "if a | b\n    c = 1;\nend\n");
        assert!(has_id(&diags, "OR2"), "got: {diags:?}");
    }

    #[test]
    fn and2_fires_on_elementwise_and_in_condition() {
        let src = "if a & b\n    c = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "AND2"), "got: {diags:?}");
    }

    #[test]
    fn and2_not_fire_on_short_circuit() {
        let src = "if a && b\n    c = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "AND2"), "got: {diags:?}");
    }

    #[test]
    fn or2_fires_on_elementwise_or_in_condition() {
        let src = "if a | b\n    c = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "OR2"), "got: {diags:?}");
    }

    #[test]
    fn or2_not_fire_on_short_circuit() {
        let src = "if a || b\n    c = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "OR2"), "got: {diags:?}");
    }
}

#[cfg(test)]
mod settled_tests {
    use super::super::test_support::engine;
    use crate::test_util::{has_id, lint_nodes};

    #[test]
    fn settled_bare_logical_operands_are_silent() {
        // Probed: MATLAB reports neither OR2 nor AND2 for `true`/`false` variables.
        let diags = lint_nodes(
            &*engine(),
            "a = true; b = false;\nif a | b\n    c = 1;\nend\n",
        );
        assert!(!has_id(&diags, "OR2"), "got: {diags:?}");
    }

    #[test]
    fn bare_numeric_operands_still_fire() {
        // Probed: `x | 0` with a numeric scalar `x` is OR2.
        let diags = lint_nodes(&*engine(), "x = 3;\nif x | 0\n    c = 1;\nend\n");
        assert!(has_id(&diags, "OR2"), "got: {diags:?}");
    }

    /// Lint `src` and return the (rule, column) pairs of the diagnostics.
    fn ids_and_columns(src: &str) -> Vec<(&'static str, usize)> {
        lint_nodes(&*engine(), src)
            .iter()
            .map(|d| (d.rule_id, d.column))
            .collect()
    }

    #[test]
    fn or2_fires_on_comparisons_that_tree_sitter_mis_nests() {
        // Probed (g_cell_counter.m:69 and robot_sim.m:77): comparisons of values
        // MATLAB does not know. The tree has `1 | L`, MATLAB has `(L > 1) | (L < 0.1)`.
        for src in [
            "x = input('x');\nif x > 1 | x < 0.1\n    c = 1;\nend\n",
            "x = input('x');\nif x(1) > 4 | x(1) < -4\n    c = 1;\nend\n",
            "a = [1 2 3];\nfor k = 1:3\n    a(k) = k;\nend\nif a(2) > 1 | a(3) < 0\n    c = 1;\nend\n",
            "s.d = input('x');\nif s.d == 0 | isnan(s.d)\n    c = 1;\nend\n",
            "x = input('x');\nif x == 0 | isnan(x)\n    c = 1;\nend\n",
            "x = input('x');\nif isnan(x) | x > 1\n    c = 1;\nend\n",
            "x = input('x');\nif numel(x) | x > 1\n    c = 1;\nend\n",
            "c = {1, 2};\nif c{1} > 1 | c{2} < 0\n    d = 1;\nend\n",
            "x = input('x');\nif x > 1 | isfield(s, 'a')\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, "OR2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn or2_reports_at_the_operator() {
        // MATLAB reports the column of the `|` (`if x > 1 | ...`: column 10).
        let cols = ids_and_columns("x = input('x');\nif x > 1 | x < 0.1\n    c = 1;\nend\n");
        assert_eq!(cols, vec![("OR2", 10)]);
    }

    #[test]
    fn or2_silent_on_constant_comparison_from_mis_nesting() {
        // Probed (g_lab_analysis.m:197): `L = 0.75` makes `L > 1` a known logical,
        // so MATLAB is silent on `L > 1 | L < 0.1`.
        for src in [
            "L = 0.75;\nif L > 1 | L < 0.1\n    error('L looks wrong')\nend\n",
            "x = 3;\nif x > 1 | x < 2\n    c = 1;\nend\n",
            "x = 0.75;\nif x > 1 | input('y')\n    c = 1;\nend\n",
            "x = input('x');\nif x > 1 | 2 > 3\n    c = 1;\nend\n",
            "x = 2;\nif x > 1 | b\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "OR2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn or2_silent_on_known_logical_call_operands() {
        // Probed: `isempty`, `ischar`, `strcmp` give logical values, so no OR2
        // (`numel` and `length` give numbers and do fire).
        for src in [
            "x = input('x');\nif x > 1 | isempty(x)\n    c = 1;\nend\n",
            "x = input('x');\nif x > 1 | numel(x) > 3\n    c = 1;\nend\n",
            "x = input('x');\nif x > 1 | length(x) > 3\n    c = 1;\nend\n",
            "x = input('x');\nif x == 1 | ischar(x)\n    c = 1;\nend\n",
            "x = input('x');\nn = numel(x);\nif n > 3 | x > 0\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "OR2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn or2_fires_on_mixed_comparison_and_known_call() {
        // Probed (q23): `any(x) == 1` is not known, so the OR2 stays.
        let diags = lint_nodes(
            &*engine(),
            "x = input('x');\nif any(x) == 1 | x > 1\n    c = 1;\nend\n",
        );
        assert!(has_id(&diags, "OR2"), "got: {diags:?}");
    }

    #[test]
    fn and2_fires_on_mis_nested_comparisons() {
        // Probed (p13, p15): comparisons of input values are AND2.
        for src in [
            "running = input('r');\nif running == 1 & running < 3\n    c = 1;\nend\n",
            "x = input('x');\nif x > 1 & x < 3\n    c = 1;\nend\n",
            "x = input('x');\nif x > 1 & 1\n    c = 1;\nend\n",
            "c = {1, 2};\nif c{1} > 1 & c{2} < 2\n    d = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, "AND2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn and2_silent_on_type_predicate_operands() {
        // Probed (j_stats_helpers.m:56, :61, :66): `isstruct`, `isnumeric`, and
        // `~isempty` are known logical, so MATLAB is silent on each of these.
        for src in [
            "out = 1;\nif isstruct(out) & isfield(out, 'mean')\n    c = 1;\nend\n",
            "out = 1;\nif exist('out') & ~isempty(out)\n    c = 1;\nend\n",
            "data = 1;\nif isnumeric(data) & ~isempty(data)\n    c = 1;\nend\n",
            "x = input('x');\nif true & x > 1\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "AND2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn and2_fires_on_predicates_matlab_reports() {
        // Probed: `exist`, `isfile`, `xor`, and `~x` of an unknown input are not
        // known logical, so `&` with such an operand is AND2.
        for src in [
            "x = input('x');\nif exist('x') & x > 1\n    c = 1;\nend\n",
            "x = input('x');\nif isfile(x) & x > 1\n    c = 1;\nend\n",
            "x = input('x');\nif xor(x, 1) & x > 1\n    c = 1;\nend\n",
            "x = input('x');\nif ~x & x > 1\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, "AND2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn or2_fires_on_predicates_matlab_reports_as_bare_operand() {
        // Probed: these predicates are reported as a bare OR2 operand.
        for src in [
            "x = input('x');\nif isvector(x) | x > 1\n    c = 1;\nend\n",
            "x = input('x');\nif contains(x, 'a') | x > 1\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(has_id(&diags, "OR2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn or2_silent_on_type_predicate_bare_operand() {
        // Probed: `isnumeric` and `iscell` are known logical, so `|` is silent.
        for src in [
            "x = input('x');\nif isnumeric(x) | x > 1\n    c = 1;\nend\n",
            "x = input('x');\nif iscell(x) | x > 1\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "OR2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn and2_silent_on_known_logical_operands_from_mis_nesting() {
        // Probed (g_menu_tool.m:107): `running = 1` makes `running == 1` known, and
        // `isempty(items) == 0` is a known logical, so MATLAB is silent.
        // Probed (p11, p12, p19): with `isempty(items) == 0` on one side, MATLAB
        // is silent even when the other side is an unknown input comparison.
        for src in [
            "running = 1;\nitems = {};\nif running == 1 & isempty(items) == 0\n    c = 1;\nend\n",
            "items = input('i');\nrunning = input('r');\nif running == 1 & isempty(items) == 0\n    c = 1;\nend\n",
            "flag = true;\nx = input('x');\nif flag & x > 1\n    c = 1;\nend\n",
        ] {
            let diags = lint_nodes(&*engine(), src);
            assert!(!has_id(&diags, "AND2"), "{src}: got: {diags:?}");
        }
    }

    #[test]
    fn or2_on_size_operand_in_a_condition() {
        // Probed: `size` is a bare OR2 operand as `numel` is (`if size(a, 1) | b`);
        // `c = size(a) | b;` is not a condition and stays silent.
        let diags = lint_nodes(
            &*engine(),
            "a = rand(3);\nb = rand;\nif size(a, 1) | b\n    c = 1;\nend\n",
        );
        assert!(has_id(&diags, "OR2"), "got: {diags:?}");
        let diags = lint_nodes(&*engine(), "a = rand(3);\nb = rand;\nc = size(a) | b;\n");
        assert!(!has_id(&diags, "OR2"), "got: {diags:?}");
    }
}
