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

        // Probed: `H = onCleanup(...)` as a property default is silent, so only
        // calls in code are checked.
        let mut ancestor = node.parent();
        while let Some(a) = ancestor {
            if a.kind() == "properties" {
                return Vec::new();
            }
            ancestor = a.parent();
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

    fn unonc_lines(source: &str) -> Vec<usize> {
        crate::test_util::lint_nodes(&engine(), source)
            .iter()
            .filter(|d| d.rule_id == "UNONC")
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn test_unonc_silent_for_property_default() {
        // Probed (U1): `H = onCleanup(...)` as a property default is silent.
        let source =
            "classdef U1\n    properties\n        H = onCleanup(@() disp(1));\n    end\nend\n";
        assert!(unonc_lines(source).is_empty());
    }

    #[test]
    fn test_unonc_fires_on_bare_call_in_method_and_script() {
        // Probed (U5, U4): a bare call in a method (line 4) or in a script (line 1) is UNONC.
        assert_eq!(
            unonc_lines(
                "classdef U5\n    methods\n        function f(obj)\n            onCleanup(@() disp(1));\n        end\n    end\nend\n"
            ),
            vec![4]
        );
        assert_eq!(unonc_lines("onCleanup(@() disp(1));\n"), vec![1]);
    }

    #[test]
    fn test_unonc_fires_only_on_bare_call_after_assignment() {
        // Probed (U6): the assigned call (line 2) is silent, the bare call (line 3) is UNONC.
        let source = "function f()\nc = onCleanup(@() disp(1));\nonCleanup(@() disp(2));\nend\n";
        assert_eq!(unonc_lines(source), vec![3]);
        // Probed (U3): an assignment inside a function is silent.
        assert!(unonc_lines("function f()\nH = onCleanup(@() disp(1));\nend\n").is_empty());
    }
}
