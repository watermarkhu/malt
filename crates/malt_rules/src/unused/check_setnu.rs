//! SETNU check: variable is assigned but its value is never subsequently used.

use super::*;

impl UnusedEngine {
    /// Run SETNU check: a variable that is assigned (set) but whose assigned
    /// value is never subsequently used. Unlike NASGU (which fires when a
    /// variable is never used at all), SETNU fires per-assignment when the
    /// value written at that point is never read before the next assignment.
    ///
    /// # Limitations
    ///
    /// The boundary between SETNU and NASGU is not documented precisely by
    /// MathWorks. This implementation fires SETNU on any assignment whose value
    /// is never read before the next assignment (a dead store), which overlaps
    /// with NASGU for variables that are assigned once and never read: such a
    /// variable may be reported by both checks. This follows the message text
    /// literally but is intentionally permissive.
    pub(crate) fn check_setnu(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("SETNU") {
            return;
        }

        for scope in &table.scopes {
            // Output arguments are "used" externally, so skip them.
            let output_args: HashSet<&str> = scope
                .defs
                .iter()
                .filter(|d| d.kind == DefKind::OutputArg)
                .map(|d| d.name.as_str())
                .collect();

            for def in &scope.defs {
                if def.kind != DefKind::Assignment {
                    continue;
                }
                let name = &def.name;
                // Full assignments of a never-read variable are NASGU's.
                if !def.partial || def.multi_output {
                    continue;
                }

                if output_args.contains(name.as_str()) {
                    continue;
                }
                if self.should_ignore_name(name) {
                    continue;
                }

                // Report once per variable, at its first partial assignment,
                // and only when it is never read: a use that is the target of
                // a partial assignment itself does not count as a read.
                let is_target = |at: usize| {
                    scope
                        .defs
                        .iter()
                        .any(|d| d.partial && d.name == *name && d.byte_range.start == at)
                };
                let first = scope
                    .defs
                    .iter()
                    .find(|d| d.partial && d.name == *name)
                    .is_some_and(|d| std::ptr::eq(d, def));
                let read = scope
                    .uses
                    .iter()
                    .any(|u| u.name == *name && !is_target(u.byte_range.start));
                let fully_assigned = scope
                    .defs
                    .iter()
                    .any(|d| d.name == *name && !d.partial && d.kind == DefKind::Assignment);

                if first && !read && !fully_assigned {
                    diagnostics.push(Diagnostic {
                        rule_id: "SETNU",
                        message: "Variable is set, but might be unused.".to_string(),
                        severity: Severity::Warning,
                        byte_range: def.byte_range.clone(),
                        line: def.line,
                        column: def.column,
                        fix: None,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    // -- SETNU: assignment whose value is never subsequently used ------------

    #[test]
    fn setnu_fires_on_partial_assignment_never_read() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    A(2) = 5;\n    A(3) = 6;\nend\n",
        );
        let n = diags.iter().filter(|d| d.rule_id == "SETNU").count();
        assert_eq!(n, 1, "once per variable, got: {diags:?}");
        assert!(!has_id(&diags, "NASGU"), "disjoint from NASGU: {diags:?}");
    }

    #[test]
    fn setnu_leaves_full_assignments_to_nasgu() {
        let diags = lint_file(&*engine(), "function foo()\n    x = 5;\nend\n");
        assert!(!has_id(&diags, "SETNU"), "got: {diags:?}");
        assert!(has_id(&diags, "NASGU"), "got: {diags:?}");
    }

    #[test]
    fn setnu_ok_when_value_used() {
        let diags = lint_file(
            &*engine(),
            "function foo()\n    x = 1;\n    disp(x);\nend\n",
        );
        assert!(!has_id(&diags, "SETNU"), "got: {diags:?}");
    }

    #[test]
    fn setnu_ok_on_output_arg() {
        let diags = lint_file(&*engine(), "function y = foo()\n    y = 1;\nend\n");
        assert!(!has_id(&diags, "SETNU"), "got: {diags:?}");
    }
}
