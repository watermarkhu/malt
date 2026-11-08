//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// GPFST: Global/persistent must precede first use.
    pub(crate) fn check_gpfst(
        &self,
        symbol_table: &SymbolTable,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for scope in &symbol_table.scopes {
            // Find all global/persistent declarations
            let gp_defs: Vec<&_> = scope
                .defs
                .iter()
                .filter(|d| d.kind == DefKind::Global || d.kind == DefKind::Persistent)
                .collect();

            for gp_def in &gp_defs {
                // Check if there are any uses of this variable before the declaration
                let first_use = scope
                    .uses
                    .iter()
                    .filter(|u| u.name == gp_def.name)
                    .min_by_key(|u| u.byte_range.start);

                if let Some(first_use) = first_use {
                    if first_use.byte_range.start < gp_def.byte_range.start {
                        diagnostics.push(Diagnostic {
                            rule_id: "GPFST",
                            message: "A GLOBAL or PERSISTENT declaration must precede first use."
                                .to_string(),
                            severity: Severity::Error,
                            byte_range: gp_def.byte_range.clone(),
                            line: gp_def.line,
                            column: gp_def.column,
                            fix: None,
                        });
                    }
                }

                // Also check if there are any assignment definitions before the declaration
                let prior_assignment = scope.defs.iter().find(|d| {
                    d.name == gp_def.name
                        && d.kind == DefKind::Assignment
                        && d.byte_range.start < gp_def.byte_range.start
                });
                if let Some(prior) = prior_assignment {
                    let _ = prior;
                    diagnostics.push(Diagnostic {
                        rule_id: "GPFST",
                        message: "A GLOBAL or PERSISTENT declaration must precede first use."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: gp_def.byte_range.clone(),
                        line: gp_def.line,
                        column: gp_def.column,
                        fix: None,
                    });
                }
            }
        }
    }
}
