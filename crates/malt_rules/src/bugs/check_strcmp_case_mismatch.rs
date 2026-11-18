use super::*;
use crate::analysis::strcase::{strcmp_case, StrcmpCase};

impl BugsEngine {
    /// STCUL: a `strcmp` or `strncmp` whose case conversion makes it fail, such as
    /// `strcmp(a, lower(b))` with `a = 'X'`. See [`crate::analysis::strcase`].
    pub(crate) fn check_strcmp_case_mismatch(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if strcmp_case(node, source) != Some(StrcmpCase::Mismatch) {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "STCUL",
            message: "The comparison will likely fail due to case mismatch.".to_string(),
            severity: Severity::Error,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- STCUL ---------------------------------------------------------------

    fn stcul(src: &str) -> bool {
        has_id(&node_diags(src), "STCUL")
    }

    #[test]
    fn stcul_fires_on_conversion_against_other_case() {
        // Probed: `a = 'X'` against `lower(b)` (bugs probe gO_s1:3).
        assert!(stcul("a = 'X';\nr = strcmp(a, lower(b));\n"));
        assert!(stcul("r = strcmp('X', lower(b));\n"));
        assert!(stcul("r = strcmp(lower(b), 'Abc');\n"));
        assert!(stcul("r = strcmp(upper(b), 'abc');\n"));
        assert!(stcul("r = strcmp(lower(b), upper(a));\n"));
    }

    #[test]
    fn stcul_fires_on_variable_from_conversion_and_strncmp() {
        // Probed: `x = lower(b)` is lowercase, so `'ABC'` fails (gO_t1:3).
        assert!(stcul("x = lower(b);\nr = strcmp(x, 'ABC');\n"));
        assert!(stcul("r = strncmp('X', lower(b), 1);\n"));
        assert!(stcul("r = strncmp('x', upper(b), 1);\n"));
    }

    #[test]
    fn stcul_silent_on_matching_case_or_literals() {
        // Probed: matching case is STCI, and two literals are silent.
        assert!(!stcul("r = strcmp(lower(b), 'x');\n"));
        assert!(!stcul("r = strcmp(upper(b), 'ABC');\n"));
        assert!(!stcul("r = strcmp(lower(a), lower(b));\n"));
        assert!(!stcul("r = strcmp('abc', 'ABC');\n"));
        assert!(!stcul("a = 'X';\nb = 'x';\nr = strcmp(a, b);\n"));
        assert!(!stcul("r = strcmpi('X', lower(b));\n"));
        assert!(!stcul("r = strcmp(a, lower(b));\n"));
    }
}
