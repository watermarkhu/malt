//! `check_name_value` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_name_value(
        &self,
        call: Node,
        args: Option<Node>,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Assignment shape: `f('Bad Name' = 1)` parses as an assignment whose
        // LHS is a function_call with a MISSING closing paren.
        let is_assignment_lhs = call.parent().is_some_and(|p| {
            p.kind() == "assignment"
                && p.child_by_field_name("left")
                    .is_some_and(|l| l.id() == call.id())
        });
        let has_missing_paren = (0..call.child_count()).any(|i| {
            call.child(i)
                .is_some_and(|c| c.is_missing() && c.kind() == ")")
        });
        if is_assignment_lhs && has_missing_paren {
            if let Some(name) = args.and_then(Self::last_named_child) {
                self.push_name_value_diagnostic(name, source, diagnostics);
            }
        }

        // ERROR shape: an ERROR node containing `=` inside the call's args.
        for e in Self::call_error_nodes(call) {
            let text = &source[e.start_byte()..e.end_byte()];
            if !text.contains('=') {
                continue;
            }
            let name = if Self::has_string_child(e) {
                e
            } else if let Some(last) = args.and_then(Self::last_named_child) {
                last
            } else {
                e
            };
            self.push_name_value_diagnostic(name, source, diagnostics);
        }

        // Direct `=` tokens inside arguments (name=value parses cleanly only
        // for valid identifier names).
        if let Some(a) = args {
            for i in 0..a.child_count() {
                if let Some(eq) = a.child(i) {
                    if eq.kind() != "=" {
                        continue;
                    }
                    if let Some(name) = eq.prev_named_sibling() {
                        self.push_name_value_diagnostic(name, source, diagnostics);
                    }
                }
            }
        }
    }
}
