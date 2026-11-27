//! `check_file`: one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn scan_source_text(
        &self,
        source: &str,
        skip_ranges: &[Range<usize>],
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let bytes = source.as_bytes();
        let len = bytes.len();

        // Track line/column for diagnostics.
        let mut line = 1usize;
        let mut line_start = 0usize;

        let mut i = 0;
        while i < len {
            let b = bytes[i];

            // Track newlines.
            if b == b'\n' {
                line += 1;
                line_start = i + 1;
                i += 1;
                continue;
            }

            // BADCH: Invalid control characters (< 0x20 except \t, \n, \r and NUL,
            // which MATLAB reads as the end of the text), and `$` and backtick,
            // which MATLAB has no token for. `?` and `@` are parse errors (SYNER).
            if self.is_check_enabled("BADCH")
                && ((b < 0x20 && !matches!(b, b'\t' | b'\n' | b'\r' | 0x0b | 0x0c | 0))
                    || b == b'$'
                    || b == b'`')
                && !Self::in_skip_range(i, skip_ranges)
            {
                diagnostics.push(Diagnostic {
                    rule_id: "BADCH",
                    message: "Invalid text character(s).".to_string(),
                    severity: Severity::Error,
                    byte_range: i..i + 1,
                    line,
                    column: i - line_start + 1,
                    fix: None,
                });
                i += 1;
                continue;
            }

            // BADCH: a name that starts with `_` (`_x = 1`, `y = x + _a`, `f(_a)`, `_y` as an
            // output): MATLAB has no token for it. A function name (`function _f()`) is BADCH
            // from check_function_names instead.
            let name_of_function = source[..i].trim_end().ends_with("function") && {
                let rest = &source[i..];
                let token = rest
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .unwrap_or(rest.len());
                let after = rest[token..].trim_start_matches([' ', '\t']);
                after.is_empty() || after.starts_with(['(', '\n', '\r', '%'])
            };
            if self.is_check_enabled("BADCH")
                && b == b'_'
                && (i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_'))
                && bytes.get(i + 1).is_some_and(|c| c.is_ascii_alphabetic())
                && !Self::in_skip_range(i, skip_ranges)
                && !name_of_function
            {
                diagnostics.push(Diagnostic {
                    rule_id: "BADCH",
                    message: "Invalid text character(s).".to_string(),
                    severity: Severity::Error,
                    byte_range: i..i + 1,
                    line,
                    column: i - line_start + 1,
                    fix: None,
                });
                i += 1;
                continue;
            }

            // BADSP: Non-ASCII whitespace detection.
            // Check for multi-byte UTF-8 sequences that are whitespace.
            if self.is_check_enabled("BADSP") && b > 0x7F && !Self::in_skip_range(i, skip_ranges) {
                // Decode the UTF-8 character at this position.
                if let Some(ch) = source[i..].chars().next() {
                    if ch.is_whitespace() && ch != ' ' && ch != '\t' && ch != '\n' && ch != '\r' {
                        let ch_len = ch.len_utf8();
                        diagnostics.push(Diagnostic {
                            rule_id: "BADSP",
                            message: "Invalid text character(s). The text contains an unsupported non-ASCII whitespace character.".to_string(),
                            severity: Severity::Error,
                            byte_range: i..i + ch_len,
                            line,
                            column: i - line_start + 1,
                            // Outside strings and comments the character separates tokens
                            // like a blank does; a line or paragraph separator is left alone.
                            fix: (!matches!(ch, '\u{85}' | '\u{2028}' | '\u{2029}'))
                                .then(|| malt_core::Fix::new(i..i + ch_len, " ")),
                        });
                        i += ch_len;
                        continue;
                    }
                }
            }

            // BADNE: `!=` (MATLAB uses `~=`).
            if self.is_check_enabled("BADNE")
                && b == b'!'
                && i + 1 < len
                && bytes[i + 1] == b'='
                && !Self::in_skip_range(i, skip_ranges)
            {
                diagnostics.push(Diagnostic {
                    rule_id: "BADNE",
                    message: "'Not Equals' is spelled ~= in MATLAB, not !=.".to_string(),
                    severity: Severity::Error,
                    byte_range: i..i + 2,
                    line,
                    column: i - line_start + 1,
                    fix: Some(malt_core::Fix::new(i..i + 2, "~=")),
                });
                i += 2;
                continue;
            }

            // A `...` continuation is skipped whole, so its dots are not read as `..`.
            // MATLAB never reports BADOT: `1..2` is a SYNER there.
            if b == b'.'
                && i + 2 < len
                && bytes[i + 1] == b'.'
                && bytes[i + 2] == b'.'
                && !Self::in_skip_range(i, skip_ranges)
            {
                i += 3;
                continue;
            }

            // TWOCM: `,,` (double comma). MATLAB reports it only inside brackets
            // (`[1,,2]`); between statements it is NOCOMMA.
            if self.is_check_enabled("TWOCM")
                && b == b','
                && i + 1 < len
                && bytes[i + 1] == b','
                && Self::inside_brackets(&source[line_start..i])
                && !Self::in_skip_range(i, skip_ranges)
            {
                // MATLAB reports the second comma (`x = [1,,2];` is column 8).
                diagnostics.push(Diagnostic {
                    rule_id: "TWOCM",
                    message: "A comma cannot immediately follow another comma.".to_string(),
                    severity: Severity::Error,
                    byte_range: i + 1..i + 2,
                    line,
                    column: i - line_start + 2,
                    // MATLAB's fix deletes the second comma (probed: `[1,,2]`
                    // gives `[1,2]`, `[1,, 2]` gives `[1, 2]`). `[1,,2]` is an
                    // error in MATLAB, so no running code changes meaning.
                    fix: Some(malt_core::Fix::new(i + 1..i + 2, "")),
                });
                i += 2;
                continue;
            }

            // `f(1,,2)`: outside brackets MATLAB reports SYNER at the second comma and at the
            // `)` that closes the call (probed: `f(1,,2);` gives columns 5 and 7).
            if self.is_check_enabled("SYNER")
                && b == b','
                && i + 1 < len
                && bytes[i + 1] == b','
                && last_unmatched_opener(&source[line_start..i])
                    .is_some_and(|o| source.as_bytes()[line_start + o] == b'(')
                && !Self::in_skip_range(i, skip_ranges)
            {
                diagnostics.push(Self::syner_at(source, i + 1));
                let close = source[i + 2..]
                    .find(['\n', ')'])
                    .map(|o| i + 2 + o)
                    .filter(|&c| bytes[c] == b')');
                if let Some(close) = close {
                    diagnostics.push(Self::syner_at(source, close));
                }
                i += 2;
                continue;
            }

            // Advance past multi-byte UTF-8 characters.
            if b > 0x7F {
                if let Some(ch) = source[i..].chars().next() {
                    i += ch.len_utf8();
                    continue;
                }
            }

            i += 1;
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

    // -- BADNE: `!=` instead of `~=` ----------------------------------------

    #[test]
    fn badne_fires_on_bang_equals() {
        let diags = lint_file(&*engine(), "x = 1 != 2;\n");
        assert!(has_id(&diags, "BADNE"), "got: {diags:?}");
    }

    #[test]
    fn badne_ok_on_tilde_equals() {
        let diags = lint_file(&*engine(), "x = 1 ~= 2;\n");
        assert!(!has_id(&diags, "BADNE"), "got: {diags:?}");
    }

    // -- BADOT: never reported; `..` is a SYNER --------------------------------

    #[test]
    fn badot_not_reported_on_double_dot() {
        // MATLAB probe: `y = 1..2;` gives SYNER only.
        let diags = lint_file(&*engine(), "x = 1..2;\n");
        assert!(!has_id(&diags, "BADOT"), "got: {diags:?}");
    }

    // -- BADCH: `$` and backtick are not MATLAB tokens -------------------------

    #[test]
    fn badch_fires_on_dollar_and_backtick() {
        // MATLAB probe: `x = 3 $ 4;` and `y = 5 \` 6;` give BADCH at the character.
        let diags = lint_file(&*engine(), "x = 3 $ 4;\ny = 5 ` 6;\n");
        assert_eq!(
            diags.iter().filter(|d| d.rule_id == "BADCH").count(),
            2,
            "got: {diags:?}"
        );
    }

    #[test]
    fn badch_ok_on_nul_byte() {
        // MATLAB reads a NUL byte as the end of the text: nothing is reported.
        let diags = lint_file(&*engine(), "csvread\n\0");
        assert!(!has_id(&diags, "BADCH"), "got: {diags:?}");
    }

    #[test]
    fn badch_ok_on_dollar_in_comment() {
        let diags = lint_file(&*engine(), "x = 1; % cost $5\n");
        assert!(!has_id(&diags, "BADCH"), "got: {diags:?}");
    }

    // -- TWOCM: only inside brackets --------------------------------------------

    #[test]
    fn twocm_fires_on_double_comma_in_matrix() {
        let diags = lint_file(&*engine(), "x = [1,,2];\n");
        assert!(has_id(&diags, "TWOCM"), "got: {diags:?}");
    }

    #[test]
    fn twocm_not_reported_between_statements() {
        // MATLAB probe: `x = 1,, y = 2;` gives NOCOMMA, not TWOCM.
        let diags = lint_file(&*engine(), "x = 1,, y = 2;\n");
        assert!(!has_id(&diags, "TWOCM"), "got: {diags:?}");
    }

    #[test]
    fn badot_ok_on_line_continuation() {
        let diags = lint_file(&*engine(), "x = 1 + ...\n    2;\n");
        assert!(!has_id(&diags, "BADOT"), "got: {diags:?}");
    }

    // -- TWOCM: double comma ------------------------------------------------

    #[test]
    fn twocm_fires_on_double_comma() {
        let diags = lint_file(&*engine(), "x = [1,,2];\n");
        assert!(has_id(&diags, "TWOCM"), "got: {diags:?}");
    }

    #[test]
    fn twocm_ok_on_single_comma() {
        let diags = lint_file(&*engine(), "x = [1,2];\n");
        assert!(!has_id(&diags, "TWOCM"), "got: {diags:?}");
    }

    // -- BADCH: invalid control characters ----------------------------------

    #[test]
    fn badch_fires_on_control_char() {
        let diags = lint_file(&*engine(), "x = 1;\u{1}\n");
        assert!(has_id(&diags, "BADCH"), "got: {diags:?}");
    }

    #[test]
    fn badch_ok_on_normal_source() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "BADCH"), "got: {diags:?}");
    }

    // -- BADSP: non-ASCII whitespace ----------------------------------------

    #[test]
    fn badsp_fires_on_non_ascii_space() {
        let diags = lint_file(&*engine(), "x = 1;\u{a0}\n");
        assert!(has_id(&diags, "BADSP"), "got: {diags:?}");
    }

    #[test]
    fn badsp_fix_replaces_the_character_with_a_blank() {
        let (safe, fixed) =
            crate::test_util::fix_of(&*engine(), "x\u{a0}= 1;\ny =\u{2003}2;\n", "BADSP").unwrap();
        assert!(safe);
        assert_eq!(fixed, "x = 1;\ny = 2;\n");
        assert!(!has_id(&lint_file(&*engine(), &fixed), "BADSP"));
    }

    #[test]
    fn badsp_no_fix_for_a_line_separator() {
        assert!(crate::test_util::reported_without_fix(
            &*engine(),
            "x = 1;\u{2028}y = 2;\n",
            "BADSP"
        ));
    }

    #[test]
    fn badsp_ok_on_regular_spaces() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "BADSP"), "got: {diags:?}");
    }
}
