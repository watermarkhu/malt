use super::*;

/// Parents whose body may start on the line of the condition. `catch` is left
/// out on purpose: `catch id` followed by a statement cannot be told apart
/// from `catch` followed by a call.
const CONDITIONAL_PARENTS: &[&str] = &[
    "if_statement",
    "elseif_clause",
    "while_statement",
    "for_statement",
    "parfor_statement",
];

impl GoodPracticesEngine {
    /// SEPEX: A statement follows a condition on the same line without a
    /// separator, as in `if x y = 1; end`.
    ///
    /// Statements that are separated (`a = 1; b = 2`, `x = 1;; y = 2`) are fine.
    /// The fix inserts the `,` that separates the condition from the body.
    pub(crate) fn check_sepex(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("SEPEX") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        // Iterative walk: no recursion, so deeply nested input cannot overflow the stack.
        let mut cursor = tree.walk();
        loop {
            let node = cursor.node();
            if node.kind() == "block" {
                diagnostics.extend(missing_separator(node, source));
            }
            if cursor.goto_first_child() {
                continue;
            }
            loop {
                if cursor.goto_next_sibling() {
                    break;
                }
                if !cursor.goto_parent() {
                    return diagnostics;
                }
            }
        }
    }
}

/// The diagnostic for a block body whose first statement shares the line of the
/// condition before it without a separator.
fn missing_separator(block: Node, source: &str) -> Option<Diagnostic> {
    if !CONDITIONAL_PARENTS.contains(&block.parent()?.kind()) {
        return None;
    }
    // The node right before the body is the condition. A separator token (`,`
    // or `;`) or a keyword is not a named node: nothing is missing then.
    let condition = block.prev_sibling()?;
    if !condition.is_named() || condition.kind() == "comment" {
        return None;
    }
    let mut cursor = block.walk();
    let first = block
        .named_children(&mut cursor)
        .find(|child| child.kind() != "comment")?;
    if condition.end_position().row != first.start_position().row {
        return None;
    }
    let gap = &source[condition.end_byte()..first.start_byte()];
    if gap.contains(';') || gap.contains(',') {
        return None;
    }

    let pos = first.start_position();
    Some(Diagnostic {
        rule_id: "SEPEX",
        message:
            "Consider using newline, semicolon, or comma before this statement for readability."
                .to_string(),
        severity: Severity::Warning,
        byte_range: first.byte_range(),
        line: pos.row + 1,
        column: pos.column + 1,
        fix: Some(Fix::insert(condition.end_byte(), ",")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use malt_core::apply_fixes;

    fn sepex(source: &str) -> Vec<Diagnostic> {
        let tree = parse(source);
        engine().check_sepex(&tree, source)
    }

    #[test]
    fn fires_when_a_body_statement_follows_the_condition() {
        for src in [
            "if x y = 1; end\n",
            "while k disp(1); end\n",
            "for i = 1:3 disp(i), end\n",
            "if a, b = 1; elseif c d = 2; end\n",
        ] {
            assert_eq!(sepex(src).len(), 1, "{src:?}");
        }
    }

    #[test]
    fn silent_when_a_separator_is_present() {
        for src in [
            "a = 1; b = 2\n",
            "x = 1;; y = 2;\n",
            "if x, y = 1; end\n",
            "if x; y = 1; end\n",
            "if x\n  y = 1;\nend\n",
            "for i = 1:3, disp(i), end\n",
            "try, a = 1; catch, b = 2; end\n",
            "if x % c\n  y = 1;\nend\n",
        ] {
            assert!(sepex(src).is_empty(), "{src:?}: {:?}", sepex(src));
        }
    }

    #[test]
    fn fix_adds_the_separator_and_keeps_the_code() {
        for (src, expected) in [
            ("if x y = 1; end\n", "if x, y = 1; end\n"),
            ("while k disp(1); end\n", "while k, disp(1); end\n"),
            ("for i = 1:3 disp(i), end\n", "for i = 1:3, disp(i), end\n"),
        ] {
            let fixed = apply_fixes(src, &sepex(src));
            assert_eq!(fixed, expected);
            assert!(sepex(&fixed).is_empty(), "fix must resolve the finding");
        }
    }

    #[test]
    fn reports_the_real_column() {
        let d = &sepex("if x y = 1; end\n")[0];
        assert_eq!((d.line, d.column), (1, 6));
    }
}
