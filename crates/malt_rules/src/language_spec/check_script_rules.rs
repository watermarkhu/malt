//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// Check script-level rules.
    pub(crate) fn check_script_rules(
        &self,
        meta: &FileMeta,
        symbol_table: &SymbolTable,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if meta.file_type != FileType::Script {
            return;
        }

        // NPERS: Persistent declarations are not allowed in scripts
        self.check_npers(symbol_table, diagnostics);

        // FCONV: Variable name same as script name
        self.check_fconv(symbol_table, ctx, diagnostics);

        // USESWNS: Variable must be explicitly defined before first use
        self.check_useswns(symbol_table, ctx, diagnostics);
    }
}
