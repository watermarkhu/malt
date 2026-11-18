//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVORDI: Ignored input arguments are not allowed after a Repeating
    /// arguments block or name-value arguments.
    pub(crate) fn check_fvordi(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVORDI") {
            return;
        }
        // The declaration before an ignored argument decides: MATLAB reports a `~` right after a
        // name-value argument (default or bare) or a Repeating block, and is silent when a
        // positional comes in between (ls-b__f3.m:6, fvordi05, fvordi07). An ignored argument
        // in a Repeating block is silent.
        #[derive(Clone, Copy, PartialEq)]
        enum Previous {
            Other,
            NameValue,
            Repeating,
        }
        let mut previous = Previous::Other;
        for block in blocks {
            if !(block.role.is_input() || block.is_input_repeating()) {
                continue;
            }
            for prop in block_properties(block.node) {
                if block.is_input_repeating() {
                    previous = Previous::Repeating;
                } else if prop_has_ignored(prop) {
                    if previous != Previous::Other {
                        self.push_diag(
                            prop,
                            "FVORDI",
                            "Ignored input arguments are not allowed after a Repeating arguments block or name-value arguments.",
                            diagnostics,
                        );
                    }
                } else if prop_has_name_value_src(prop, source) {
                    previous = Previous::NameValue;
                } else {
                    previous = Previous::Other;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvordi_fires_ignored_after_bare_name_value() {
        // Probed: a bare `opts.A` followed by `~` reports the `~` (MATLAB line 4).
        let source = "\
function fvordn04(opts, ~)
arguments
    opts.A
    ~
end
end
";
        let diags = check_source(source, "f.m");
        let fvordi = filter_by_id(&diags, "FVORDI");
        assert_eq!(fvordi.len(), 1, "got: {diags:?}");
        assert_eq!(fvordi[0].line, 4);
    }

    #[test]
    fn test_fvordi_fires_ignored_after_class_name_value() {
        // Probed: the class form `opts.?pkg.Class = 1` counts as a name-value argument.
        let source = "\
function fvordi13(opts, ~)
arguments
    opts.?pkg.Class = 1
    ~
end
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "FVORDI").len(), 1, "got: {diags:?}");
    }

    #[test]
    fn test_fvordi_reports_only_the_first_ignored_argument() {
        // Probed: MATLAB reports the first `~` after the name-value argument (line 4) only.
        let source = "\
function fvordi16(opts, ~)
arguments
    opts.Name = 1
    ~
    ~
end
end
";
        let diags = check_source(source, "f.m");
        let fvordi = filter_by_id(&diags, "FVORDI");
        assert_eq!(fvordi.len(), 1, "got: {diags:?}");
        assert_eq!(fvordi[0].line, 4);
    }

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

    #[test]
    fn test_fvordi_no_fire_ignored_after_bare_name_value() {
        // MATLAB: `opts.A` has no default, so the trailing `~` is silent (ls-b__f3.m:6).
        let source = "\
function f3(opts, x, ~)
    arguments
        opts.A
        opts.B
        x
        ~
    end
end
";
        let diags = check_source(source, "f3.m");
        assert!(
            filter_by_id(&diags, "FVORDI").is_empty(),
            "FVORDI should NOT fire after a name-value argument without a default"
        );
    }
}
