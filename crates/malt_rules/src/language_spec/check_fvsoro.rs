//! See the parent module `super` for the shared engine and helpers.

use super::*;

/// The output declarations of the output `arguments` blocks, in order, as (name, property node).
///
/// A Repeating block without the Output attribute is an input block and is left out. Ignored
/// outputs are left out too: MATLAB does not compare them (fvood05).
fn output_declaration_elements<'a>(
    blocks: &[ArgumentsBlockMeta<'a>],
    source: &str,
) -> Vec<(String, tree_sitter::Node<'a>)> {
    let mut elements: Vec<(String, tree_sitter::Node<'a>)> = Vec::new();
    for block in blocks {
        let repeating_output =
            block.role.is_repeating() && block.attributes.iter().any(|a| a == "Output");
        if !(block.role.is_output() || repeating_output) {
            continue;
        }
        for prop in block_properties(block.node) {
            if prop_has_ignored(prop) {
                continue;
            }
            let name = if prop_has_name_value_src(prop, source) {
                prop_name_value_struct_src(prop, source)
            } else {
                prop_plain_name(prop, source)
            };
            let Some(name) = name else {
                continue;
            };
            if prop_has_name_value_src(prop, source)
                && elements.last().is_some_and(|(last, _)| *last == name)
            {
                continue;
            }
            elements.push((name, prop));
        }
    }
    elements
}

/// The `index`-th output of the function line that the arguments blocks belong to.
fn function_output_node<'a>(
    blocks: &[ArgumentsBlockMeta<'a>],
    index: usize,
) -> Option<tree_sitter::Node<'a>> {
    let function = blocks.first()?.node.parent()?;
    let function_output = find_child_kind(function, "function_output")?;
    let mut identifiers = Vec::new();
    collect_identifier_nodes(function_output, &mut identifiers);
    identifiers.get(index).copied()
}

/// The identifier nodes below `node`, in source order.
fn collect_identifier_nodes<'a>(node: tree_sitter::Node<'a>, out: &mut Vec<tree_sitter::Node<'a>>) {
    if node.kind() == "identifier" {
        out.push(node);
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_identifier_nodes(child, out);
    }
}

impl LanguageSpecEngine {
    /// FVSORO: Output arguments block declarations must match the function line.
    ///
    /// The output names of the output blocks must equal the function's outputs, in order.
    /// The diagnostic points at the first element that differs: the declaration at that
    /// position or, when the blocks stop early, the function-line output they leave out.
    pub(crate) fn check_fvsoro(
        &self,
        func_meta: &FunctionMeta,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVSORO") {
            return;
        }
        let declared = output_declaration_elements(blocks, source);
        if declared.is_empty() {
            return;
        }
        let outputs = &func_meta.outputs;
        let mismatch = (0..declared.len().max(outputs.len())).find(|&i| {
            declared.get(i).map(|(name, _)| name.as_str()) != outputs.get(i).map(String::as_str)
        });
        let Some(index) = mismatch else {
            return;
        };
        let node = match declared.get(index) {
            Some((_, node)) => *node,
            None => match function_output_node(blocks, index) {
                Some(node) => node,
                None => return,
            },
        };
        let pos = node.start_position();
        diagnostics.push(Diagnostic {
            rule_id: "FVSORO",
            message: "Output arguments block declarations and the function line must contain the same output arguments in the same order.".to_string(),
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
    fn test_fvsoro_reports_missing_repeating_output_at_function_line() {
        // Probed: `[a, varargout]` with an Output block for `a` and a Repeating varargout block
        // (MATLAB reports the function line, column 14).
        let source = "\
function [a, varargout] = f(x)
arguments (Output)
    a (1,1) double
end
arguments (Repeating)
    varargout
end
end
";
        let diags = check_source(source, "f.m");
        let fvsoro = filter_by_id(&diags, "FVSORO");
        assert_eq!(fvsoro.len(), 1, "got: {diags:?}");
        assert_eq!((fvsoro[0].line, fvsoro[0].column), (1, 14));
    }

    #[test]
    fn test_fvsoro_no_fire_ignored_output_in_block() {
        // Probed: a `~` in an Output block is not compared; MATLAB reports nothing here.
        let source = "\
function [a, b] = f(x)
arguments (Output)
    a double
    ~
    b double
end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVSORO").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_fvsoro_reported_at_the_first_output_declaration() {
        // Probed: MATLAB points at the first declaration of the output block.
        let source = "\
function [b, a] = f(x)
    arguments (Output)
        a (1,1) double
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        let lines: Vec<usize> = filter_by_id(&diags, "FVSORO")
            .iter()
            .map(|d| d.line)
            .collect();
        assert_eq!(lines, vec![3]);
    }

    #[test]
    fn test_fvsoro_fires_order_mismatch() {
        let source = "\
function [b, a] = f(x)
    arguments (Output)
        a (1,1) double
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVSORO").is_empty(),
            "FVSORO should fire when output block declarations do not match the function line"
        );
    }

    #[test]
    fn test_fvsoro_no_fire_matching_order() {
        let source = "\
function [a, b] = f(x)
    arguments (Output)
        a (1,1) double
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVSORO").is_empty(),
            "FVSORO should NOT fire when output block declarations match the function line"
        );
    }
}
