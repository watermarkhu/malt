//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVSOR: Input arguments block declarations must match the function line.
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
        let sequence = block_declaration_sequence(blocks, source, false);
        if sequence.is_empty() {
            return;
        }
        if !is_prefix_of(&sequence, &func_meta.inputs) {
            if let Some(block) = blocks
                .iter()
                .find(|b| b.role.is_input() || b.is_input_repeating())
            {
                self.push_block_diag(
                    block,
                    "FVSOR",
                    "Input arguments block declarations and the function line must contain the same input arguments in the same order, including ignored arguments.",
                    diagnostics,
                );
            }
        }
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
        assert!(
            !filter_by_id(&diags, "FVSOR").is_empty(),
            "FVSOR should fire when block declarations do not match the function line"
        );
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
    fn test_fvsor_no_fire_partial_declaration() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVSOR").is_empty(),
            "FVSOR should NOT fire when the block declares a prefix of the inputs"
        );
    }
}
