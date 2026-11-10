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
        if !is_condition_operand(node) {
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
            fix: None,
        }]
    }
}

/// Whether `node` is part of the condition of an `if`, `elseif` or `while`.
fn is_condition_operand(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "if_statement" | "elseif_clause" | "while_statement" => return true,
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
