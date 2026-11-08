//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// VTPOD: Validation order must be: size, class, then functions.
    ///
    /// Since the parser extracts validation components in source order,
    /// the ordering is inherently encoded in the struct. The real check
    /// requires inspecting the raw AST node order for edge cases.
    /// TODO: Implement detailed AST-level ordering check for edge cases.
    pub(crate) fn check_vtpod(&self, _meta: &FileMeta, _diagnostics: &mut Vec<Diagnostic>) {
        // Currently a no-op stub. The tree-sitter grammar enforces the
        // basic ordering. A future implementation will check for degenerate
        // cases where user annotations conflict with MATLAB's expected order.
    }
}
