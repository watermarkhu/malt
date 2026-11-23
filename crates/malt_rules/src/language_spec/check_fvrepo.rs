//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVREPO: A repeating input block with varargin must not have other arguments.
    pub(crate) fn check_fvrepo(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVREPO") {
            return;
        }
        for block in blocks {
            if !block.is_input_repeating() {
                continue;
            }
            let has_varargin = block.args.iter().any(|a| a.name == "varargin");
            if has_varargin && block.args.len() > 1 {
                self.push_arg_diag(
                    block,
                    "varargin",
                    "FVREPO",
                    "Repeating input arguments block containing varargin must not have other arguments.",
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
    fn test_fvrepo_reported_at_the_varargin_line() {
        // Probed: MATLAB points at the varargin declaration, not the block header.
        let source = "\
function f(x, varargin)
    arguments (Repeating)
        x
        varargin
    end
end
";
        let diags = check_source(source, "f.m");
        let lines: Vec<usize> = filter_by_id(&diags, "FVREPO")
            .iter()
            .map(|d| d.line)
            .collect();
        assert_eq!(lines, vec![4]);
    }

    #[test]
    fn test_fvrepo_fires_varargin_with_others() {
        let source = "\
function f(varargin, x)
    arguments (Repeating)
        varargin
        x (1,:) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVREPO").is_empty(),
            "FVREPO should fire when a Repeating block with varargin has other arguments"
        );
    }

    #[test]
    fn test_fvrepo_no_fire_varargin_alone() {
        let source = "\
function f(varargin)
    arguments (Repeating)
        varargin
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVREPO").is_empty(),
            "FVREPO should NOT fire when a Repeating block contains only varargin"
        );
    }
}
