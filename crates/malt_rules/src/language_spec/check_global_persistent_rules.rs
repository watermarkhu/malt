//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// Check global/persistent ordering rules.
    pub(crate) fn check_global_persistent_rules(
        &self,
        symbol_table: &SymbolTable,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.check_gpfst(symbol_table, diagnostics);
        self.check_gpnes(ctx, diagnostics);
    }
}
