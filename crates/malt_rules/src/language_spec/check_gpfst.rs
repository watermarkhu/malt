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
                let Some(first) = scope
                    .defs
                    .iter()
                    .filter(|d| d.name == gp_def.name && is_gp_decl(&d.kind))
                    .min_by_key(|d| d.byte_range.start)
                else {
                    continue;
                };
                if first.byte_range.start != gp_def.byte_range.start {
                    // A later declaration conflicts with the first when the kinds differ
                    // or both are persistent (probed: `persistent n` then `global n`, and a
                    // second `persistent n`, give REDEF at the later name, quoting the first
                    // declaration's line). Two globals give REDEFGG, reported elsewhere.
                    let is_input_or_output = scope.defs.iter().any(|d| {
                        d.name == gp_def.name
                            && matches!(d.kind, DefKind::InputArg | DefKind::OutputArg)
                    });
                    let conflicts = first.kind != gp_def.kind || gp_def.kind == DefKind::Persistent;
                    if conflicts && !is_input_or_output && self.is_check_enabled("REDEF") {
                        diagnostics.push(Diagnostic {
                            rule_id: "REDEF",
                            message: format!(
                                "The current use of '{}' is inconsistent with its previous use or definition (line {}).",
                                gp_def.name, first.line
                            ),
                            severity: Severity::Error,
                            byte_range: gp_def.byte_range.clone(),
                            line: gp_def.line,
                            column: gp_def.column,
                            fix: None,
                        });
                    }
                    continue;
                }

                // The name is also a function input or output, which MATLAB treats
                // as a use at the header, not as a first use before the declaration.
                // A persistent declaration of it is REDEF at the name (probed:
                // `persistent P` with output `P` gives REDEF, not GPFST). A global
                // one is REDEFGI, reported by the compatibility engine.
                if let Some(arg) = scope.defs.iter().find(|d| {
                    d.name == gp_def.name
                        && matches!(d.kind, DefKind::InputArg | DefKind::OutputArg)
                }) {
                    if gp_def.kind == DefKind::Persistent && self.is_check_enabled("REDEF") {
                        diagnostics.push(Diagnostic {
                            rule_id: "REDEF",
                            message: format!(
                                "The current use of '{}' is inconsistent with its previous use or definition (line {}).",
                                gp_def.name, arg.line
                            ),
                            severity: Severity::Error,
                            byte_range: gp_def.byte_range.clone(),
                            line: gp_def.line,
                            column: gp_def.column,
                            fix: None,
                        });
                    }
                    continue;
                }

                // PSET: a persistent variable that is read but never assigned in the
                // scope. Probed: any assignment, even a conditional one, silences it,
                // and so does another declaration of the name (`persistent n; global n`
                // gives REDEF only). It is reported at the declaration once per name.
                if self.is_check_enabled("PSET")
                    && matches!(gp_def.kind, DefKind::Persistent)
                    && !scope
                        .defs
                        .iter()
                        .any(|d| d.name == gp_def.name && !is_gp_decl(&d.kind))
                    && !scope.defs.iter().any(|d| {
                        d.name == gp_def.name
                            && is_gp_decl(&d.kind)
                            && d.byte_range.start != gp_def.byte_range.start
                    })
                    && scope.uses.iter().any(|u| u.name == gp_def.name)
                {
                    diagnostics.push(Diagnostic {
                        rule_id: "PSET",
                        message: "Persistent variable is used, but might be unset.".to_string(),
                        severity: Severity::Warning,
                        byte_range: gp_def.byte_range.clone(),
                        line: gp_def.line,
                        column: gp_def.column,
                        fix: None,
                    });
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

#[cfg(test)]
mod tests {
    use crate::language_spec::LanguageSpecEngine;
    use crate::test_util::lint_file;
    use malt_core::Config;

    /// Rule IDs reported for `src`, in report order.
    fn ids(src: &str) -> Vec<&'static str> {
        let e = LanguageSpecEngine::from_config(&Config::default());
        lint_file(&*e, src).iter().map(|d| d.rule_id).collect()
    }

    #[test]
    fn persistent_input_is_redef_not_gpfst() {
        // Probed: `persistent x` with input `x` gives REDEF at the name.
        let src = "function y = f(x)\npersistent x\ny = x;\nend\n";
        assert_eq!(ids(src), vec!["REDEF"]);
    }

    #[test]
    fn persistent_output_is_redef_not_gpfst() {
        // Probed: the d_kalman_filter case, `persistent P` with output `P`.
        let src = "function [xHat, P] = f(z)\nxHat = z;\npersistent x P\nP = 1;\nx = 1;\nend\n";
        assert_eq!(ids(src).iter().filter(|id| **id == "REDEF").count(), 1);
        assert!(!ids(src).contains(&"GPFST"));
    }

    #[test]
    fn global_input_is_neither_gpfst_nor_redef() {
        // Probed: `global u` with input `u` gives REDEFGI only, reported elsewhere.
        let src = "function y = f(u)\nglobal u\ny = u;\nend\n";
        assert!(!ids(src).contains(&"GPFST"));
        assert!(!ids(src).contains(&"REDEF"));
    }

    #[test]
    fn persistent_assigned_before_declaration_is_still_gpfst() {
        // Probed: `P = 1; persistent P` gives GPFST at the first use, no REDEF.
        let src = "function y = f(u)\nP = 1;\npersistent P\ny = u + P;\nend\n";
        assert!(ids(src).contains(&"GPFST"));
        assert!(!ids(src).contains(&"REDEF"));
    }

    #[test]
    fn persistent_read_without_any_assignment_is_pset() {
        // Probed: `persistent n` read and never assigned gives PSET at the declaration.
        let src = "function y = f(u)\npersistent n\ny = n + u;\nend\n";
        assert_eq!(ids(src), vec!["PSET"]);
    }

    #[test]
    fn persistent_with_conditional_assignment_is_silent() {
        // Probed: an assignment anywhere in the scope silences PSET.
        let src = "function y = f(u)\npersistent n\nif u > 0\n    n = 1;\nend\ny = n;\nend\n";
        assert!(!ids(src).contains(&"PSET"));
    }

    #[test]
    fn persistent_never_read_is_not_pset() {
        // Probed: an unused persistent is NUSED, not PSET.
        let src = "function f()\npersistent n\nend\n";
        assert!(!ids(src).contains(&"PSET"));
    }

    #[test]
    fn pset_reported_once_per_unassigned_name() {
        // Probed: `m` is assigned, so only `n` is PSET.
        let src = "function y = f(u)\npersistent n m\nm = u;\ny = n + m;\nend\n";
        assert_eq!(ids(src), vec!["PSET"]);
    }

    /// (rule ID, line) of each diagnostic for `src`, in report order.
    fn id_lines(src: &str) -> Vec<(&'static str, usize)> {
        let e = LanguageSpecEngine::from_config(&Config::default());
        lint_file(&*e, src)
            .iter()
            .map(|d| (d.rule_id, d.line))
            .collect()
    }

    #[test]
    fn persistent_then_global_is_redef_at_global() {
        // Probed (g05): REDEF at the `global n` (line 3), quoting the persistent (line 2);
        // no PSET for the read `n`.
        let src = "function y = f(u)\npersistent n\nglobal n\ny = n;\nend\n";
        assert_eq!(id_lines(src), vec![("REDEF", 3)]);
    }

    #[test]
    fn second_persistent_is_redef() {
        // Probed (g06, g25): a second `persistent n` gives REDEF at its name.
        let src = "function y = f(u)\npersistent n\npersistent n\nn = u;\ny = n;\nend\n";
        assert_eq!(id_lines(src), vec![("REDEF", 3)]);
        let src = "function y = f(u)\npersistent n\ny = n;\npersistent n\nend\n";
        assert_eq!(id_lines(src), vec![("REDEF", 4)]);
    }

    #[test]
    fn global_then_persistent_is_redef_at_persistent() {
        // Probed (g23): REDEF at the `persistent n` (line 3).
        let src = "function y = f(u)\nglobal n\npersistent n\ny = n + u;\nend\n";
        assert_eq!(id_lines(src), vec![("REDEF", 3)]);
    }

    #[test]
    fn persistent_then_assignment_then_global_is_redef() {
        // Probed (g24): the assignment between them does not change the REDEF.
        let src = "function y = f(u)\npersistent n\nn = u;\nglobal n\ny = n;\nend\n";
        assert_eq!(id_lines(src), vec![("REDEF", 4)]);
    }

    #[test]
    fn two_globals_are_not_redef() {
        // Probed (g22): two globals give REDEFGG (reported by the compatibility engine), not REDEF.
        let src = "function y = f(u)\nglobal n\nglobal n\ny = n;\nend\n";
        assert!(!ids(src).contains(&"REDEF"));
    }

    #[test]
    fn global_input_redef_is_not_duplicated() {
        // The input-argument case stays with the existing REDEFGI report: no REDEF here.
        let src = "function y = f(u)\nglobal u\nglobal u\ny = u;\nend\n";
        assert!(!ids(src).contains(&"REDEF"));
    }
}
