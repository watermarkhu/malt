use super::*;

impl GoodPracticesEngine {
    /// STCI: Use `strcmpi` for case-insensitive comparison.
    ///
    /// Detects patterns like `lower(s) == '...'` or `strcmp(lower(s), '...')`.
    pub(crate) fn check_stci(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("STCI") || node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match get_function_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };

        if func_name != "strcmp" {
            return Vec::new();
        }

        // Check if either argument is wrapped in lower() or upper().
        let args_node = match find_child_of_kind(node, "arguments") {
            Some(a) => a,
            None => return Vec::new(),
        };

        let has_case_conversion = args_has_case_conversion(args_node, source);
        // Probed: a conversion against a string of the other case is STCUL, not STCI.
        if !has_case_conversion
            || crate::analysis::strcase::strcmp_case(node, source)
                != Some(crate::analysis::strcase::StrcmpCase::Insensitive)
        {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "STCI",
            message: "Use STRCMPI(str1,str2) instead of using UPPER/LOWER in a call to STRCMP."
                .to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use crate::good_practices::engine;
    use crate::test_util::{has_id, lint_nodes};

    #[test]
    fn stci_fires_when_the_conversion_turns_the_known_value_into_the_literal() {
        // Probed (sx2, sx12, sx14, zf_stci4): the conversion changes the value to the literal.
        for src in [
            "b = 'X';\ny = strcmp(lower(b), 'x');\n",
            "b = 'x';\ny = strcmp(upper(b), 'X');\n",
            "y = strcmp(lower('ABC'), 'abc');\n",
            "b = 'x';\ny = strcmp(lower(b), 'y');\n",
        ] {
            assert!(has_id(&lint_nodes(&engine(), src), "STCI"), "{src}");
        }
    }

    #[test]
    fn stci_silent_when_the_conversion_changes_nothing_and_matches() {
        // Probed silent (zf_stci1, zf_stci5, sx1, sx4, sx11, sx15, sx21): the known value
        // is already in the literal's case.
        for src in [
            "b = 'x';\ny = strcmp(lower(b), 'x');\n",
            "b = 'hello';\ny = strcmp(lower(b), 'hello');\n",
            "b = 'X';\ny = strcmp(upper(b), 'X');\n",
            "y = strcmp(lower('abc'), 'abc');\n",
            "b = 'x';\nc = 'x';\ny = strcmp(lower(b), c);\n",
        ] {
            assert!(!has_id(&lint_nodes(&engine(), src), "STCI"), "{src}");
        }
    }
}
