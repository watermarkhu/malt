//! `check_directional_formatting` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    /// Unicode directional formatting characters (U+202A..U+202E, U+2066..U+2069).
    /// MATLAB reports them as BADCT at the start of the comment or string that holds
    /// them (probed: `x = 3 % <LRE> c` is column 7, the `%`), and as BADCH at the
    /// character when they are in code (probed: `x = 3;<LRE>` is BADCH).
    pub(crate) fn check_directional_formatting(
        &self,
        source: &str,
        skip_ranges: &[Range<usize>],
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, ch) in source.char_indices() {
            let cp = ch as u32;
            if !((0x202A..=0x202E).contains(&cp) || (0x2066..=0x2069).contains(&cp)) {
                continue;
            }
            let (rule_id, message, at) = match skip_ranges.iter().find(|r| r.contains(&i)) {
                Some(r) => (
                    "BADCT",
                    "Unicode explicit directional formatting characters are not supported.",
                    r.start,
                ),
                None => ("BADCH", "Invalid text character(s).", i),
            };
            if !self.is_check_enabled(rule_id) {
                continue;
            }
            let (line, column) = line_col(source, at);
            diagnostics.push(Diagnostic {
                rule_id,
                message: message.to_string(),
                severity: Severity::Error,
                byte_range: at..at + 1,
                line,
                column,
                // The character is deleted: safe in a comment, unsafe in a string,
                // whose value changes.
                fix: (rule_id == "BADCT").then(|| {
                    let fix = malt_core::Fix::new(i..i + ch.len_utf8(), "");
                    if source[at..].starts_with('%') {
                        fix
                    } else {
                        fix.unsafe_fix()
                    }
                }),
            });
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{fix_of, has_id, lint_file, reported_without_fix};
    use malt_core::Config;

    #[test]
    fn badct_fix_deletes_the_character_safely_in_a_comment() {
        let (safe, fixed) = fix_of(&*engine(), "x = 3 % a\u{202a}b\n", "BADCT").unwrap();
        assert!(safe);
        assert_eq!(fixed, "x = 3 % ab\n");
        assert!(!has_id(&lint_file(&*engine(), &fixed), "BADCT"));
    }

    #[test]
    fn badct_fix_is_unsafe_in_a_string() {
        let (safe, fixed) = fix_of(&*engine(), "s = 'a\u{2067}b';\n", "BADCT").unwrap();
        assert!(!safe);
        assert_eq!(fixed, "s = 'ab';\n");
    }

    #[test]
    fn badch_in_code_has_no_fix() {
        assert!(reported_without_fix(
            &*engine(),
            "x = 3;\u{202a}\n",
            "BADCH"
        ));
    }

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

    fn find<'a>(diags: &'a [Diagnostic], id: &str) -> &'a Diagnostic {
        diags
            .iter()
            .find(|d| d.rule_id == id)
            .unwrap_or_else(|| panic!("no {id} in {diags:?}"))
    }

    // Probe b2: a directional character in code is BADCH at the character.
    #[test]
    fn badch_fires_on_directional_formatting_in_code() {
        let diags = lint_file(&*engine(), "x = 3;\u{202a}\n");
        assert!(has_id(&diags, "BADCH"), "got: {diags:?}");
        assert!(!has_id(&diags, "BADCT"), "got: {diags:?}");
        assert_eq!(
            (find(&diags, "BADCH").line, find(&diags, "BADCH").column),
            (1, 7)
        );
    }

    // Probe b2: in a comment, BADCT is at the `%` (column 7), not at the character.
    #[test]
    fn badct_fires_at_comment_start() {
        let diags = lint_file(&*engine(), "x = 3 % \u{202a} comment\n");
        assert!(has_id(&diags, "BADCT"), "got: {diags:?}");
        assert_eq!(
            (find(&diags, "BADCT").line, find(&diags, "BADCT").column),
            (1, 7)
        );
    }

    // Probe b2: in a string, BADCT is at the opening quote (column 5).
    #[test]
    fn badct_fires_at_string_start() {
        let diags = lint_file(&*engine(), "x = '\u{202a}';\n");
        assert!(has_id(&diags, "BADCT"), "got: {diags:?}");
        assert_eq!(
            (find(&diags, "BADCT").line, find(&diags, "BADCT").column),
            (1, 5)
        );
    }

    #[test]
    fn badct_ok_on_plain_source() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "BADCT"), "got: {diags:?}");
        assert!(!has_id(&diags, "BADCH"), "got: {diags:?}");
    }

    #[test]
    fn badct_disabled_in_config_does_not_fire() {
        let engine = engine_with_disabled(&["BADCT"]);
        let diags = lint_file(&*engine, "x = 3 % \u{202a}\n");
        assert!(!has_id(&diags, "BADCT"), "got: {diags:?}");
    }
}
