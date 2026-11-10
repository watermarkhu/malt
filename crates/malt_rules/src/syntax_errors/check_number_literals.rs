//! `check_number_literals` — one check of the Syntax Errors engine.

use super::*;

const HEX_DIGIT_MESSAGE: &str = "Invalid digit in hexadecimal literal. Supported hexadecimal digits are 0-9 and A-F. Supported type suffixes are u8, u16, u32, u64, and s8, s16, s32, s64.";
const BIN_DIGIT_MESSAGE: &str = "Invalid digit in binary literal. Supported binary digits are 0 and 1. Supported type suffixes are u8, u16, u32, u64, and s8, s16, s32, s64.";

/// True if `rest` (the text after an `e` or `E`) has no exponent digits: an
/// optional sign, then nothing that could continue a number.
fn is_empty_exponent(rest: &str) -> bool {
    let rest = rest.trim_start_matches(['+', '-']);
    rest.chars()
        .next()
        .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
}

impl SyntaxErrorsEngine {
    pub(crate) fn check_number_literals(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut numbers = Vec::new();
        let mut errors = Vec::new();
        Self::collect_numbers_and_errors(root, &mut numbers, &mut errors);
        errors.sort_by_key(|node| node.start_byte());

        for number in numbers {
            let start = number.start_byte();
            let end = number.end_byte();
            let text = &source[start..end];

            // Truncated literal checks: an ERROR node immediately following
            // the number holds the invalid remainder of the literal.
            if !Self::inside_error(number) {
                if let Ok(idx) = errors.binary_search_by_key(&end, |n| n.start_byte()) {
                    let error = errors[idx];
                    let err_text = &source[error.byte_range()];
                    if let Some(first) = err_text.chars().next() {
                        let literal_like = first.is_ascii_alphanumeric() || first == '_';
                        // A diagnostic on the literal's remainder covers the whole
                        // token, so the SYNER of that remainder is recognised as the same error.
                        let span = start..error.end_byte();
                        let at_number = |rule_id: &'static str, message: &str| Diagnostic {
                            rule_id,
                            message: message.to_string(),
                            severity: Severity::Error,
                            byte_range: span.clone(),
                            line: number.start_position().row + 1,
                            column: number.start_position().column + 1,
                            fix: None,
                        };
                        if text == "0" && matches!(first, 'x' | 'X') {
                            if self.is_check_enabled("BADHBH") {
                                diagnostics.push(at_number("BADHBH", HEX_DIGIT_MESSAGE));
                            }
                        } else if text == "0" && matches!(first, 'b' | 'B') {
                            if self.is_check_enabled("BADHBB") {
                                diagnostics.push(at_number("BADHBB", BIN_DIGIT_MESSAGE));
                            }
                        } else if (text.starts_with("0x") || text.starts_with("0X"))
                            && self.is_check_enabled("BADHBH")
                            && literal_like
                        {
                            diagnostics.push(Self::number_diagnostic(
                                "BADHBH",
                                HEX_DIGIT_MESSAGE,
                                number,
                            ));
                        } else if (text.starts_with("0b") || text.starts_with("0B"))
                            && self.is_check_enabled("BADHBB")
                            && literal_like
                        {
                            diagnostics.push(Self::number_diagnostic(
                                "BADHBB",
                                BIN_DIGIT_MESSAGE,
                                number,
                            ));
                        } else if self.is_check_enabled("BADFP")
                            && matches!(first, 'e' | 'E')
                            && is_empty_exponent(&err_text[1..])
                        {
                            // `1e` and `1e+` are a float with no exponent digits. A
                            // `.` or digit after a number is a SYNER (`1.2.3`, `1..2`).
                            diagnostics
                                .push(at_number("BADFP", "Invalid floating-point constant."));
                        }
                    }
                }
            }

            // Digit-count checks for hex and binary literals.
            if let Some((is_hex, digit_count, suffix)) = Self::parse_hex_binary_literal(text) {
                match suffix {
                    Some(suffix) => {
                        if let Some(max) = Self::max_digits_for_suffix(suffix, is_hex) {
                            if digit_count > max {
                                let (rule_id, message) = if is_hex {
                                    (
                                            "BADHBHT",
                                            "Hexadecimal literal has too many digits for specified type suffix.",
                                        )
                                } else {
                                    (
                                            "BADHBBT",
                                            "Binary literal has too many digits for specified type suffix.",
                                        )
                                };
                                if self.is_check_enabled(rule_id) {
                                    diagnostics
                                        .push(Self::number_diagnostic(rule_id, message, number));
                                }
                            }
                        }
                    }
                    None => {
                        if is_hex && digit_count > 16 && self.is_check_enabled("HEXTOOLONG") {
                            diagnostics.push(Self::number_diagnostic(
                                "HEXTOOLONG",
                                "Hexadecimal literal has too many digits.",
                                number,
                            ));
                        } else if !is_hex
                            && digit_count > 64
                            && self.is_check_enabled("BINARYTOOLONG")
                        {
                            diagnostics.push(Self::number_diagnostic(
                                "BINARYTOOLONG",
                                "Binary literal has too many digits.",
                                number,
                            ));
                        }
                    }
                }
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

    // -- BADFP: invalid floating-point constant ------------------------------

    #[test]
    fn badfp_fires_on_empty_exponent() {
        // MATLAB probe: `v = 1e;` and `v = 1e+;` give BADFP at the number.
        let diags = lint_file(&*engine(), "x = 1e;\n");
        assert!(has_id(&diags, "BADFP"), "got: {diags:?}");
        let diags = lint_file(&*engine(), "x = 1e+;\n");
        assert!(has_id(&diags, "BADFP"), "got: {diags:?}");
    }

    #[test]
    fn badfp_ok_on_valid_float() {
        let diags = lint_file(&*engine(), "x = 1.2;\n");
        assert!(!has_id(&diags, "BADFP"), "got: {diags:?}");
    }

    #[test]
    fn badfp_ok_on_whitespace_separated_numbers() {
        let diags = lint_file(&*engine(), "x = 1 2;\n");
        assert!(!has_id(&diags, "BADFP"), "got: {diags:?}");
    }

    #[test]
    fn badfp_not_reported_on_extra_dot_or_digits() {
        // MATLAB probe: `1.2.3`, `1.5e5.3` and `1..2` give SYNER only.
        for source in ["x = 1.2.3;\n", "x = 1.5e5.3;\n", "y = 1..2;\n"] {
            let diags = lint_file(&*engine(), source);
            assert!(!has_id(&diags, "BADFP"), "{source:?}: got: {diags:?}");
        }
    }

    #[test]
    fn badfp_reports_number_byte_range() {
        // The diagnostic covers the number and its invalid remainder, `1e`.
        let diags = lint_file(&*engine(), "x = 1e;\n");
        let d = diags
            .iter()
            .find(|d| d.rule_id == "BADFP")
            .expect("BADFP fired");
        assert_eq!(d.byte_range, 4..6);
    }

    // -- BADHBH: invalid digit in hexadecimal literal ------------------------

    #[test]
    fn badhbh_fires_on_invalid_hex_digit() {
        let diags = lint_file(&*engine(), "x = 0xFFG;\n");
        assert!(has_id(&diags, "BADHBH"), "got: {diags:?}");
    }

    #[test]
    fn badhbh_fires_on_invalid_hex_suffix() {
        let diags = lint_file(&*engine(), "x = 0xFFu;\n");
        assert!(has_id(&diags, "BADHBH"), "got: {diags:?}");
    }

    #[test]
    fn badhbh_ok_on_valid_hex() {
        let diags = lint_file(&*engine(), "x = 0xFF;\n");
        assert!(!has_id(&diags, "BADHBH"), "got: {diags:?}");
    }

    #[test]
    fn badhbh_ok_on_unrelated_error() {
        let diags = lint_file(&*engine(), "x = 0xFF);\n");
        assert!(!has_id(&diags, "BADHBH"), "got: {diags:?}");
    }

    // -- BADHBB: invalid digit in binary literal -----------------------------

    #[test]
    fn badhbb_fires_on_invalid_binary_digit() {
        let diags = lint_file(&*engine(), "x = 0b102;\n");
        assert!(has_id(&diags, "BADHBB"), "got: {diags:?}");
    }

    #[test]
    fn badhbb_ok_on_valid_binary() {
        let diags = lint_file(&*engine(), "x = 0b10;\n");
        assert!(!has_id(&diags, "BADHBB"), "got: {diags:?}");
    }

    // -- BADHBHT: hex literal too long for its type suffix -------------------

    #[test]
    fn badhbht_fires_on_too_many_hex_digits() {
        let diags = lint_file(&*engine(), "x = 0xFFFFu8;\n");
        assert!(has_id(&diags, "BADHBHT"), "got: {diags:?}");
    }

    #[test]
    fn badhbht_ok_on_exact_fit() {
        let diags = lint_file(&*engine(), "x = 0xFFu8;\n");
        assert!(!has_id(&diags, "BADHBHT"), "got: {diags:?}");
    }

    #[test]
    fn badhbht_ok_on_larger_suffix() {
        let diags = lint_file(&*engine(), "x = 0xFFFFu16;\n");
        assert!(!has_id(&diags, "BADHBHT"), "got: {diags:?}");
    }

    // -- BADHBBT: binary literal too long for its type suffix ----------------

    #[test]
    fn badhbbt_fires_on_too_many_binary_digits() {
        let diags = lint_file(&*engine(), "x = 0b111111111u8;\n");
        assert!(has_id(&diags, "BADHBBT"), "got: {diags:?}");
    }

    #[test]
    fn badhbbt_ok_on_exact_fit() {
        let diags = lint_file(&*engine(), "x = 0b11111111u8;\n");
        assert!(!has_id(&diags, "BADHBBT"), "got: {diags:?}");
    }

    #[test]
    fn badhbbt_ok_on_larger_suffix() {
        let diags = lint_file(&*engine(), "x = 0b111111111u16;\n");
        assert!(!has_id(&diags, "BADHBBT"), "got: {diags:?}");
    }

    // -- HEXTOOLONG: unsuffixed hex literal with too many digits -------------

    #[test]
    fn hextoolong_fires_on_more_than_16_hex_digits() {
        let diags = lint_file(&*engine(), "x = 0xFFFFFFFFFFFFFFFFFFF;\n");
        assert!(has_id(&diags, "HEXTOOLONG"), "got: {diags:?}");
    }

    #[test]
    fn hextoolong_ok_on_16_hex_digits() {
        let diags = lint_file(&*engine(), "x = 0xFFFFFFFFFFFFFFFF;\n");
        assert!(!has_id(&diags, "HEXTOOLONG"), "got: {diags:?}");
    }

    // -- BINARYTOOLONG: unsuffixed binary literal with too many digits -------

    #[test]
    fn binarytoolong_fires_on_more_than_64_bits() {
        let src = format!("x = 0b{};\n", "1".repeat(65));
        let diags = lint_file(&*engine(), &src);
        assert!(has_id(&diags, "BINARYTOOLONG"), "got: {diags:?}");
    }

    #[test]
    fn binarytoolong_ok_on_64_bits() {
        let src = format!("x = 0b{};\n", "1".repeat(64));
        let diags = lint_file(&*engine(), &src);
        assert!(!has_id(&diags, "BINARYTOOLONG"), "got: {diags:?}");
    }

    // -- number literal checks disabled via config ---------------------------

    #[test]
    fn disabled_checks_turn_off_number_literal_checks() {
        let engine = engine_with_disabled(&[
            "BADFP",
            "BADHBH",
            "BADHBB",
            "BADHBHT",
            "BADHBBT",
            "HEXTOOLONG",
            "BINARYTOOLONG",
        ]);
        let src = format!(
            "x = 1.2.3;\nx = 0xFFG;\nx = 0b102;\nx = 0xFFFFFFFFFFFFFFFFFFFF;\nx = 0b{};\nx = 0xFFFFu8;\nx = 0b111111111u8;\n",
            "1".repeat(65)
        );
        let diags = lint_file(&*engine, &src);
        assert!(!has_id(&diags, "BADFP"), "got: {diags:?}");
        assert!(!has_id(&diags, "BADHBH"), "got: {diags:?}");
        assert!(!has_id(&diags, "BADHBB"), "got: {diags:?}");
        assert!(!has_id(&diags, "BADHBHT"), "got: {diags:?}");
        assert!(!has_id(&diags, "BADHBBT"), "got: {diags:?}");
        assert!(!has_id(&diags, "HEXTOOLONG"), "got: {diags:?}");
        assert!(!has_id(&diags, "BINARYTOOLONG"), "got: {diags:?}");
    }
}
