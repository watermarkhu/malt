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
            let mut cursor = attrs.walk();
            let children: Vec<tree_sitter::Node> = attrs.children(&mut cursor).collect();
            for attr in children {
                if attr.kind() == "identifier"
                    && !matches!(node_text(attr, source), "Input" | "Output" | "Repeating")
                {
                    self.push_diag(attr, "ATUNK", "Unknown attribute name.", diagnostics);
                }
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
