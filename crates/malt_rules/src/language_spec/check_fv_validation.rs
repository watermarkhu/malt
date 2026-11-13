//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// Validation function and size constraint checks for arguments blocks.
    pub(crate) fn check_fv_validation(
        &self,
        blocks: &[ArgumentsBlockMeta],
        nested_function_names: &[String],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut nv_structs: Vec<String> = Vec::new();
        for block in blocks {
            for prop in block_properties(block.node) {
                if !prop_has_name_value(prop) {
                    continue;
                }
                if let Some(struct_name) = prop_name_value_struct(prop, source) {
                    if !nv_structs.contains(&struct_name) {
                        nv_structs.push(struct_name);
                    }
                }
            }
        }

        self.check_block_attribute_names(blocks, source, diagnostics);
        self.check_fvtinvaldim_ttoofewdims(blocks, source, diagnostics);
        self.check_fvbtn(blocks, source, diagnostics);
        self.check_fvnsc(blocks, nested_function_names, source, diagnostics);
        self.check_fvond(blocks, &nv_structs, source, diagnostics);
        self.check_fvvin_fvvcon(blocks, source, diagnostics);
        self.check_fvvin_fvocon(blocks, source, diagnostics);
    }

    /// ATUNK for function arguments blocks: only `Input`, `Output` and `Repeating` are attributes.
    fn check_block_attribute_names(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for block in blocks {
            let Some(attrs) = find_child_kind(block.node, "attributes") else {
                continue;
            };
            // Only attribute names are checked: a value after `=` (`Input = ~true`) is not.
            let text = node_text(attrs, source);
            let body_start = text.find('(').map_or(0, |i| i + 1);
            let body_end = text.rfind(')').unwrap_or(text.len());
            let mut offset = body_start;
            for part in text[body_start..body_end].split(',') {
                let lead = part.len() - part.trim_start().len();
                let name = part.trim().split('=').next().unwrap_or_default().trim_end();
                // A dotted name (`pkg.Attr`) is unknown too (probed).
                if !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
                    && !matches!(name, "Input" | "Output" | "Repeating")
                {
                    let start = attrs.start_byte() + offset + lead;
                    self.push_diag_bytes(
                        source,
                        start,
                        start + name.len(),
                        "ATUNK",
                        "Unknown attribute name.",
                        diagnostics,
                    );
                }
                offset += part.len() + 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_atunk_fires_unknown_arguments_attribute() {
        let source = "\
function f(x)
    arguments (Input, Foo)
        x
    end
end
";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "ATUNK").len(), 1);
    }

    #[test]
    fn test_atunk_fires_unknown_name_after_value() {
        // MATLAB checks the attribute name after the comma, not the value before it (probed).
        let source = "\
function f(x)
    arguments (Input = true, Foo)
        x
    end
end
";
        let diags = check_source(source, "f.m");
        let atunk = filter_by_id(&diags, "ATUNK");
        assert_eq!(atunk.len(), 1, "got: {diags:?}");
        assert_eq!(atunk[0].column, 30);
    }

    #[test]
    fn test_atunk_no_fire_value_expression() {
        // `true` after `=` is an attribute value, not a name (probed: `Input = ~true` is silent).
        let source = "\
function f(x)
    arguments (Input = ~true)
        x
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "ATUNK").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_atunk_no_fire_known_arguments_attributes() {
        let source = "\
function f(x)
    arguments (Input, Repeating)
        x
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "ATUNK").is_empty());
    }
}
