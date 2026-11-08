//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVORDI: Ignored input arguments are not allowed after a Repeating
    /// arguments block or name-value arguments.
    pub(crate) fn check_fvordi(
        &self,
        blocks: &[ArgumentsBlockMeta],
        _source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVORDI") {
            return;
        }
        let mut saw_name_value = false;
        let mut saw_repeating = false;
        for block in blocks {
            if !(block.role.is_input() || block.is_input_repeating()) {
                continue;
            }
            if block.is_input_repeating() {
                saw_repeating = true;
            }
            for prop in block_properties(block.node) {
                if prop_has_ignored(prop) {
                    if saw_name_value || saw_repeating {
                        self.push_diag(
                            prop,
                            "FVORDI",
                            "Ignored input arguments are not allowed after a Repeating arguments block or name-value arguments.",
                            diagnostics,
                        );
                    }
                } else if prop_has_name_value(prop) {
                    saw_name_value = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvordi_fires_ignored_after_name_value() {
        let source = "\
function f(opts, ~)
    arguments
        opts.Name = 'x'
        ~
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVORDI").is_empty(),
            "FVORDI should fire for an ignored argument after name-value arguments"
        );
    }

    #[test]
    fn test_fvordi_fires_ignored_after_repeating() {
        let source = "\
function f(varargin, ~)
    arguments (Repeating)
        varargin
    end
    arguments
        ~
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVORDI").is_empty(),
            "FVORDI should fire for an ignored argument after a Repeating block"
        );
    }

    #[test]
    fn test_fvordi_no_fire_ignored_before_name_value() {
        let source = "\
function f(~, opts)
    arguments
        ~
        opts.Name = 'x'
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVORDI").is_empty(),
            "FVORDI should NOT fire for an ignored argument before name-value arguments"
        );
    }
}
