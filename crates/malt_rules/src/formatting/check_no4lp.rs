//! NO4LP check: parentheses around the iterator of a `for` statement.

use super::*;

// ---------------------------------------------------------------------------
// NO4LP: Parentheses are not needed in a FOR statement
// ---------------------------------------------------------------------------

impl FormattingEngine {
    /// Flag `for (i = 1:n)`: the parentheses around the loop header are
    /// redundant. `parfor` is left alone (its parentheses also delimit the
    /// worker options).
    ///
    /// The diagnostic sits on the opening parenthesis. The fix removes both
    /// parentheses, but only when what follows the closing one cannot be
    /// glued onto the header (`for (i = 1:3) disp(i), end` is left unfixed).
    pub(crate) fn check_no4lp(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.child(0).map(|c| c.kind()) != Some("for") {
            return;
        }
        let mut open = None;
        let mut close = None;
        for i in 0..node.child_count() {
            let Some(child) = node.child(i) else {
                continue;
            };
            match child.kind() {
                "(" if open.is_none() => open = Some(child),
                ")" if open.is_some() && close.is_none() => close = Some(child),
                "block" | "end" => break,
                _ => {}
            }
        }
        let (Some(open), Some(close)) = (open, close) else {
            return;
        };

        let pos = open.start_position();
        let rest = &source[close.end_byte()..];
        let line_rest = rest.split(['\n', '\r']).next().unwrap_or("").trim_start();
        let separable = line_rest.is_empty() || line_rest.starts_with(['%', ';', ',']);
        // `for(i = 1:3)`: the keyword and the header need a blank between them.
        let glued = source[..open.start_byte()]
            .chars()
            .next_back()
            .is_some_and(|c| !c.is_whitespace());

        let fix = separable.then(|| {
            let (primary_replacement, close_fix) = (
                if glued { " " } else { "" },
                Fix::new(close.start_byte()..close.end_byte(), ""),
            );
            Fix::with_additional(
                open.start_byte()..open.end_byte(),
                primary_replacement,
                vec![close_fix],
            )
        });

        diagnostics.push(Diagnostic {
            rule_id: "NO4LP",
            message: "Parentheses are not needed in a FOR statement.".to_string(),
            severity: Severity::Info,
            byte_range: open.start_byte()..close.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix,
        });
    }
}

#[cfg(test)]
mod tests {

    use crate::formatting::tests::{has_id, lint};

    // -- NO4LP ---------------------------------------------------------------

    fn fix_no4lp(source: &str) -> String {
        let diags: Vec<_> = lint(source)
            .into_iter()
            .filter(|d| d.rule_id == "NO4LP")
            .collect();
        malt_core::apply_fixes(source, &diags)
    }

    #[test]
    fn no4lp_parenthesised_for_header() {
        let diags = lint("for (i = 1:3)\n    disp(i);\nend\n");
        let d = diags.iter().find(|d| d.rule_id == "NO4LP").expect("NO4LP");
        assert_eq!((d.line, d.column), (1, 5), "got: {diags:?}");
    }

    #[test]
    fn no4lp_plain_for_header_is_fine() {
        let diags = lint("for i = 1:3\n    disp(i);\nend\n");
        assert!(!has_id(&diags, "NO4LP"), "got: {diags:?}");
    }

    #[test]
    fn no4lp_parentheses_inside_the_range_are_fine() {
        let diags = lint("for i = (1:3)\n    disp(i);\nend\n");
        assert!(!has_id(&diags, "NO4LP"), "got: {diags:?}");
    }

    #[test]
    fn no4lp_indentation_is_not_this_check() {
        // NO4LP used to be an indentation check; MATLAB has no such check.
        let diags = lint("function f()\nfor i = 1:10\nx = i;\nend\nend\n");
        assert!(!has_id(&diags, "NO4LP"), "got: {diags:?}");
    }

    #[test]
    fn no4lp_parfor_is_left_alone() {
        let diags = lint("parfor (i = 1:3, 4)\n    disp(i);\nend\n");
        assert!(!has_id(&diags, "NO4LP"), "got: {diags:?}");
    }

    #[test]
    fn no4lp_fix_removes_the_parentheses() {
        assert_eq!(
            fix_no4lp("for (i = 1:3)\n    disp(i);\nend\n"),
            "for i = 1:3\n    disp(i);\nend\n"
        );
        assert_eq!(
            fix_no4lp("for(i = 1:3)\n    disp(i);\nend\n"),
            "for i = 1:3\n    disp(i);\nend\n"
        );
    }

    #[test]
    fn no4lp_no_fix_when_the_body_shares_the_line() {
        let source = "for (i = 1:3) disp(i), end\n";
        let diags = lint(source);
        let d = diags.iter().find(|d| d.rule_id == "NO4LP").expect("NO4LP");
        assert!(d.fix.is_none(), "got: {d:?}");
    }
}
