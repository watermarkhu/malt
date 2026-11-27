use super::*;

impl GoodPracticesEngine {
    /// STCMP: Use `strcmp`/`strcmpi` instead of `==` for string comparison.
    pub(crate) fn check_stcmp(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("STCMP") {
            return Vec::new();
        }
        if node.kind() != "comparison_operator" && node.kind() != "binary_operator" {
            return Vec::new();
        }

        // MATLAB reports only `==`/`~=` in an if or while condition, against a
        // single-quoted literal of two or more characters (probed). Plain
        // assignments such as `v = s == 'ab'` and double-quoted strings are silent.
        if !is_condition_operand(node, source) {
            return Vec::new();
        }
        let mut cursor = node.walk();
        let is_comparison = node
            .children(&mut cursor)
            .any(|c| !c.is_named() && matches!(node_text(c, source), "==" | "~="));
        let has_string_operand = node
            .children(&mut node.walk())
            .any(|c| c.kind() == "string" && is_multichar_single_quoted(node_text(c, source)));
        if !is_comparison || !has_string_operand {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "STCMP",
            message: "Use STRCMP instead of == or ~= to compare character vectors, or convert character vectors to string scalars for direct comparison.".to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: strcmp_fix(node, source),
        }]
    }
}

/// The unsafe fix of a char-vector comparison: `a == b` becomes
/// `strcmp(a, b)` and `a ~= b` becomes `~strcmp(a, b)`.
///
/// Unsafe because `==` compares character by character and errors on operands
/// of different lengths where `strcmp` answers false, and because `strcmp`
/// accepts cell arrays and gives false for non-text operands. Offered only
/// for plain operands (names, field reads, calls and literals), so no
/// precedence can change, and a `~=` only where a `~` binds the call alone.
pub(crate) fn strcmp_fix(node: Node, source: &str) -> Option<malt_core::Fix> {
    let (left, op, right) = (node.child(0)?, node.child(1)?, node.child(2)?);
    if node.child_count() != 3 {
        return None;
    }
    let plain = |n: Node| {
        matches!(
            n.kind(),
            "identifier" | "string" | "field_expression" | "function_call"
        )
    };
    if !plain(left) || !plain(right) {
        return None;
    }
    let negate = match node_text(op, source) {
        "==" => "",
        "~=" => "~",
        _ => return None,
    };
    if !negate.is_empty() {
        let parent = node.parent()?;
        let bound = matches!(
            parent.kind(),
            "if_statement"
                | "elseif_clause"
                | "assignment"
                | "while_statement"
                | "parenthesis"
                | "boolean_operator"
        );
        if !bound {
            return None;
        }
    }
    let text = format!(
        "{negate}strcmp({}, {})",
        node_text(left, source),
        node_text(right, source)
    );
    Some(malt_core::Fix::new(node.byte_range(), text).unsafe_fix())
}

/// Whether `node` is part of the condition of an `if`, `elseif` or `while`, or
/// an operand of a short-circuit `||` or `&&` (probed: `k = s == 'abc' || 1`
/// is reported in an assignment, `s == 'abc' & t` is not).
fn is_condition_operand(node: Node, source: &str) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "if_statement" | "elseif_clause" | "while_statement" => return true,
            "boolean_operator"
                if matches!(find_operator_text(parent, source).as_str(), "||" | "&&") =>
            {
                return true
            }
            "block" | "source_file" | "assignment" | "function_call" | "function_definition" => {
                return false
            }
            _ => current = parent.parent(),
        }
    }
    false
}

/// A single-quoted character literal of two or more characters.
fn is_multichar_single_quoted(text: &str) -> bool {
    text.starts_with('\'') && text.trim_matches('\'').chars().count() >= 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{fix_of, reported_without_fix};

    #[test]
    fn stcmp_fix_rewrites_to_strcmp_and_is_unsafe() {
        let (safe, fixed) = fix_of(&engine(), "if s == 'abc'\n    x = 1;\nend\n", "STCMP").unwrap();
        assert!(!safe);
        assert_eq!(fixed, "if strcmp(s, 'abc')\n    x = 1;\nend\n");
        let (_, fixed) =
            fix_of(&engine(), "if s.name ~= 'abc'\n    x = 1;\nend\n", "STCMP").unwrap();
        assert_eq!(fixed, "if ~strcmp(s.name, 'abc')\n    x = 1;\nend\n");
        let (_, fixed) = fix_of(&engine(), "while a(1) == 'ab' && t\nend\n", "STCMP").unwrap();
        assert_eq!(fixed, "while strcmp(a(1), 'ab') && t\nend\n");
    }

    #[test]
    fn stcmp_fixed_code_no_longer_reports() {
        let fixed = crate::test_util::fixed_all(&engine(), "if s ~= 'abc' || 1\nend\n");
        assert_eq!(fixed, "if ~strcmp(s, 'abc') || 1\nend\n");
        assert_eq!(stcmp(&fixed), 0);
    }

    #[test]
    fn stcmp_offers_no_fix_for_compound_operands() {
        for source in ["if s(1:2) + t == 'abc'\nend\n", "if a + b == 'abc'\nend\n"] {
            assert!(reported_without_fix(&engine(), source, "STCMP"), "{source}");
        }
        // A negation needs a context in which `~` binds the call alone.
        assert!(reported_without_fix(
            &engine(),
            "if s ~= 'abc' == t\nend\n",
            "STCMP"
        ));
    }

    /// STCMP diagnostics for every comparison in `source`.
    fn stcmp(source: &str) -> usize {
        let tree = parse(source);
        let eng = engine();
        let mut count = 0;
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            count += eng.check_stcmp(n, source).len();
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        count
    }

    #[test]
    fn test_stcmp_fires_on_short_circuit_operand() {
        // Probed: an operand of `||` or `&&` is reported, even in an assignment.
        for source in [
            "a = s == 'abc' || 1;\n",
            "b = s == 'abc' && t;\n",
            "if t || s == 'abc'\nend\n",
            "e = (s == 'abc') || t;\n",
            "while s == 'abc' && t\n    break\nend\n",
        ] {
            assert_eq!(stcmp(source), 1, "{source}");
        }
    }

    #[test]
    fn test_stcmp_silent_on_elementwise_plain_and_double_quoted() {
        // Probed: `&` and `|` operands, plain assignments, negations, matrices,
        // double-quoted strings and single-character literals are silent.
        for source in [
            "c = s == 'abc' & t;\n",
            "d = s == 'abc' | t;\n",
            "f = s == 'abc';\n",
            "k = ~(s == 'abc');\n",
            "g = [s == 'abc'];\n",
            "m = s == \"abc\" || 1;\n",
            "n = s == 'a' || 1;\n",
        ] {
            assert_eq!(stcmp(source), 0, "{source}");
        }
    }
}
