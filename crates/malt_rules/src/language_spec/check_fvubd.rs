//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVUBD: a default value refers to an argument that the arguments blocks
    /// declare after it. The argument is known only once it is declared, so the
    /// reference is an error at the reference itself.
    pub(crate) fn check_fvubd(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVUBD") {
            return;
        }
        let mut props = Vec::new();
        for block in blocks {
            props.extend(block_properties(block.node));
        }
        let declared: Vec<(&str, usize)> = props
            .iter()
            .filter_map(|prop| {
                let name = prop.child_by_field_name("name")?;
                Some((node_text(name, source), prop.start_byte()))
            })
            .collect();
        for prop in &props {
            let Some(default) = find_child_kind(*prop, "default_value") else {
                continue;
            };
            let mut stack = vec![default];
            while let Some(node) = stack.pop() {
                if node.kind() == "identifier" && is_variable_reference(node) {
                    let text = node_text(node, source);
                    let later = declared
                        .iter()
                        .any(|(name, start)| *name == text && *start > prop.start_byte());
                    if later {
                        self.push_diag(
                            node,
                            "FVUBD",
                            "Argument is referenced before it is declared in the arguments block.",
                            diagnostics,
                        );
                    }
                }
                let mut cursor = node.walk();
                stack.extend(node.children(&mut cursor));
            }
        }
    }
}

/// False for the field of `a.field`, which is not a variable. A call name is a
/// reference too: `y(end)` indexes the argument `y`, and a variable shadows a
/// function of the same name in MATLAB.
fn is_variable_reference(node: tree_sitter::Node) -> bool {
    let Some(parent) = node.parent() else {
        return true;
    };
    !(parent.kind() == "field_expression"
        && parent
            .child_by_field_name("field")
            .is_some_and(|n| n.id() == node.id()))
}

#[cfg(test)]
mod tests {
    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn fvubd_fires_on_default_naming_a_later_argument() {
        // MATLAB probe: FVUBD at the `y` in the default (line 3, column 22).
        let source = "function r = f(x, y)\narguments\n    x double = numel(y)\n    y double\nend\nr = x;\nend\n";
        let diags = check_source(source, "f.m");
        let fvubd = filter_by_id(&diags, "FVUBD");
        assert_eq!(fvubd.len(), 1, "got: {diags:?}");
        assert_eq!((fvubd[0].line, fvubd[0].column), (3, 22));
    }

    #[test]
    fn fvubd_fires_on_an_index_of_a_later_argument() {
        // MATLAB probe: FVUBD at the `y` of `y(end)` (line 3, column 16).
        let source = "function r = f(x, y)\narguments\n    x double = y(end)\n    y double\nend\nr = x;\nend\n";
        let diags = check_source(source, "f.m");
        let fvubd = filter_by_id(&diags, "FVUBD");
        assert_eq!(
            (fvubd.len(), fvubd.first().map(|d| (d.line, d.column))),
            (1, Some((3, 16))),
            "got: {diags:?}"
        );
    }

    #[test]
    fn fvubd_ok_for_default_naming_an_earlier_argument_or_a_function() {
        let source = "function r = f(x, y)\narguments\n    x double = 1\n    y double = max(x, 2)\nend\nr = y;\nend\n";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVUBD").is_empty(), "got: {diags:?}");
    }
}
