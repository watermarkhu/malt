use super::*;

impl BugsEngine {
    /// STCUL: `strcmp` (case-sensitive) comparison whose two string arguments
    /// differ only by case, so it will likely fail.
    pub(crate) fn check_strcmp_case_mismatch(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match extract_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };

        if func_name != "strcmp" {
            return Vec::new();
        }

        let args = collect_call_args(node, source);
        if args.len() < 2 {
            return Vec::new();
        }

        let a = args[0].trim();
        let b = args[1].trim();

        let a_is_literal = is_string_literal(a);
        let b_is_literal = is_string_literal(b);
        if !a_is_literal || !b_is_literal {
            return Vec::new();
        }

        let ai = a.trim_matches(|c| c == '\'' || c == '"');
        let bi = b.trim_matches(|c| c == '\'' || c == '"');

        if ai.eq_ignore_ascii_case(bi) && ai != bi {
            let pos = node.start_position();
            return vec![Diagnostic {
                rule_id: "STCUL",
                message: "The comparison will likely fail due to case mismatch.".to_string(),
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }];
        }

        Vec::new()
    }
}

/// Whether `text` is a string literal (`'...'` or `"..."`).
fn is_string_literal(text: &str) -> bool {
    (text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2)
        || (text.starts_with('"') && text.ends_with('"') && text.len() >= 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- STCUL ---------------------------------------------------------------

    #[test]
    fn stcul_fires_on_case_mismatch() {
        let src = "strcmp('a', 'A');\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "STCUL"), "got: {diags:?}");
    }

    #[test]
    fn stcul_fires_on_multi_char_case_mismatch() {
        let src = "strcmp('abc', 'ABC');\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "STCUL"), "got: {diags:?}");
    }

    #[test]
    fn stcul_no_fire_on_identical_args() {
        let src = "strcmp('abc', 'abc');\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "STCUL"), "got: {diags:?}");
    }

    #[test]
    fn stcul_no_fire_on_different_strings() {
        let src = "strcmp('abc', 'def');\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "STCUL"), "got: {diags:?}");
    }
}
