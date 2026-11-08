//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVNREP / FVREPD: name-value arguments and default values are not
    /// supported in a Repeating arguments block.
    pub(crate) fn check_fvnrep_fvrepd(
        &self,
        blocks: &[ArgumentsBlockMeta],
        _source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let nrep = self.is_check_enabled("FVNREP");
        let repd = self.is_check_enabled("FVREPD");
        if !nrep && !repd {
            return;
        }
        for block in blocks {
            if !block.role.is_repeating() {
                continue;
            }
            for prop in block_properties(block.node) {
                if nrep && prop_has_name_value(prop) {
                    self.push_diag(
                        prop,
                        "FVNREP",
                        "Name-value arguments are not supported in a Repeating arguments block.",
                        diagnostics,
                    );
                }
                if repd && prop_has_default(prop) {
                    self.push_diag(
                        prop,
                        "FVREPD",
                        "Default values are not supported in a Repeating arguments block.",
                        diagnostics,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvnrep_fires_name_value_in_repeating() {
        let source = "\
function f(opts)
    arguments (Repeating)
        opts.Name
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVNREP").is_empty(),
            "FVNREP should fire for a name-value argument in a Repeating block"
        );
    }

    #[test]
    fn test_fvnrep_no_fire_name_value_normal() {
        let source = "\
function f(opts)
    arguments
        opts.Name = 'x'
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVNREP").is_empty(),
            "FVNREP should NOT fire for a name-value argument in a normal block"
        );
    }

    #[test]
    fn test_fvrepd_fires_default_in_repeating() {
        let source = "\
function f(a)
    arguments (Repeating)
        a = 3
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVREPD").is_empty(),
            "FVREPD should fire for a default value in a Repeating block"
        );
    }

    #[test]
    fn test_fvrepd_no_fire_repeating_without_default() {
        let source = "\
function f(a)
    arguments (Repeating)
        a (1,:) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVREPD").is_empty(),
            "FVREPD should NOT fire for a Repeating block without defaults"
        );
    }
}
