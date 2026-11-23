//! See the parent module `super` for the shared engine and helpers.

use super::check_fvbtn::{inside_validator_arguments, is_field_name};
use super::*;

impl LanguageSpecEngine {
    /// FVNSC: Calling nested functions is not supported in arguments blocks.
    pub(crate) fn check_fvnsc(
        &self,
        blocks: &[ArgumentsBlockMeta],
        nested_function_names: &[String],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVNSC") {
            return;
        }
        if nested_function_names.is_empty() {
            return;
        }
        for block in blocks {
            for prop in block_properties(block.node) {
                let mut found = Vec::new();
                let mut cursor = prop.walk();
                for child in prop.children(&mut cursor) {
                    if matches!(child.kind(), "default_value" | "validation_functions") {
                        collect_nested_references(child, source, nested_function_names, &mut found);
                    }
                }
                for node in found {
                    self.push_diag(
                        node,
                        "FVNSC",
                        "Use of nested functions is not supported in arguments blocks.",
                        diagnostics,
                    );
                }
            }
        }
    }
}

/// Collect calls and bare references to nested functions in a default value or validator.
/// A nested call inside a validator's arguments is FVVCON in MATLAB, not FVNSC.
fn collect_nested_references<'a>(
    node: tree_sitter::Node<'a>,
    source: &str,
    names: &[String],
    out: &mut Vec<tree_sitter::Node<'a>>,
) {
    match node.kind() {
        "function_call" => {
            if let Some(callee) = callee_name(node, source) {
                if names.contains(&callee) && !inside_validator_arguments(node) {
                    out.push(node);
                }
            }
        }
        "identifier" => {
            let name = &source[node.byte_range()];
            // The callee of a call is handled by the call; a field name is not a reference.
            let is_callee = node.parent().is_some_and(|p| {
                p.kind() == "function_call" && p.start_byte() == node.start_byte()
            });
            if !is_callee
                && !is_field_name(node)
                && names.iter().any(|n| n == name)
                && !inside_validator_arguments(node)
            {
                out.push(node);
            }
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_nested_references(child, source, names, out);
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvnsc_fires_bare_nested_name() {
        // MATLAB reports a nested function named without parentheses (probed).
        let source = "\
function f(a)
    arguments
        a = helper
        b {helper}
    end
    disp(a)
    function y = helper(x)
        y = x;
    end
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "FVNSC").len(), 2, "got: {diags:?}");
    }

    #[test]
    fn test_fvnsc_no_fire_nested_call_in_validator_arguments() {
        // MATLAB reports the nested call inside `mustBeNumeric(...)` as FVVCON (probed).
        let source = "\
function f(a)
    arguments
        a (1,1) double {mustBeNumeric(helper(3))}
    end
    disp(a)
    function y = helper(x)
        y = x;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVNSC").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_fvnsc_fires_nested_function_call() {
        let source = "\
function outer(a)
    arguments
        a (1,1) double = helper(3)
    end
    b = helper(4);
    function y = helper(x)
        y = x * 2;
    end
end
";
        let diags = check_source(source, "outer.m");
        assert!(
            !filter_by_id(&diags, "FVNSC").is_empty(),
            "FVNSC should fire for a nested function call in an arguments block"
        );
    }

    #[test]
    fn test_fvnsc_no_fire_builtin_call() {
        let source = "\
function f(a)
    arguments
        a (1,1) double = rand(3)
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVNSC").is_empty(),
            "FVNSC should NOT fire for a builtin function call in an arguments block"
        );
    }
}
