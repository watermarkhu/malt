//! See the parent module `super` for the shared engine and helpers.

use super::*;

/// The input declarations of the input `arguments` blocks, in order, as (name, property node).
///
/// Names follow `block_declaration_sequence`: `~` for an ignored argument.
fn block_input_elements<'a>(
    blocks: &[ArgumentsBlockMeta<'a>],
    source: &str,
) -> Vec<(String, tree_sitter::Node<'a>)> {
    let mut elements: Vec<(String, tree_sitter::Node<'a>)> = Vec::new();
    for block in blocks {
        if !(block.role.is_input() || block.is_input_repeating()) {
            continue;
        }
        for prop in block_properties(block.node) {
            let name = if prop_has_ignored(prop) {
                Some("~".to_string())
            } else if prop_has_name_value(prop) {
                prop_name_value_struct(prop, source)
            } else {
                prop_plain_name(prop, source)
            };
            let Some(name) = name else {
                continue;
            };
            if elements.last().is_some_and(|(last, _)| *last == name) {
                continue;
            }
            elements.push((name, prop));
        }
    }
    elements
}

/// The `index`-th input of the function line that the arguments blocks belong to.
fn function_input_node<'a>(
    blocks: &[ArgumentsBlockMeta<'a>],
    index: usize,
) -> Option<tree_sitter::Node<'a>> {
    let function = blocks.first()?.node.parent()?;
    let function_arguments = find_child_kind(function, "function_arguments")?;
    function_arguments.named_child(index)
}

impl LanguageSpecEngine {
    /// FVSOR: Input arguments block declarations must match the function line.
    ///
    /// The input names of the input `arguments` blocks must equal the function's
    /// input list, in order, including ignored `~` arguments. A partial
    /// declaration is reported too. The diagnostic points at the first element
    /// that differs: the block declaration at that position or, when the blocks
    /// stop early, the function-line input they leave out.
    pub(crate) fn check_fvsor(
        &self,
        func_meta: &FunctionMeta,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVSOR") {
            return;
        }
        let elements = block_input_elements(blocks, source);
        if elements.is_empty() {
            return;
        }
        let inputs = &func_meta.inputs;
        let mismatch = (0..elements.len().max(inputs.len())).find(|&i| {
            elements.get(i).map(|(name, _)| name.as_str()) != inputs.get(i).map(String::as_str)
        });
        let Some(index) = mismatch else {
            return;
        };

        let node = match elements.get(index) {
            Some((_, node)) => *node,
            None => match function_input_node(blocks, index) {
                Some(node) => node,
                None => blocks[0].node,
            },
        };
        let pos = node.start_position();
        diagnostics.push(Diagnostic {
            rule_id: "FVSOR",
            message: "Input arguments block declarations and the function line must contain the same input arguments in the same order, including ignored arguments.".to_string(),
            severity: Severity::Error,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        });
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvsor_fires_order_mismatch() {
        let source = "\
function f(a, b)
    arguments
        b (1,1) double
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        let fvsor = filter_by_id(&diags, "FVSOR");
        assert_eq!(fvsor.len(), 1, "got: {diags:?}");
        // MATLAB points at the first block declaration that differs (`b`, line 3).
        assert_eq!((fvsor[0].line, fvsor[0].column), (3, 9));
    }

    #[test]
    fn test_fvsor_no_fire_matching_order() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVSOR").is_empty(),
            "FVSOR should NOT fire when block declarations match the function line"
        );
    }

    #[test]
    fn test_fvsor_fires_partial_declaration_at_function_line() {
        // Probed with checkcode: `function fvf(a, b)` with only `a` declared
        // reports FVSOR at the function-line input `b` (line 1, column 15).
        let source = "\
function f(a, b)
    arguments
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        let fvsor = filter_by_id(&diags, "FVSOR");
        assert_eq!(fvsor.len(), 1, "got: {diags:?}");
        assert_eq!((fvsor[0].line, fvsor[0].column), (1, 15));
    }

    #[test]
    fn test_fvsor_no_fire_matching_ignored_argument() {
        let source = "\
function f(~, b)
    arguments
        ~
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVSOR").is_empty(),
            "FVSOR should NOT fire when an ignored argument is declared as ~"
        );
    }
}
