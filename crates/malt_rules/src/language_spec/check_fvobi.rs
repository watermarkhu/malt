//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVOBI: Declare all input argument blocks before all output arguments blocks.
    pub(crate) fn check_fvobi(
        &self,
        blocks: &[ArgumentsBlockMeta],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVOBI") {
            return;
        }
        let mut saw_output = false;
        for block in blocks {
            if block.role.is_output() || block.is_output_repeating() {
                saw_output = true;
            } else if saw_output && (block.role.is_input() || block.is_input_repeating()) {
                self.push_block_diag(
                    block,
                    "FVOBI",
                    "Declare all input argument blocks before all output arguments blocks.",
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
    fn test_fvobi_fires_output_before_input() {
        let source = "\
function y = f(x)
    arguments (Output)
        y (1,1) double
    end
    arguments (Input)
        x (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVOBI").is_empty(),
            "FVOBI should fire when an output block precedes an input block"
        );
    }

    #[test]
    fn test_fvobi_no_fire_input_before_output() {
        let source = "\
function y = f(x)
    arguments (Input)
        x (1,1) double
    end
    arguments (Output)
        y (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVOBI").is_empty(),
            "FVOBI should NOT fire when input blocks precede output blocks"
        );
    }
}
