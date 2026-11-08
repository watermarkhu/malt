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
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

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
