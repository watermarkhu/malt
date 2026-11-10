//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// FVNDE / FVNVL: name-value arguments using a class name cannot have
    /// default values or validation functions.
    pub(crate) fn check_fvnde_fvnvl(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let nde = self.is_check_enabled("FVNDE");
        let nvl = self.is_check_enabled("FVNVL");
        if !nde && !nvl {
            return;
        }
        for block in blocks {
            let mut cursor = block.node.walk();
            let errors: Vec<tree_sitter::Node> = block
                .node
                .children(&mut cursor)
                .filter(|child| child.is_error())
                .collect();
            for prop in block_properties(block.node).into_iter().chain(errors) {
                // The grammar splits `opts.?double` into a name and an error node,
                // so the `?` is read from the source text of the property.
                let text = node_text(prop, source);
                let name_len = text
                    .find(|c: char| c.is_whitespace() || matches!(c, '=' | '{' | '('))
                    .unwrap_or(text.len());
                if !text[..name_len].contains('?') {
                    continue;
                }
                let tail = &text[name_len..];
                let head = tail.split('=').next().unwrap_or_default();
                let has_default = prop_has_default(prop) || tail.contains('=');
                let has_validation = prop_has_validation(prop) || head.contains('{');
                if nde && has_default {
                    self.push_diag(
                        prop,
                        "FVNDE",
                        "When specifying name-value arguments using a class name, it is illegal to specify default values for the arguments.",
                        diagnostics,
                    );
                }
                if nvl && has_validation {
                    self.push_diag(
                        prop,
                        "FVNVL",
                        "When specifying name-value arguments using a class name, it is illegal to specify validation for the arguments.",
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
    fn test_fvnde_fires_class_name_default() {
        let source = "\
function f(opts)
    arguments
        opts.?double = 3
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVNDE").is_empty(),
            "FVNDE should fire for a default value on a class-name name-value argument"
        );
    }

    #[test]
    fn test_fvnde_fires_dotted_class_name_default() {
        let source = "\
function f(opts)
    arguments
        opts.?matlab.graphics.chart.primitive.Line = 1
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(!filter_by_id(&diags, "FVNDE").is_empty());
    }

    #[test]
    fn test_fvnde_no_fire_field_named_like_class() {
        // MATLAB: only the `?Class` form is a class name; `opts.double` is a plain name.
        let source = "\
function f(opts)
    arguments
        opts.double = 3
        opts.string (1,1) string = \"a\"
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVNDE").is_empty());
    }

    #[test]
    fn test_fvnde_no_fire_plain_name_value() {
        let source = "\
function f(opts)
    arguments
        opts.Name = 3
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVNDE").is_empty(),
            "FVNDE should NOT fire for a regular name-value argument"
        );
    }

    #[test]
    fn test_fvnvl_fires_class_name_validation() {
        let source = "\
function f(opts)
    arguments
        opts.?double {mustBeReal}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FVNVL").is_empty(),
            "FVNVL should fire for validation on a class-name name-value argument"
        );
    }

    #[test]
    fn test_fvnvl_no_fire_field_named_like_class() {
        let source = "\
function f(opts)
    arguments
        opts.double {mustBeReal}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVNVL").is_empty());
    }

    #[test]
    fn test_fvnvl_no_fire_plain_name_value_validation() {
        let source = "\
function f(opts)
    arguments
        opts.Name {mustBeReal}
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FVNVL").is_empty(),
            "FVNVL should NOT fire for a regular name-value argument with validation"
        );
    }
}
