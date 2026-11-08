//! `check_directional_formatting` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_directional_formatting(&self, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("BADCT") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let mut line = 1usize;
        let mut line_start = 0usize;

        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                line += 1;
                line_start = i + 1;
                continue;
            }
            let cp = ch as u32;
            // U+202A..U+202E (LRE, RLE, PDF, LRO, RLO) and
            // U+2066..U+2069 (LRI, RLI, FSI, PDI) are directional formatting
            // characters that MATLAB rejects.
            if (0x202A..=0x202E).contains(&cp) || (0x2066..=0x2069).contains(&cp) {
                diagnostics.push(Diagnostic {
                    rule_id: "BADCT",
                    message:
                        "Unicode explicit directional formatting characters are not supported."
                            .to_string(),
                    severity: Severity::Error,
                    byte_range: i..i + ch.len_utf8(),
                    line,
                    column: i - line_start + 1,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        SyntaxErrorsEngine::from_config(&Config::default())
    }

    fn engine_with_disabled(checks: &[&str]) -> Box<dyn Rule> {
        let disabled = checks
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let config = Config::from_toml(&format!(
            "[lint.rules.SYNTAX_ERRORS_ENGINE]\ndisabled_checks = [{disabled}]\n"
        ))
        .unwrap();
        SyntaxErrorsEngine::from_config(&config)
    }

    // -- BADCT: Unicode directional formatting characters --------------------

    #[test]
    fn badct_fires_on_directional_formatting() {
        let diags = lint_file(&*engine(), "x = 1;\u{202a}\n");
        assert!(has_id(&diags, "BADCT"), "got: {diags:?}");
    }

    #[test]
    fn badct_fires_on_rlo() {
        let diags = lint_file(&*engine(), "x = 1;\u{202e}\n");
        assert!(has_id(&diags, "BADCT"), "got: {diags:?}");
    }

    #[test]
    fn badct_ok_on_plain_source() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "BADCT"), "got: {diags:?}");
    }

    #[test]
    fn badct_disabled_in_config_does_not_fire() {
        let engine = engine_with_disabled(&["BADCT"]);
        let diags = lint_file(&*engine, "x = 1;\u{202a}\n");
        assert!(!has_id(&diags, "BADCT"), "got: {diags:?}");
    }
}
