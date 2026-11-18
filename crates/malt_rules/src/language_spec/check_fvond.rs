//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVOND: Use of name-value arguments in default values is not supported.
    pub(crate) fn check_fvond(
        &self,
        blocks: &[ArgumentsBlockMeta],
        nv_structs: &[String],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVOND") {
            return;
        }
        // The class form `opts.?pkg.Class = v` declares a name-value structure too.
        let mut names: Vec<String> = nv_structs.to_vec();
        for block in blocks {
            for prop in block_properties(block.node) {
                if !prop_has_name_value_src(prop, source) {
                    continue;
                }
                if let Some(name) = prop_name_value_struct_src(prop, source) {
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
            }
        }
        for block in blocks {
            for prop in block_properties(block.node) {
                let Some(default_node) = find_child_kind(prop, "default_value") else {
                    continue;
                };
                if default_refs_name_value(default_node, source, &names) {
                    // MATLAB points at the first reference to a name-value structure.
                    let reference =
                        first_name_reference(default_node, source, &names).unwrap_or(prop);
                    self.push_diag(
                        reference,
                        "FVOND",
                        "Use of name-value arguments in default values is not supported.",
                        diagnostics,
                    );
                }
            }
        }
    }
}

/// The first identifier below `node` that names one of `names`, in source order.
fn first_name_reference<'a>(
    node: tree_sitter::Node<'a>,
    source: &str,
    names: &[String],
) -> Option<tree_sitter::Node<'a>> {
    if node.kind() == "identifier" && names.iter().any(|n| n.as_str() == node_text(node, source)) {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = first_name_reference(child, source, names) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvond_points_at_reference_in_continuation() {
        // Probed: a default spanning `...` is reported at the line of the reference (line 5,
        // column 9), not at the argument.
        let source = "\
function fvond13(opts, y)
arguments
    opts.Name = 3
    y (1,1) double = ...
        opts.Name
end
end
";
        let diags = check_source(source, "f.m");
        let fvond = filter_by_id(&diags, "FVOND");
        assert_eq!(fvond.len(), 1, "got: {diags:?}");
        assert_eq!((fvond[0].line, fvond[0].column), (5, 9));
    }

    #[test]
    fn test_fvond_fires_name_value_in_default() {
        let source = "\
function f(opts, y)
    arguments
        opts.Name = 3
        y (1,1) double = opts.Name
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVOND").is_empty(),
            "FVOND should fire when a default value uses a name-value argument"
        );
    }

    #[test]
    fn test_fvond_no_fire_default_uses_positional() {
        let source = "\
function f(a, y)
    arguments
        a (1,1) double
        y (1,1) double = a + 1
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVOND").is_empty(),
            "FVOND should NOT fire when a default value uses a positional argument"
        );
    }

    #[test]
    fn test_fvond_no_fire_class_typed_default_call() {
        // Probed: `y pkg.Class = pkg.Class()` is a class name, not a name-value argument.
        let source = "\
function f(a, y)
arguments
    a double
    y pkg.Class = pkg.Class()
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVOND").is_empty(), "got: {diags:?}");
    }
}
