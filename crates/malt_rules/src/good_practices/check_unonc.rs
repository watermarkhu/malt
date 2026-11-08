use super::*;

impl GoodPracticesEngine {
    /// UNONC: `onCleanup` output must be assigned to a variable; `~` is not
    /// permitted in place of a variable.
    pub(crate) fn check_unonc(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("UNONC") || node.kind() != "function_call" {
            return Vec::new();
        }
        if get_function_call_name(node, source) != Some("onCleanup") {
            return Vec::new();
        }

        let parent = match node.parent() {
            Some(p) => p,
            None => return Vec::new(),
        };

        // Bare statement fires; an assignment fires only when `~` is used.
        let tilde_used = if parent.kind() == "assignment" {
            parent
                .child_by_field_name("left")
                .map(|lhs| {
                    lhs.kind() == "ignored_argument"
                        || ((lhs.kind() == "multioutput_variable" || lhs.kind() == "matrix")
                            && has_child_of_kind(lhs, "ignored_argument"))
                })
                .unwrap_or(false)
        } else {
            true
        };

        if !tilde_used {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "UNONC",
            message:
                "Assign the onCleanup output argument to a variable. Do not use the tilde operator (~) in place of a variable."
                    .to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
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
    fn test_unonc_fires_on_bare_call() {
        let source = "onCleanup(@f);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_unonc(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "UNONC");
    }

    #[test]
    fn test_unonc_fires_on_tilde_assignment() {
        let source = "[~] = onCleanup(@f);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_unonc(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "UNONC");
    }

    #[test]
    fn test_unonc_silent_when_assigned() {
        let source = "h = onCleanup(@f);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        let diags = eng.check_unonc(fc, source);
        assert!(diags.is_empty());
    }
}
