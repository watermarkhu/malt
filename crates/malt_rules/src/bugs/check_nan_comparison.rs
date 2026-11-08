use super::*;

impl BugsEngine {
    /// FNAN / MNANC: Comparison with NaN using `==` or `~=`.
    ///
    /// `NaN == NaN` is always false in IEEE 754; `isnan()` must be used instead.
    pub(crate) fn check_nan_comparison(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "comparison_operator" {
            return Vec::new();
        }

        let lhs = match node.child(0) {
            Some(c) => c,
            None => return Vec::new(),
        };
        let rhs = match node.child(2) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let lhs_text = node_text(lhs, source).trim().to_string();
        let rhs_text = node_text(rhs, source).trim().to_string();

        let has_nan = lhs_text.eq_ignore_ascii_case("nan")
            || lhs_text.eq_ignore_ascii_case("NaN")
            || rhs_text.eq_ignore_ascii_case("nan")
            || rhs_text.eq_ignore_ascii_case("NaN");

        if !has_nan {
            return Vec::new();
        }

        let op = find_operator_text(node, source);
        let (rule_id, message) = if op == "~=" {
            (
                "MNANC",
                "NaN never compares equal to any value, so this case will never be matched.",
            )
        } else {
            ("FNAN", "Use ISNAN when comparing values to NaN.")
        };

        let pos = node.start_position();

        // Suggest a fix: replace `x == NaN` with `isnan(x)`.
        let (var_text, is_negated) = if lhs_text.eq_ignore_ascii_case("nan") {
            (rhs_text.as_str(), op == "~=")
        } else {
            (lhs_text.as_str(), op == "~=")
        };

        let replacement = if is_negated {
            format!("~isnan({var_text})")
        } else {
            format!("isnan({var_text})")
        };

        vec![Diagnostic {
            rule_id,
            message: message.to_string(),
            severity: Severity::Error,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: Some(Fix::new(node.start_byte()..node.end_byte(), replacement)),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- FNAN / MNANC --------------------------------------------------------

    #[test]
    fn fnan_fires_on_eq_nan() {
        let src = "y = x == NaN;\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "FNAN"), "got: {diags:?}");
    }

    #[test]
    fn fnan_no_fire_on_number_comparison() {
        let src = "y = x == 5;\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "FNAN"), "got: {diags:?}");
    }

    #[test]
    fn mnanc_fires_on_ne_nan() {
        let src = "y = x ~= NaN;\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "MNANC"), "got: {diags:?}");
    }

    #[test]
    fn mnanc_no_fire_on_number_comparison() {
        let src = "y = x ~= 5;\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "MNANC"), "got: {diags:?}");
    }
}
