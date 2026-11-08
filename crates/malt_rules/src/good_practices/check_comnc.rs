use super::*;

impl GoodPracticesEngine {
    /// COMNC: a `%` comment that follows a comma on the same line.
    ///
    /// A `%` comment after a comma acts as a row separator and can silently
    /// change the shape of a matrix/cell expression.
    pub(crate) fn check_comnc(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("COMNC") || node.kind() != "comment" {
            return Vec::new();
        }

        let text = node_text(node, source);

        // Ignore block comments (%{ ... %}) and pragma comments (%#...).
        if text.starts_with("%{") || text.starts_with("%#") || text.starts_with("%%") {
            return Vec::new();
        }

        // Fire only when the comment follows a comma on the same line.
        let Some(comma_byte) = comma_before_comment(node, source) else {
            return Vec::new();
        };

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "COMNC",
            message: "Comment with percent (%) following comma acts as a row separator. Replace the comma with a semicolon to make the row separation clearer. Alternatively, replace the percent (%) with an ellipsis (...) to add a comment inside a row.".to_string(),
            severity: Severity::Info,
            byte_range: node.start_byte()..node.start_byte() + 2,
            line: pos.row + 1,
            column: pos.column + 1,
            fix: Some(malt_core::Fix::new(
                comma_byte..comma_byte + 1,
                ";",
            )),
        }]
    }
}

/// Find the byte offset of a comma immediately preceding a comment on the same
/// line (skipping only whitespace). Returns `None` when the character before
/// the comment is not a comma.
fn comma_before_comment(node: Node, source: &str) -> Option<usize> {
    let start = node.start_byte();
    let bytes = source.as_bytes();
    let line_start = source[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);

    let mut i = start;
    while i > line_start {
        i -= 1;
        match bytes[i] as char {
            ',' => return Some(i),
            c if c.is_whitespace() => continue,
            _ => return None,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comnc_fires_on_comment_after_comma() {
        let source = "a = [1, 2, % comment\n 3];\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comment = find_descendant_of_kind(root, "comment").unwrap();
        let diags = eng.check_comnc(comment, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "COMNC");
    }

    #[test]
    fn test_comnc_silent_on_standalone_comment() {
        let source = "x = 1;\n% comment on its own line\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comment = find_descendant_of_kind(root, "comment").unwrap();
        let diags = eng.check_comnc(comment, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_comnc_silent_on_comment_after_semicolon() {
        let source = "x = 1; % comment after semicolon\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let comment = find_descendant_of_kind(root, "comment").unwrap();
        let diags = eng.check_comnc(comment, source);
        assert!(diags.is_empty());
    }
}
