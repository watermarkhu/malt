use super::*;

impl BugsEngine {
    /// FNAN: `x == NaN` / `x ~= NaN`. NaN never compares equal to anything, so
    /// `isnan` must be used.
    pub(crate) fn check_nan_comparison(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "comparison_operator" {
            return Vec::new();
        }
        let (Some(lhs), Some(rhs)) = (node.child(0), node.child(2)) else {
            return Vec::new();
        };
        let op = find_operator_text(node, source);
        if op != "==" && op != "~=" {
            return Vec::new();
        }
        let lhs_text = node_text(lhs, source).trim().to_string();
        let rhs_text = node_text(rhs, source).trim().to_string();
        let var_text = if rhs_text == "NaN" {
            lhs_text.as_str()
        } else if lhs_text == "NaN" {
            rhs_text.as_str()
        } else {
            return Vec::new();
        };
        let replacement = if op == "~=" {
            format!("~isnan({var_text})")
        } else {
            format!("isnan({var_text})")
        };
        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "FNAN",
            message: "Use ISNAN when comparing values to NaN.".to_string(),
            severity: Severity::Error,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            // The original expression is a constant; isnan changes the result.
            fix: Some(Fix::new(node.start_byte()..node.end_byte(), replacement).unsafe_fix()),
        }]
    }

    /// MNANC: a `case NaN` label in a `switch`, which can never match.
    pub(crate) fn check_switch_nan_case(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "switch_statement" {
            return Vec::new();
        }
        let mut diagnostics = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "case_clause" {
                continue;
            }
            let Some(value) = find_case_value(child) else {
                continue;
            };
            let mut labels = vec![value];
            if value.kind() == "cell" {
                let mut inner = value.walk();
                for row in value.children(&mut inner) {
                    let mut row_cursor = row.walk();
                    labels.extend(row.children(&mut row_cursor).filter(|c| c.is_named()));
                }
            }
            for label in labels {
                if label.kind() == "identifier" && node_text(label, source).trim() == "NaN" {
                    let pos = label.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "MNANC",
                        message:
                            "NaN never compares equal to any value, so this case will never be matched."
                                .to_string(),
                        severity: Severity::Error,
                        byte_range: label.start_byte()..label.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    #[test]
    fn fnan_fires_on_eq_nan() {
        let diags = node_diags("y = x == NaN;\n");
        assert!(has_id(&diags, "FNAN"), "got: {diags:?}");
    }

    #[test]
    fn fnan_fires_on_ne_nan() {
        let diags = node_diags("y = x ~= NaN;\n");
        assert!(has_id(&diags, "FNAN"), "got: {diags:?}");
        assert!(!has_id(&diags, "MNANC"), "got: {diags:?}");
    }

    #[test]
    fn fnan_no_fire_on_number_comparison() {
        let diags = node_diags("y = x == 5;\n");
        assert!(!has_id(&diags, "FNAN"), "got: {diags:?}");
    }

    #[test]
    fn fnan_no_fire_on_ordering_or_lowercase_nan() {
        let diags = node_diags("y = x < NaN;\nz = x == nan;\n");
        assert!(!has_id(&diags, "FNAN"), "got: {diags:?}");
    }

    #[test]
    fn mnanc_fires_on_case_nan() {
        let src = "switch x\n    case NaN\n        a = 1;\n    case 1\n        a = 2;\nend\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "MNANC"), "got: {diags:?}");
    }

    #[test]
    fn mnanc_fires_on_nan_in_cell_case() {
        let src = "switch x\n    case {1, NaN}\n        a = 1;\nend\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "MNANC"), "got: {diags:?}");
    }

    #[test]
    fn mnanc_no_fire_on_number_case() {
        let src = "switch x\n    case 1\n        a = 1;\nend\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "MNANC"), "got: {diags:?}");
    }
}
