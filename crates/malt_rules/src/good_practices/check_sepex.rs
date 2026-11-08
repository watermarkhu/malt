use super::*;

impl GoodPracticesEngine {
    /// SEPEX: Multiple statements on one line.
    pub(crate) fn check_sepex(&self, tree: &tree_sitter::Tree, _source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("SEPEX") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let mut line_statements: std::collections::HashMap<usize, Vec<(usize, usize)>> =
            std::collections::HashMap::new();

        // Walk all statement-level nodes and group by line.
        collect_statements_by_line(tree.root_node(), &mut line_statements);

        for (line, stmts) in &line_statements {
            if stmts.len() > 1 {
                // Report on the second statement onwards.
                for &(start, end) in &stmts[1..] {
                    diagnostics.push(Diagnostic {
                        rule_id: "SEPEX",
                        message: "Consider using newline, semicolon, or comma before this statement for readability."
                            .to_string(),
                        severity: Severity::Info,
                        byte_range: start..end,
                        line: *line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}
