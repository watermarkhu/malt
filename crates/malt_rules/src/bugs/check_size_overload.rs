use super::*;

impl BugsEngine {
    /// DEFSIZE: a user-defined function named `size` overloads the builtin
    /// `size` for fundamental data types.
    ///
    /// # Limitations (heuristic)
    ///
    /// This flags any free function named `size`; a legitimate `size` method on
    /// a user-defined class is correctly excluded (guarded by
    /// `!is_inside_class_method`), but no static analysis verifies that the
    /// free function actually overloads the builtin for fundamental types — the
    /// name match alone triggers the diagnostic.
    pub(crate) fn check_size_overload(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        Self::walk_size_overload(root, source, &mut diagnostics);
        diagnostics
    }

    fn walk_size_overload(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "function_definition" {
            if let Some(name_node) = node.child_by_field_name("name") {
                // A `size` method defined on a user-defined class is a legal
                // overload; only a free function shadows the builtin.
                if node_text(name_node, source).trim() == "size" && !is_inside_class_method(node) {
                    let pos = name_node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "DEFSIZE",
                        message: "Do not overload 'size' for fundamental data types.".to_string(),
                        severity: Severity::Error,
                        byte_range: name_node.start_byte()..name_node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_size_overload(child, source, diagnostics);
        }
    }
}

/// Whether a `function_definition` sits inside a class `methods` block.
fn is_inside_class_method(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "methods" => return true,
            "class_definition" | "source_file" => return false,
            _ => {}
        }
        current = parent.parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- DEFSIZE -------------------------------------------------------------

    #[test]
    fn defsize_fires_on_size_function_definition() {
        let src = "\
function y = size(x)
    y = 1;
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "DEFSIZE"), "got: {diags:?}");
    }

    #[test]
    fn defsize_fires_on_multioutput_size() {
        let src = "\
function [m, n] = size(x)
    m = 1;
    n = 2;
end
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "DEFSIZE"), "got: {diags:?}");
    }

    #[test]
    fn defsize_no_fire_on_other_function() {
        let src = "\
function y = mysize(x)
    y = 1;
end
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "DEFSIZE"), "got: {diags:?}");
    }

    #[test]
    fn defsize_no_fire_on_size_usage() {
        let src = "y = size(x) == [1 2];\n";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "DEFSIZE"), "got: {diags:?}");
    }
}
