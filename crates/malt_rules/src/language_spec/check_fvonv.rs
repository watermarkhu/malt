//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVONV: Name-value arguments must use the dotted name in validation.
    pub(crate) fn check_fvonv(
        &self,
        blocks: &[ArgumentsBlockMeta],
        nv_fields: &[String],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("FVONV") {
            return;
        }
        for block in blocks {
            for prop in block_properties(block.node) {
                for call in prop_validator_calls(prop, source) {
                    let uses_bare_field = call.references.iter().any(|r| {
                        matches!(
                            r,
                            CallReference::Identifier(name)
                                if nv_fields.iter().any(|f| f == name)
                        )
                    });
                    if uses_bare_field {
                        self.push_diag(
                            prop,
                            "FVONV",
                            "Use of name-value arguments without dotted name in the validation is not supported.",
                            diagnostics,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvonv_fires_bare_name_value_field() {
        let source = "\
function f(opts, y)
    arguments
        opts.Name {mustBeReal}
        y (1,1) double {mustBeEqual(y, Name)}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVONV").is_empty(),
            "FVONV should fire when validation uses a name-value field without the dotted name"
        );
    }

    #[test]
    fn test_fvonv_no_fire_dotted_name_value() {
        let source = "\
function f(opts)
    arguments
        opts.Name {mustBeMember(opts.Name, {'a','b'})}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVONV").is_empty(),
            "FVONV should NOT fire when validation uses the dotted name-value name"
        );
    }
}
