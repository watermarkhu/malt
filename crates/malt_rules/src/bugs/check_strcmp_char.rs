use super::*;

impl BugsEngine {
    /// STRCMPCSTR: `strcmp` compared against a cell array of strings, which
    /// always returns false for string elements of the cell array.
    pub(crate) fn check_strcmp_cell(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if node.kind() != "function_call" {
            return Vec::new();
        }

        let func_name = match extract_call_name(node, source) {
            Some(n) => n,
            None => return Vec::new(),
        };

        if func_name != "strcmp" {
            return Vec::new();
        }

        let has_cell_arg = arg_nodes(node).iter().any(|a| a.kind() == "cell");

        if has_cell_arg {
            let pos = node.start_position();
            vec![Diagnostic {
                rule_id: "STRCMPCSTR",
                message: "'strcmp' always returns false for string elements of a cell array. Use [\"str1\", \"str2\"] instead of {\"str1\", \"str2\"}.".to_string(),
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- STRCMPCSTR ----------------------------------------------------------

    #[test]
    fn strcmpcstr_fires_on_cell_array_arg() {
        let src = "strcmp({'a', 'b'}, 'a');\n";
        let diags = node_diags(src);
        assert!(has_id(&diags, "STRCMPCSTR"), "got: {diags:?}");
    }

    #[test]
    fn strcmpcstr_no_fire_on_char_args() {
        let src = "strcmp('ab', 'cd');\n";
        let diags = node_diags(src);
        assert!(!has_id(&diags, "STRCMPCSTR"), "got: {diags:?}");
    }
}
