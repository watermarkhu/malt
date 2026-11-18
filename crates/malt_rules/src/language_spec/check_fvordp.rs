//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVORDP: Positional arguments must be ordered required, optional, repeating.
    pub(crate) fn check_fvordp(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVORDP") {
            return;
        }
        let mut saw_optional = false;
        let mut saw_repeating = false;
        for block in blocks {
            if !(block.role.is_input() || block.is_input_repeating()) {
                continue;
            }
            let block_repeating = block.is_input_repeating();
            for prop in block_properties(block.node) {
                if prop_has_ignored(prop) || prop_has_name_value_src(prop, source) {
                    continue;
                }
                if prop_plain_name(prop, source).is_none() {
                    continue;
                }
                // Only a Repeating block makes the later positionals repeating. A varargin in a
                // plain block is an ordinary positional here (MATLAB reports it at FVVREP).
                if block_repeating {
                    saw_repeating = true;
                    continue;
                }
                let is_optional = prop_has_default(prop);
                if saw_repeating || (!is_optional && saw_optional) {
                    self.push_diag(
                        prop,
                        "FVORDP",
                        "Positional arguments must be defined in the following order: required, optional, and repeating.",
                        diagnostics,
                    );
                }
                if is_optional {
                    saw_optional = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvordp_fires_positional_after_repeating_block() {
        // Probed: a plain positional declared after a Repeating block (MATLAB: line 6).
        let source = "\
function fvvrep12(varargin, x)
arguments (Repeating)
    varargin
end
arguments
    varargin
end
end
";
        let diags = check_source(source, "f.m");
        let fvordp = filter_by_id(&diags, "FVORDP");
        assert_eq!(fvordp.len(), 1, "got: {diags:?}");
        assert_eq!((fvordp[0].line, fvordp[0].column), (6, 5));
    }

    #[test]
    fn test_fvordp_no_fire_varargin_in_plain_block() {
        // Probed: a varargin in a plain block is an ordinary positional; MATLAB reports FVVREP
        // only (fvordp16).
        let source = "\
function fvordp16(varargin, b)
arguments
    varargin
    b (1,1) double
end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVORDP").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_fvordp_no_fire_after_name_value_reports_fvordn() {
        // Probed: MATLAB reports FVORDN at `c` (line 5) and no FVORDP: one ordering error per list.
        let source = "\
function fvordp14(a, b, c)
arguments
    a = 1
    opts.Name = 2
    c (1,1) double
end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVORDP").is_empty(), "got: {diags:?}");
        let fvordn = filter_by_id(&diags, "FVORDN");
        assert_eq!(fvordn.len(), 1, "got: {diags:?}");
        assert_eq!(fvordn[0].line, 5);
    }

    #[test]
    fn test_fvordp_fires_optional_before_required() {
        let source = "\
function f(a, b)
    arguments
        b = 3
        a (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVORDP").is_empty(),
            "FVORDP should fire when an optional positional precedes a required one"
        );
    }

    #[test]
    fn test_fvordp_no_fire_required_before_optional() {
        let source = "\
function f(a, b)
    arguments
        a (1,1) double
        b = 3
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVORDP").is_empty(),
            "FVORDP should NOT fire when required positionals precede optional ones"
        );
    }
}
