//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVOOD / FVOOI / FVOON: output arguments blocks cannot declare defaults,
    /// ignored arguments, or name-value arguments.
    pub(crate) fn check_fvood_fvooi_fvoon(
        &self,
        blocks: &[ArgumentsBlockMeta],
        _source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let ood = self.is_check_enabled("FVOOD");
        let ooi = self.is_check_enabled("FVOOI");
        let oon = self.is_check_enabled("FVOON");
        if !ood && !ooi && !oon {
            return;
        }
        for block in blocks.iter().filter(|b| b.role.is_output()) {
            for prop in block_properties(block.node) {
                if ood && prop_has_default(prop) {
                    self.push_diag(
                        prop,
                        "FVOOD",
                        "Specifying a default value for an output argument is not supported.",
                        diagnostics,
                    );
                }
                if ooi && prop_has_ignored(prop) {
                    self.push_diag(
                        prop,
                        "FVOOI",
                        "Use of ignored arguments in output arguments block is not supported.",
                        diagnostics,
                    );
                }
                if oon && prop_has_name_value(prop) {
                    self.push_diag(
                        prop,
                        "FVOON",
                        "Using name-value argument as output argument is not supported.",
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
    fn test_fvood_fires_output_default() {
        let source = "\
function y = f(x)
    arguments (Output)
        y (1,1) double = 5
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVOOD").is_empty(),
            "FVOOD should fire for a default value on an output argument"
        );
    }

    #[test]
    fn test_fvood_no_fire_output_without_default() {
        let source = "\
function y = f(x)
    arguments (Output)
        y (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVOOD").is_empty(),
            "FVOOD should NOT fire for an output argument without a default"
        );
    }

    #[test]
    fn test_fvooi_fires_ignored_in_output() {
        let source = "\
function [a, ~] = f(x)
    arguments (Output)
        a (1,1) double
        ~
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVOOI").is_empty(),
            "FVOOI should fire for an ignored argument in an output block"
        );
    }

    #[test]
    fn test_fvooi_no_fire_clean_output() {
        let source = "\
function [a] = f(x)
    arguments (Output)
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVOOI").is_empty(),
            "FVOOI should NOT fire for a clean output block"
        );
    }

    #[test]
    fn test_fvoon_fires_name_value_output() {
        let source = "\
function [a] = f(x)
    arguments (Output)
        opts.Name
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVOON").is_empty(),
            "FVOON should fire for a name-value argument in an output block"
        );
    }

    #[test]
    fn test_fvoon_no_fire_positional_output() {
        let source = "\
function [a] = f(x)
    arguments (Output)
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVOON").is_empty(),
            "FVOON should NOT fire for a positional output argument"
        );
    }
}
