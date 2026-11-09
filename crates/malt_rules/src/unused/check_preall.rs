//! PREALL check: loop variable preallocated but unused.

use super::*;

impl UnusedEngine {
    /// Run PREALL check: an array preallocated (`x = zeros(1, n)`) whose value
    /// is replaced by a later full assignment in the same block before anything
    /// reads it, so the preallocation does nothing.
    pub(crate) fn check_preall(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("PREALL") {
            return;
        }

        for scope in &table.scopes {
            for def in &scope.defs {
                if !def.prealloc || def.kind != DefKind::Assignment || def.partial {
                    continue;
                }
                let name = &def.name;
                if self.should_ignore_name(name) {
                    continue;
                }
                // The next definition of the variable, in source order.
                let next = scope
                    .defs
                    .iter()
                    .filter(|d| d.name == *name && d.byte_range.start > def.byte_range.start)
                    .min_by_key(|d| d.byte_range.start);
                let Some(next) = next else {
                    continue;
                };
                // Overwritten unconditionally: a full assignment in the same block.
                if next.kind != DefKind::Assignment
                    || next.partial
                    || next.block_start != def.block_start
                {
                    continue;
                }
                // Read in between (including by the next assignment's own
                // right-hand side)?
                let read = scope.uses.iter().any(|u| {
                    u.name == *name
                        && u.byte_range.start >= def.effect_start
                        && u.byte_range.start < next.effect_start
                });
                if read {
                    continue;
                }
                diagnostics.push(Diagnostic {
                    rule_id: "PREALL",
                    message: format!(
                        "The preallocated value assigned to variable '{name}' might be unused."
                    ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    fn preall(src: &str) -> bool {
        has_id(&lint_file(&*engine(), src), "PREALL")
    }

    #[test]
    fn preall_fires_when_preallocation_is_overwritten() {
        assert!(preall(
            "function foo(n)\n    x = zeros(1, n);\n    x = compute(n);\n    disp(x);\nend\n"
        ));
    }

    #[test]
    fn preall_ok_when_filled_by_indexing() {
        assert!(!preall("function foo(n)\n    x = zeros(1, n);\n    for i = 1:n\n        x(i) = i;\n    end\n    disp(x);\nend\n"));
    }

    #[test]
    fn preall_ok_when_read_before_overwrite() {
        assert!(!preall(
            "function foo(n)\n    x = zeros(1, n);\n    x = x + 1;\n    disp(x);\nend\n"
        ));
    }

    #[test]
    fn preall_ok_when_overwrite_is_conditional() {
        assert!(!preall("function foo(n, c)\n    x = zeros(1, n);\n    if c\n        x = 5;\n    end\n    disp(x);\nend\n"));
    }

    #[test]
    fn preall_ignores_unused_loop_variables() {
        assert!(!preall(
            "function foo()\n    for i = 1:10\n        disp('hi');\n    end\nend\n"
        ));
    }
}
