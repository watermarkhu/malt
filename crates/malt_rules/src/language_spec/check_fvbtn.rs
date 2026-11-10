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
                    if BANNED_ARGUMENTS_FUNCTIONS.iter().any(|f| *f == callee) {
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
        if !is_callee && BANNED_ARGUMENTS_FUNCTIONS.contains(&name) {
            out.push(node);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_banned_identifiers(child, source, out);
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

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
    fn test_fvbtn_fires_banned_function() {
        let source = "\
function f(a)
    arguments
        a (1,1) double {mustBeReal(eval('x'))}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVBTN").is_empty(),
            "FVBTN should fire for eval in an arguments block"
        );
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
