//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// Ordering, duplication, and consistency checks for arguments blocks.
    pub(crate) fn check_fv_ordering(
        &self,
        func_meta: &FunctionMeta,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let input_props: Vec<tree_sitter::Node> = blocks
            .iter()
            .filter(|b| b.role.is_input() || b.is_input_repeating())
            .flat_map(|b| block_properties(b.node))
            .collect();

        self.check_fvdap(&input_props, source, diagnostics);
        self.check_fvdnf(&input_props, source, diagnostics);
        self.check_fvdan(&input_props, source, diagnostics);
        self.check_fvniv(func_meta, &input_props, source, diagnostics);
        self.check_fvordi(blocks, source, diagnostics);
        self.check_fvordn(&input_props, source, diagnostics);
        self.check_fvordp(blocks, source, diagnostics);
        self.check_fvordo(blocks, source, diagnostics);
        self.check_fvidv(blocks, source, diagnostics);
        self.check_fvsor(func_meta, blocks, source, diagnostics);
        self.check_fvsoro(func_meta, blocks, source, diagnostics);
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_fvapn_no_fire_for_arguments_block_name_value() {
        // MATLAB reports FVORDN for a name-value declaration before a positional one.
        let source = "\
function f(opts, b)
    arguments
        opts.Name = 'x'
        b (1,1) double
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVAPN").is_empty());
    }
}
