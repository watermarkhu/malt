//! `check_sepexc`: one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    /// SEPEXC: two input argument names in a function header are separated only
    /// by whitespace. Probed against R2026b: the diagnostic points at the second
    /// name, once per adjacent pair. Output lists (`function [a b] = f()`) get
    /// NCOMMA instead, so only the part after the function name is checked.
    pub(crate) fn check_sepexc(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        if !self.is_check_enabled("SEPEXC") {
            return diagnostics;
        }
        let mut line_start = 0;
        for (index, raw) in source.split_inclusive('\n').enumerate() {
            let line = raw.trim_end_matches(['\r', '\n']);
            let body = line.trim_start();
            if let Some(after) = body.strip_prefix("function") {
                if after.starts_with([' ', '\t']) {
                    if let Some(open) = line.find('(') {
                        let close = line[open..].find(')').map(|c| open + c);
                        if let Some(close) = close.filter(|_| !line[..open].contains('%')) {
                            let params = &line[open + 1..close];
                            for offset in Self::sepexc_positions(params) {
                                let at = line_start + open + 1 + offset;
                                diagnostics.push(Diagnostic {
                                    rule_id: "SEPEXC",
                                    message: "Use comma to separate between input arguments."
                                        .to_string(),
                                    severity: Severity::Error,
                                    byte_range: at..at + 1,
                                    line: index + 1,
                                    column: open + 1 + offset + 1,
                                    fix: None,
                                });
                            }
                        }
                    }
                }
            }
            line_start += raw.len();
        }
        diagnostics
    }

    /// Byte offsets, within `params`, of identifiers that follow another
    /// identifier or a `~` after whitespace only.
    fn sepexc_positions(params: &str) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut prev_token_ident = false;
        let mut prev_char_ident = false;
        let mut pending_ws = false;
        for (i, c) in params.char_indices() {
            if c == ' ' || c == '\t' {
                pending_ws = true;
                prev_char_ident = false;
            } else if c == '~' {
                // A `~` input is a name for SEPEXC: `(~ a)` reports `a` (probed).
                prev_token_ident = true;
                pending_ws = false;
                prev_char_ident = false;
            } else if c.is_ascii_alphanumeric() || c == '_' {
                if !prev_char_ident {
                    if prev_token_ident && pending_ws {
                        positions.push(i);
                    }
                    prev_token_ident = true;
                    pending_ws = false;
                }
                prev_char_ident = true;
            } else {
                prev_token_ident = false;
                pending_ws = false;
                prev_char_ident = false;
            }
        }
        positions
    }
}

#[cfg(test)]
mod tests {
    use super::SyntaxErrorsEngine;

    #[test]
    fn sepexc_positions_point_at_second_adjacent_name() {
        assert_eq!(SyntaxErrorsEngine::sepexc_positions("a b"), vec![2]);
        assert_eq!(SyntaxErrorsEngine::sepexc_positions("a b c"), vec![2, 4]);
    }

    #[test]
    fn sepexc_positions_point_at_name_after_tilde() {
        assert_eq!(SyntaxErrorsEngine::sepexc_positions("~ a"), vec![2]);
        assert_eq!(SyntaxErrorsEngine::sepexc_positions("a ~ b"), vec![4]);
    }

    #[test]
    fn sepexc_positions_ok_with_commas_tilde_and_single_names() {
        assert!(SyntaxErrorsEngine::sepexc_positions("a, b").is_empty());
        assert!(SyntaxErrorsEngine::sepexc_positions("~a").is_empty());
        assert!(SyntaxErrorsEngine::sepexc_positions("varargin").is_empty());
        assert!(SyntaxErrorsEngine::sepexc_positions("").is_empty());
    }
}
