//! `check_file` — one check of the Syntax Errors engine.

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

            // BADCH: Invalid control characters (< 0x20 except \t, \n, \r).
            if self.is_check_enabled("BADCH")
                && b < 0x20
                && b != b'\t'
                && b != b'\n'
                && b != b'\r'
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
                            fix: None,
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

            // BADOT: `..` not part of `...` (line continuation).
            if self.is_check_enabled("BADOT")
                && b == b'.'
                && i + 1 < len
                && bytes[i + 1] == b'.'
                && !Self::in_skip_range(i, skip_ranges)
            {
                // Check if this is part of `...` (line continuation).
                let is_ellipsis = i + 2 < len && bytes[i + 2] == b'.';
                if !is_ellipsis {
                    diagnostics.push(Diagnostic {
                        rule_id: "BADOT",
                        message: "Use of two dots (..) is an invalid MATLAB construction."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: i..i + 2,
                        line,
                        column: i - line_start + 1,
                        fix: None,
                    });
                    i += 2;
                    continue;
                } else {
                    // Skip the whole `...`.
                    i += 3;
                    continue;
                }
            }

            // TWOCM: `,,` (double comma).
            if self.is_check_enabled("TWOCM")
                && b == b','
                && i + 1 < len
                && bytes[i + 1] == b','
                && !Self::in_skip_range(i, skip_ranges)
            {
                diagnostics.push(Diagnostic {
                    rule_id: "TWOCM",
                    message: "A comma cannot immediately follow another comma.".to_string(),
                    severity: Severity::Error,
                    byte_range: i..i + 2,
                    line,
                    column: i - line_start + 1,
                    fix: None,
                });
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

    // -- BADOT: `..` not part of `...` --------------------------------------

    #[test]
    fn badot_fires_on_double_dot() {
        let diags = lint_file(&*engine(), "x = 1..2;\n");
        assert!(has_id(&diags, "BADOT"), "got: {diags:?}");
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
    fn badsp_ok_on_regular_spaces() {
        let diags = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&diags, "BADSP"), "got: {diags:?}");
    }
}
