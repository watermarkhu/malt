//! See the parent module `super` for the shared engine and helpers.

use super::*;

/// Whether a definition is a `global` or `persistent` declaration.
fn is_gp_decl(kind: &DefKind) -> bool {
    matches!(kind, DefKind::Global | DefKind::Persistent)
}

impl LanguageSpecEngine {
    /// GPFST: Global/persistent must precede first use.
    ///
    /// Reported once per name, at the first reference to it in the scope (an
    /// assignment, a loop variable or a read) when that reference comes before
    /// the first `global` or `persistent` declaration of the name. MATLAB points
    /// at that reference, not at the declaration.
    pub(crate) fn check_gpfst(
        &self,
        symbol_table: &SymbolTable,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for scope in &symbol_table.scopes {
            for gp_def in scope.defs.iter().filter(|d| is_gp_decl(&d.kind)) {
                let first_decl = scope
                    .defs
                    .iter()
                    .filter(|d| d.name == gp_def.name && is_gp_decl(&d.kind))
                    .map(|d| d.byte_range.start)
                    .min();
                if first_decl != Some(gp_def.byte_range.start) {
                    continue;
                }

                let first_ref = scope
                    .uses
                    .iter()
                    .filter(|u| u.name == gp_def.name)
                    .map(|u| (u.byte_range.clone(), u.line, u.column))
                    .chain(
                        scope
                            .defs
                            .iter()
                            .filter(|d| d.name == gp_def.name && !is_gp_decl(&d.kind))
                            .map(|d| (d.byte_range.clone(), d.line, d.column)),
                    )
                    .min_by_key(|(range, _, _)| range.start);

                if let Some((range, line, column)) = first_ref {
                    if range.start < gp_def.byte_range.start {
                        diagnostics.push(Diagnostic {
                            rule_id: "GPFST",
                            message: "A GLOBAL or PERSISTENT declaration must precede first use."
                                .to_string(),
                            severity: Severity::Error,
                            byte_range: range,
                            line,
                            column,
                            fix: None,
                        });
                    }
                }
            }
        }
    }
}
