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
        let mut nv_fields: Vec<String> = Vec::new();
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
                if let Some(field) = prop_name_value_field(prop, source) {
                    if !nv_fields.contains(&field) {
                        nv_fields.push(field);
                    }
                }
            }
        }

        self.check_fvtinvaldim_ttoofewdims(blocks, source, diagnostics);
        self.check_fvbtn(blocks, source, diagnostics);
        self.check_fvnsc(blocks, nested_function_names, source, diagnostics);
        self.check_fvond(blocks, &nv_structs, source, diagnostics);
        self.check_fvonv(blocks, &nv_fields, source, diagnostics);
        self.check_fvvin_fvvcon_fvubd(blocks, source, diagnostics);
        self.check_fvvin_fvocon(blocks, source, diagnostics);
    }
}
