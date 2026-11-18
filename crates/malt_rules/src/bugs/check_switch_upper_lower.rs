use super::*;

impl BugsEngine {
    /// MULCC: a `switch` on `upper(...)`/`lower(...)` whose case label uses a
    /// case that the conversion can never produce, so it cannot be matched.
    pub(crate) fn check_switch_upper_lower(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "switch_statement" {
            return Vec::new();
        }

        let cond = match node.child_by_field_name("condition") {
            Some(c) => c,
            None => return Vec::new(),
        };
        if cond.kind() != "function_call" {
            return Vec::new();
        }
        let name = match extract_call_name(cond, source) {
            Some(n) => n,
            None => return Vec::new(),
        };
        if name != "upper" && name != "lower" {
            return Vec::new();
        }
        let is_upper = name == "upper";

        let mut diagnostics = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "case_clause" {
                continue;
            }
            let case_val = match child.child_by_field_name("condition") {
                Some(v) => v,
                None => continue,
            };
            if case_val.kind() != "string" {
                continue;
            }
            let inner = node_text(case_val, source).trim_matches(|c| c == '\'' || c == '"');
            let unmatchable = if is_upper {
                inner.chars().any(|c| c.is_ascii_lowercase())
            } else {
                inner.chars().any(|c| c.is_ascii_uppercase())
            };
            if unmatchable {
                let pos = case_val.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "MULCC",
                    message: "This case cannot be matched due to a call to UPPER or LOWER on the SWITCH value.".to_string(),
                    severity: Severity::Error,
                    byte_range: case_val.start_byte()..case_val.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- MULCC ---------------------------------------------------------------

    #[test]
    fn mulcc_fires_on_lowercase_case_with_upper_switch() {
        let src = "\
switch upper(x)
    case 'abc'
        a = 1;
end
";
        let diags = node_diags(src);
        assert!(has_id(&diags, "MULCC"), "got: {diags:?}");
    }

    #[test]
    fn mulcc_fires_on_uppercase_case_with_lower_switch() {
        let src = "\
switch lower(x)
    case 'ABC'
        a = 1;
end
";
        let diags = node_diags(src);
        assert!(has_id(&diags, "MULCC"), "got: {diags:?}");
    }

    #[test]
    fn mulcc_no_fire_on_matching_case() {
        let src = "\
switch upper(x)
    case 'ABC'
        a = 1;
end
";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "MULCC"), "got: {diags:?}");
    }

    #[test]
    fn mulcc_no_fire_without_upper_lower() {
        let src = "\
switch x
    case 'abc'
        a = 1;
end
";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "MULCC"), "got: {diags:?}");
    }
}
