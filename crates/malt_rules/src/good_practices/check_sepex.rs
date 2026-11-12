use super::*;

/// Parents whose body may start on the line of the condition. `catch` is an
/// anchor like `else`: MATLAB flags the first token after `catch` when a
/// statement follows it on the same line (probed: `catch disp('e')`,
/// `catch ME disp(ME)`, `catch ME x = 2`), and not `catch ME` alone.
const CONDITIONAL_PARENTS: &[&str] = &[
    "if_statement",
    "elseif_clause",
    "while_statement",
    "for_statement",
    "parfor_statement",
    "catch_clause",
    // `else` and `otherwise` take no condition: the keyword is the anchor (probed).
    "else_clause",
    "otherwise_clause",
];

impl GoodPracticesEngine {
    /// SEPEX: A statement follows a condition on the same line without a
    /// separator, as in `if x y = 1; end`, `switch x case 1` or `if x end`.
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
            } else {
                diagnostics.extend(missing_separator_before(node, source));
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
    let parent = block.parent()?;
    if !CONDITIONAL_PARENTS.contains(&parent.kind()) {
        return None;
    }
    // The node right before the body is the condition. A separator token (`,`
    // or `;`) is not a named node: nothing is missing then.
    let condition = block.prev_sibling()?;
    let keyword_anchor = matches!(condition.kind(), "else" | "otherwise" | "catch");
    if !keyword_anchor && (!condition.is_named() || condition.kind() == "comment") {
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

    // `catch ME x = 2`: MATLAB flags the catch variable, not the statement after it.
    let flagged = if condition.kind() == "identifier" && parent.kind() == "catch_clause" {
        condition
    } else {
        first
    };
    let pos = flagged.start_position();
    Some(Diagnostic {
        rule_id: "SEPEX",
        message:
            "Consider using newline, semicolon, or comma before this statement for readability."
                .to_string(),
        severity: Severity::Warning,
        byte_range: flagged.byte_range(),
        line: pos.row + 1,
        column: pos.column + 1,
        fix: Some(Fix::insert(condition.end_byte(), ",")),
    })
}

/// SEPEX for a header followed on the same line, with only blanks between, by a
/// `case` or `otherwise` clause or by `end`: `switch x case 1` (flagged at the
/// clause), `if x end`, `for k = 1:3 end` and `try, catch end` (flagged at `end`).
/// Probed: MATLAB reports each of these, and reports nothing when a `,` or `;`
/// separates them.
fn missing_separator_before(node: Node, source: &str) -> Option<Diagnostic> {
    // `anchor_end` is where the header ends; `flagged` is the token MATLAB reports.
    let (anchor_end, flagged) = match node.kind() {
        "switch_statement" => {
            let condition = node.child_by_field_name("condition")?;
            // The clause node may start inside the `case` value (ERROR nodes), so
            // the keyword is found in the text.
            let rest = &source[condition.end_byte()..];
            let blanks = rest.len() - rest.trim_start_matches([' ', '\t']).len();
            let start = condition.end_byte() + blanks;
            let keyword = ["case", "otherwise"]
                .into_iter()
                .find(|keyword| source[start..].starts_with(keyword))?;
            (condition.end_byte(), start..start + keyword.len())
        }
        "if_statement" | "while_statement" | "for_statement" | "parfor_statement" => {
            let mut cursor = node.walk();
            let named: Vec<Node> = node
                .named_children(&mut cursor)
                .filter(|child| child.kind() != "comment")
                .collect();
            // A body or a further clause means the header is not alone on its line.
            if named
                .iter()
                .any(|child| matches!(child.kind(), "block" | "elseif_clause" | "else_clause"))
            {
                return None;
            }
            let end = end_keyword(node, source)?;
            (named.last()?.end_byte(), end)
        }
        "try_statement" => {
            let mut cursor = node.walk();
            let catch = node
                .named_children(&mut cursor)
                .find(|child| child.kind() == "catch_clause")?;
            let mut cursor = catch.walk();
            if catch
                .named_children(&mut cursor)
                .any(|child| child.kind() != "comment")
            {
                return None;
            }
            // The catch clause node can run on past the keyword (over a newline or
            // a comma), so the header ends after `catch` itself.
            (
                catch.start_byte() + "catch".len(),
                end_keyword(node, source)?,
            )
        }
        _ => return None,
    };

    let gap = &source[anchor_end..flagged.start];
    if flagged.start < anchor_end || !gap.chars().all(|c| c == ' ' || c == '\t') {
        return None;
    }
    let (line, column) = line_and_column(source, flagged.start);
    Some(Diagnostic {
        rule_id: "SEPEX",
        message:
            "Consider using newline, semicolon, or comma before this statement for readability."
                .to_string(),
        severity: Severity::Warning,
        byte_range: flagged.clone(),
        line,
        column,
        fix: Some(Fix::insert(anchor_end, ",")),
    })
}

/// The byte range of the `end` keyword that closes `node`, if the node ends with one.
fn end_keyword(node: Node, source: &str) -> Option<std::ops::Range<usize>> {
    let end = node.end_byte();
    (source.get(end.checked_sub(3)?..end) == Some("end")).then(|| end - 3..end)
}

/// The 1-based line and column of a byte offset.
fn line_and_column(source: &str, byte: usize) -> (usize, usize) {
    let before = &source[..byte];
    let line = before.matches('\n').count() + 1;
    let column = byte - before.rfind('\n').map_or(0, |newline| newline + 1) + 1;
    (line, column)
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
    fn fires_after_else_and_otherwise_keywords() {
        // Probed: MATLAB reports SEPEX for `else a = 1` and `otherwise a = 2`,
        // and not for the comma forms (bugs__t1:28).
        assert_eq!(sepex("if x, a = 1; else a = 1; end\n").len(), 1);
        assert_eq!(
            sepex("switch x, case 1, a = 1; otherwise a = 2; end\n").len(),
            1
        );
        assert!(sepex("if x, a = 1; else, a = 1; end\n").is_empty());
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
    fn fires_for_a_statement_after_catch_on_the_same_line() {
        // Probed: MATLAB flags the first token after `catch` (column 7 or 19 here).
        for (src, line, column) in [
            ("try\n  a = 1;\ncatch disp('e')\nend\n", 3, 7),
            ("try\n  a = 1;\ncatch err disp(err)\nend\n", 3, 7),
            ("try\n  a = 1;\ncatch fprintf('a')\nend\n", 3, 7),
            ("try, a = 1; catch ME disp(ME); end\n", 1, 19),
            ("try\n  a = 1;\ncatch ME x = 2;\nend\n", 3, 7),
        ] {
            let found = sepex(src);
            assert_eq!(found.len(), 1, "{src:?}");
            assert_eq!((found[0].line, found[0].column), (line, column), "{src:?}");
        }
    }

    #[test]
    fn silent_for_catch_without_a_statement_on_its_line() {
        for src in [
            "try\n  a = 1;\ncatch ME\n  disp(ME)\nend\n",
            "try\n  a = 1;\ncatch ME, disp(ME)\nend\n",
            "try\n  a = 1;\ncatch ME; disp(ME)\nend\n",
            "try\n  a = 1;\ncatch\n  disp(1)\nend\n",
            "try\n  a = 1;\ncatch disp\nend\n",
        ] {
            assert!(sepex(src).is_empty(), "{src:?}: {:?}", sepex(src));
        }
    }

    #[test]
    fn fires_for_a_case_clause_after_switch_on_the_same_line() {
        // Probed: MATLAB flags `case` (column 10) after `switch x` with no separator.
        let found = sepex("x = 1;\nswitch x case 1 y = 1; end\n");
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].line, found[0].column), (2, 10));
        assert!(sepex("x = 1;\nswitch x, case 1 y = 1; end\n").is_empty());
    }

    #[test]
    fn fires_for_end_on_the_header_line() {
        // Probed: `if x end` (column 6), `while x end` (9), `for k = 1:3 end` (13),
        // `try x = 1; catch end` (18). The `end` is flagged, not the header.
        for (src, column) in [
            ("x = 1;\nif x end\n", 6),
            ("x = 1;\nwhile x end\n", 9),
            ("for k = 1:3 end\n", 13),
            ("try x = 1; catch end\n", 18),
        ] {
            let found = sepex(src);
            assert_eq!(found.len(), 1, "{src:?}");
            assert_eq!(found[0].column, column, "{src:?}");
        }
    }

    #[test]
    fn silent_for_end_after_a_separator_or_on_its_own_line() {
        for src in [
            "x = 1;\nif x, end\n",
            "for k = 1:3, end\n",
            "x = 1;\nif x\nend\n",
            "try, catch, end\n",
        ] {
            assert!(sepex(src).is_empty(), "{src:?}: {:?}", sepex(src));
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
            ("x = 1;\nif x end\n", "x = 1;\nif x, end\n"),
            (
                "x = 1;\nswitch x case 1 end\n",
                "x = 1;\nswitch x, case 1 end\n",
            ),
            (
                "try\n  a = 1;\ncatch disp('e')\nend\n",
                "try\n  a = 1;\ncatch, disp('e')\nend\n",
            ),
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
