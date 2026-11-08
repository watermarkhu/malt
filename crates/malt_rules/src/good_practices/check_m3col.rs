use super::*;

impl GoodPracticesEngine {
    /// M3COL: an expression with three colons (`a:b:c:d`) is probably
    /// unintended. The grammar cannot represent four-element colon chains, so
    /// they parse as a `range` plus a trailing ERROR node (or a range nested
    /// inside an ERROR node); all three shapes are detected here.
    pub(crate) fn check_m3col(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("M3COL") || node.kind() != "range" {
            return Vec::new();
        }

        let mut colon_count = node_text(node, source).matches(':').count();
        let mut end_byte = node.end_byte();

        if let Some(next) = node.next_named_sibling() {
            if next.kind() == "ERROR" && node_text(next, source).starts_with(':') {
                colon_count += node_text(next, source).matches(':').count();
                end_byte = end_byte.max(next.end_byte());
            }
        }
        if let Some(parent) = node.parent() {
            if parent.kind() == "ERROR" {
                colon_count += node_text(parent, source).matches(':').count();
                end_byte = end_byte.max(parent.end_byte());
            } else if let Some(next) = parent.next_named_sibling() {
                if next.kind() == "ERROR" && node_text(next, source).starts_with(':') {
                    colon_count += node_text(next, source).matches(':').count();
                    end_byte = end_byte.max(next.end_byte());
                }
            }
        }

        if colon_count < 3 {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "M3COL",
            message: "Using three colons (a:b:c:d) in an expression is probably unintended."
                .to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..end_byte,
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_m3col_fires_on_three_colons() {
        let source = "a = 1:2:3:4;\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let range = find_descendant_of_kind(root, "range").unwrap();
        let diags = eng.check_m3col(range, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "M3COL");
        assert_eq!(&source[diags[0].byte_range.clone()], "1:2:3:4");
    }

    #[test]
    fn test_m3col_silent_on_two_colons() {
        let source = "a = 1:2:10;\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let range = find_descendant_of_kind(root, "range").unwrap();
        assert!(eng.check_m3col(range, source).is_empty());
    }

    #[test]
    fn test_m3col_silent_on_one_colon() {
        let source = "a = 1:10;\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let range = find_descendant_of_kind(root, "range").unwrap();
        assert!(eng.check_m3col(range, source).is_empty());
    }

    #[test]
    fn test_m3col_fires_on_parenthesized_three_colons() {
        // `(1:2:3):4` contains three `:` in the range's own text, so it fires.
        let source = "a = (1:2:3):4;\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let mut fired = false;
        for range in all_descendants_of_kind(root, "range") {
            if !eng.check_m3col(range, source).is_empty() {
                fired = true;
            }
        }
        assert!(fired);
    }
}
