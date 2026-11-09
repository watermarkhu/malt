//! VANUS check: `varargin` input argument declared but never used.

use super::*;

impl UnusedEngine {
    /// Run VANUS check: a `varargin` input argument that is never used.
    pub(crate) fn check_vanus(&self, table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        if self.is_check_disabled("VANUS") {
            return;
        }

        for scope in &table.scopes {
            for def in &scope.defs {
                if def.kind != DefKind::InputArg {
                    continue;
                }
                if def.name != "varargin" {
                    continue;
                }
                if !scope.is_used("varargin") {
                    diagnostics.push(Diagnostic {
                        rule_id: "VANUS",
                        message: "Input argument 'varargin' might be unused.".to_string(),
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

    // -- VANUS: unused `varargin` input argument -----------------------------

    #[test]
    fn vanus_fires_on_unused_varargin() {
        let diags = lint_file(&*engine(), "function foo(varargin)\nend\n");
        assert!(has_id(&diags, "VANUS"), "got: {diags:?}");
    }

    #[test]
    fn vanus_ok_when_varargin_used() {
        let diags = lint_file(
            &*engine(),
            "function foo(varargin)\n    x = varargin{1};\n    disp(x);\nend\n",
        );
        assert!(!has_id(&diags, "VANUS"), "got: {diags:?}");
    }
}
