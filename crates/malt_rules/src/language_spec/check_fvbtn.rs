//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVBTN: Banned functions are not supported in arguments blocks.
    pub(crate) fn check_fvbtn(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVBTN") {
            return;
        }
        for block in blocks {
            for prop in block_properties(block.node) {
                let mut calls = Vec::new();
                collect_function_calls(prop, &mut calls);
                for call in calls {
                    let Some(callee) = callee_name(call, source) else {
                        continue;
                    };
                    if BANNED_ARGUMENTS_FUNCTIONS.iter().any(|f| *f == callee)
                        && !inside_validator_arguments(call)
                    {
                        self.push_diag(
                            call,
                            "FVBTN",
                            "Use of this function is not supported in arguments blocks.",
                            diagnostics,
                        );
                    }
                }
                // A banned function used without parentheses, such as `= nargin`
                // or `max(nargin, 2)`, is an identifier, not a call.
                let mut identifiers = Vec::new();
                collect_banned_identifiers(prop, source, &mut identifiers);
                for ident in identifiers {
                    self.push_diag(
                        ident,
                        "FVBTN",
                        "Use of this function is not supported in arguments blocks.",
                        diagnostics,
                    );
                }
            }
        }
    }
}

/// Collect identifiers naming a banned function that are not the callee of a call.
fn collect_banned_identifiers<'a>(
    node: tree_sitter::Node<'a>,
    source: &str,
    out: &mut Vec<tree_sitter::Node<'a>>,
) {
    if node.kind() == "identifier" {
        let is_callee = node
            .parent()
            .is_some_and(|p| p.kind() == "function_call" && p.start_byte() == node.start_byte());
        let name = &source[node.byte_range()];
        if !is_callee
            && !is_field_name(node)
            && BANNED_ARGUMENTS_FUNCTIONS.contains(&name)
            && !inside_validator_arguments(node)
        {
            out.push(node);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_banned_identifiers(child, source, out);
    }
}

/// Whether an identifier is the field of a field access or name-value name (`s.nargin`),
/// which is not a reference to the function.
pub(crate) fn is_field_name(node: tree_sitter::Node) -> bool {
    node.parent().is_some_and(|p| {
        matches!(p.kind(), "field_expression" | "property_name")
            && p.named_child(0)
                .is_some_and(|first| first.id() != node.id())
    })
}

/// Whether `node` is inside the argument list of a validation function call, such as
/// `eval` in `{mustBeReal(eval('1'))}`. MATLAB reports such a name as FVVCON, not as
/// FVBTN. A banned name that is itself the validator (`{narginchk(1,1)}`) is not inside
/// its arguments and is still reported.
pub(crate) fn inside_validator_arguments(node: tree_sitter::Node) -> bool {
    let mut seen_arguments = false;
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "arguments" => seen_arguments = true,
            "validation_functions" => return seen_arguments,
            "arguments_statement" => return false,
            _ => {}
        }
        current = parent.parent();
    }
    false
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvbtn_no_fire_banned_inside_validator_arguments() {
        // MATLAB reports `eval` and `nargout` as arguments of a validator as FVVCON only.
        let source = "\
function f(a, b)
    arguments
        a (1,1) double {mustBeReal(eval('1'))}
        b (1,1) double {mustBeNonempty(nargout)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVBTN").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_fvbtn_no_fire_field_named_like_banned_function() {
        // `s.nargin` is a field access, not a call to nargin (probed).
        let source = "\
function f(a)
    arguments
        a = s.nargin
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVBTN").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_fvbtn_fires_banned_validator_name() {
        // MATLAB reports a banned function used as the validator itself (probed).
        let source = "\
function f(a)
    arguments
        a (1,1) double {narginchk(1,1)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "FVBTN").len(), 1, "got: {diags:?}");
    }

    #[test]
    fn test_fvbtn_fires_banned_identifier_without_parentheses() {
        // MATLAB reports `= nargin` and `max(nargin, 2)` in arguments blocks.
        let source = "\
function r = f(n, m)
    arguments
        n (1,1) {mustBeNumeric} = nargin
        m = max(nargin, 2)
    end
    r = n;
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "FVBTN").len(), 2);
    }

    #[test]
    fn test_fvbtn_silent_eval_inside_validator_call() {
        // MATLAB reports `eval` inside `mustBeReal(...)` as EVLCS and FVVCON, not FVBTN (probed).
        let source = "\
function f(a)
    arguments
        a (1,1) double {mustBeReal(eval('x'))}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVBTN").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_fvbtn_no_fire_validator() {
        let source = "\
function f(a)
    arguments
        a (1,1) double {mustBeReal}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVBTN").is_empty(),
            "FVBTN should NOT fire for ordinary validators"
        );
    }
}
