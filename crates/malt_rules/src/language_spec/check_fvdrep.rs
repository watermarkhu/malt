//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVDREP: Multiple Repeating arguments blocks are not supported.
    pub(crate) fn check_fvdrep(
        &self,
        blocks: &[ArgumentsBlockMeta],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVDREP") {
            return;
        }
        let mut seen_repeating = false;
        for block in blocks {
            if !block.role.is_repeating() {
                continue;
            }
            if seen_repeating {
                self.push_block_diag(
                    block,
                    "FVDREP",
                    "Multiple Repeating arguments blocks are not supported.",
                    diagnostics,
                );
            }
            seen_repeating = true;
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvdrep_fires_multiple_repeating() {
        let source = "\
function f(a2, a3)
    arguments (Repeating)
        a2
    end
    arguments (Repeating)
        a3
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVDREP").is_empty(),
            "FVDREP should fire for multiple Repeating blocks"
        );
    }

    #[test]
    fn test_fvdrep_no_fire_single_repeating() {
        let source = "\
function f(a2)
    arguments (Repeating)
        a2
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVDREP").is_empty(),
            "FVDREP should NOT fire for a single Repeating block"
        );
    }
}
