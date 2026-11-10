//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVSORO: Output arguments block declarations must match the function line.
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
        let sequence = block_declaration_sequence(blocks, source, true);
        if sequence.is_empty() {
            return;
        }
        if !is_prefix_of(&sequence, &func_meta.outputs) {
            if let Some(block) = blocks
                .iter()
                .find(|b| b.role.is_output() || b.is_output_repeating())
            {
                let first = block.args.first().map_or("", |a| a.name.as_str());
                self.push_arg_diag(
                    block,
                    first,
                    "FVSORO",
                    "Output arguments block declarations and the function line must contain the same output arguments in the same order.",
                    source,
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
