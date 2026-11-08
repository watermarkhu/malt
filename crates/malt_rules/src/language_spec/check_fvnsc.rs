//! See the parent module `super` for the shared engine and helpers.

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
                let mut calls = Vec::new();
                collect_function_calls(prop, &mut calls);
                for call in calls {
                    let Some(callee) = callee_name(call, source) else {
                        continue;
                    };
                    if nested_function_names.iter().any(|n| n == &callee) {
                        self.push_diag(
                            call,
                            "FVNSC",
                            "Use of nested functions is not supported in arguments blocks.",
                            diagnostics,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

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
