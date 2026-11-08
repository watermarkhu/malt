use super::*;

impl GoodPracticesEngine {
    /// MNUML: to create a square matrix, pass both dimensions or use `size`.
    pub(crate) fn check_mnuml(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("MNUML") || node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match get_function_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };
        if !["zeros", "ones", "rand", "randn", "false", "true"].contains(&func_name) {
            return Vec::new();
        }

        let args = match find_child_of_kind(node, "arguments") {
            Some(a) => a,
            None => return Vec::new(),
        };

        let mut named = Vec::new();
        let mut cursor = args.walk();
        for child in args.children(&mut cursor) {
            if child.is_named() {
                named.push(child);
            }
        }
        if named.len() != 1 {
            return Vec::new();
        }
        let arg = named[0];
        if arg.kind() != "function_call" {
            return Vec::new();
        }
        if get_function_call_name(arg, source) != Some("numel") {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "MNUML",
            message: format!(
                "To create a square matrix, use {func_name}(numel(...), numel(...)). Alternatively, use {func_name}(size(...)) to create an array with same size as input array."
            ),
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
    fn test_mnuml_fires_on_single_numel_arg() {
        let source = "y = zeros(numel(x));\n";
        let tree = parse(source);
        let eng = engine();
        let nodes = all_descendants_of_kind(tree.root_node(), "function_call");
        let diags: Vec<Diagnostic> = nodes
            .iter()
            .flat_map(|n| eng.check_mnuml(*n, source))
            .collect();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "MNUML");
    }

    #[test]
    fn test_mnuml_silent_on_multiple_args() {
        let source = "y = zeros(numel(x), 1);\n";
        let tree = parse(source);
        let eng = engine();
        let nodes = all_descendants_of_kind(tree.root_node(), "function_call");
        let diags: Vec<Diagnostic> = nodes
            .iter()
            .flat_map(|n| eng.check_mnuml(*n, source))
            .collect();
        assert!(diags.is_empty());
    }

    #[test]
    fn test_mnuml_silent_on_literal_args() {
        let source = "y = zeros(3, 3);\n";
        let tree = parse(source);
        let eng = engine();
        let nodes = all_descendants_of_kind(tree.root_node(), "function_call");
        let diags: Vec<Diagnostic> = nodes
            .iter()
            .flat_map(|n| eng.check_mnuml(*n, source))
            .collect();
        assert!(diags.is_empty());
    }
}
