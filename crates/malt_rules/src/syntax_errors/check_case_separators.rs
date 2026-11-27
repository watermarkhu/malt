//! `check_case_separators`: SEPEXR for a `case` value followed by a statement
//! on the same line, as in `case 1 disp(1)` or `case 1 end`.

use super::*;

impl SyntaxErrorsEngine {
    /// SEPEXR: the value of a `case` clause is followed on its line by a token
    /// with no separator. The grammar parses these lines as errors; MATLAB
    /// reports SEPEXR at the token after the value (probed), and no SYNER.
    pub(crate) fn check_case_separators(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("SEPEXR") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
            if node.kind() != "case_clause" {
                continue;
            }
            let mut cursor = node.walk();
            let Some(value) = node
                .named_children(&mut cursor)
                .find(|child| child.kind() != "comment" && !child.is_missing())
            else {
                continue;
            };
            // Blanks only: a newline, `;`, `,`, a comment or `...` ends the case value.
            // The ERROR node may end right where the next token starts, so no blank is needed.
            let start = value.end_byte()
                + source[value.end_byte()..]
                    .bytes()
                    .take_while(|&b| b == b' ' || b == b'\t')
                    .count();
            let starts_statement = source[start..].chars().next().is_some_and(|c| {
                c.is_alphanumeric() || matches!(c, '_' | '\'' | '"' | '(' | '{' | '[' | '@' | '~')
            });
            if !starts_statement {
                continue;
            }
            let before = &source[..start];
            let line = before.matches('\n').count() + 1;
            let column = start - before.rfind('\n').map_or(0, |newline| newline + 1) + 1;
            diagnostics.push(Diagnostic {
                rule_id: "SEPEXR",
                message: "Use a newline, semicolon, or comma before this statement.".to_string(),
                severity: Severity::Error,
                byte_range: start..start + 1,
                line,
                column,
                fix: None,
            });
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::lint_file;
    use malt_core::Config;

    /// The SEPEXR and SYNER findings of the syntax engine on `source`.
    fn findings(source: &str) -> Vec<Diagnostic> {
        let engine = SyntaxErrorsEngine::from_config(&Config::default());
        lint_file(&*engine, source)
            .into_iter()
            .filter(|d| matches!(d.rule_id, "SEPEXR" | "SYNER"))
            .collect()
    }

    #[test]
    fn sepexr_is_reported_after_a_case_value_without_syner() {
        // Probed: MATLAB reports SEPEXR at `disp` (3:10), `y` (2:18), `disp` (2:23)
        // and `end` (2:17 in `case 1 end`), and no SYNER on those lines.
        for (src, line, column) in [
            ("x = 1;\nswitch x\n  case 1 disp(1)\nend\n", 3, 10),
            ("x = 1;\nswitch x, case 1 y = 1; end\n", 2, 18),
            ("x = 1;\nswitch x, case {1, 2} disp(1), end\n", 2, 23),
        ] {
            let found = findings(src);
            assert_eq!(found.len(), 1, "{src:?}: {found:?}");
            assert_eq!(found[0].rule_id, "SEPEXR", "{src:?}");
            assert_eq!((found[0].line, found[0].column), (line, column), "{src:?}");
        }
    }

    #[test]
    fn silent_for_case_clauses_with_separators_or_a_body_on_its_own_line() {
        for src in [
            "x = 1;\nswitch x\n  case 1\n    disp(1)\nend\n",
            "x = 1;\nswitch x\n  case 1, disp(1)\nend\n",
            "x = 1;\nswitch x\n  case 1; disp(1)\nend\n",
            "x = 1;\nswitch x, case 1, disp(1), end\n",
        ] {
            assert!(findings(src).is_empty(), "{src:?}: {:?}", findings(src));
        }
    }

    #[test]
    fn colon_chain_in_parentheses_has_no_syner() {
        // Probed: MATLAB reports M3COL, not SYNER, for `u = (1:2:3:4);`.
        assert!(findings("u = (1:2:3:4);\n").is_empty());
        assert!(findings("u = [1:2:3:4];\n").is_empty());
        assert!(findings("u = (1:2:3);\n").is_empty());
    }

    #[test]
    fn sepexr_one_line_case_in_function_matches_matlab() {
        // Probed (S4): `case 1 y = 1;` in a function is SEPEXR at `y` (3:12), with no SYNER
        // on the following `otherwise y = 2;` line (MATLAB reports SEPEX there, not SEPEXR).
        let found = findings(
            "function s4(x)\nswitch x\n    case 1 y = 1;\n    otherwise y = 2;\nend\nend\n",
        );
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].rule_id, "SEPEXR");
        assert_eq!((found[0].line, found[0].column), (3, 12));
        // Probed (S5): the comma form `case 1, y = 1; end` is silent.
        assert!(findings("function s5(x)\nswitch x\n    case 1, y = 1; end\nend\n").is_empty());
    }
}
