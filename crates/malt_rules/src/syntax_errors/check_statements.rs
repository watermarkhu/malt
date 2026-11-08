//! `check_statements` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_statements(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Collect function names and variable assignments for REDEF.
        let mut function_names: HashSet<String> = HashSet::new();
        let mut variable_names: HashSet<String> = HashSet::new();
        let mut redef_reported: HashSet<String> = HashSet::new();

        self.walk_statements(
            root,
            source,
            &mut diagnostics,
            &mut function_names,
            &mut variable_names,
            &mut redef_reported,
        );

        diagnostics
    }

    pub(crate) fn walk_statements(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
        function_names: &mut HashSet<String>,
        variable_names: &mut HashSet<String>,
        redef_reported: &mut HashSet<String>,
    ) {
        let kind = node.kind();

        // NOLHS: Assignment with empty left side.
        if self.is_check_enabled("NOLHS") && kind == "assignment" {
            if let Some(lhs) = node.child_by_field_name("left") {
                let lhs_text = &source[lhs.start_byte()..lhs.end_byte()];
                if lhs_text.trim().is_empty() {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "NOLHS",
                        message: "Left side of an assignment is empty.".to_string(),
                        severity: Severity::Error,
                        byte_range: node.byte_range(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        // REDEF: Track function definitions and variable assignments.
        if kind == "function_definition" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = source[name_node.start_byte()..name_node.end_byte()].to_string();
                if self.is_check_enabled("REDEF")
                    && variable_names.contains(&name)
                    && !redef_reported.contains(&name)
                {
                    let pos = name_node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "REDEF",
                        message: format!(
                            "The current use of {} is inconsistent with its previous use or definition (line {}).",
                            name,
                            pos.row + 1
                        ),
                        severity: Severity::Error,
                        byte_range: name_node.byte_range(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                    redef_reported.insert(name.clone());
                }
                function_names.insert(name);
            }
        }

        if kind == "assignment" {
            if let Some(lhs) = node.child_by_field_name("left") {
                let lhs_kind = lhs.kind();
                if lhs_kind == "identifier" {
                    let name = source[lhs.start_byte()..lhs.end_byte()].to_string();
                    if self.is_check_enabled("REDEF")
                        && function_names.contains(&name)
                        && !redef_reported.contains(&name)
                    {
                        let pos = lhs.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "REDEF",
                            message: format!(
                                "The current use of {} is inconsistent with its previous use or definition (line {}).",
                                name,
                                pos.row + 1
                            ),
                            severity: Severity::Error,
                            byte_range: lhs.byte_range(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                        redef_reported.insert(name.clone());
                    }
                    variable_names.insert(name);
                }
            }
        }

        // SEPEXR: Missing separator between statements.
        // Check consecutive children of block/source_file that are statements
        // on the same line without separator.
        if self.is_check_enabled("SEPEXR") && (kind == "source_file" || kind == "block") {
            let mut cursor = node.walk();
            let children: Vec<Node> = node.children(&mut cursor).collect();

            for pair in children.windows(2) {
                let prev = pair[0];
                let curr = pair[1];

                // Skip error nodes and comments.
                if prev.is_error() || curr.is_error() {
                    continue;
                }
                if prev.kind() == "comment" || curr.kind() == "comment" {
                    continue;
                }

                // Check if they're on the same line with no separator.
                let prev_end = prev.end_position();
                let curr_start = curr.start_position();

                if prev_end.row == curr_start.row {
                    // Check the text between them for a separator (; or ,).
                    let between_start = prev.end_byte();
                    let between_end = curr.start_byte();
                    if between_start < between_end {
                        let between = &source[between_start..between_end];
                        let has_separator = between.contains(';') || between.contains(',');
                        if !has_separator {
                            let pos = curr.start_position();
                            diagnostics.push(Diagnostic {
                                rule_id: "SEPEXR",
                                message:
                                    "Use a newline, semicolon, or comma before this statement."
                                        .to_string(),
                                severity: Severity::Error,
                                byte_range: curr.byte_range(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: Some(malt_core::Fix::insert(between_start, ";")),
                            });
                        }
                    }
                }
            }
        }

        // Recurse into children.
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_statements(
                child,
                source,
                diagnostics,
                function_names,
                variable_names,
                redef_reported,
            );
        }
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

    // -- SEPEXR: missing separator between statements -----------------------

    #[test]
    fn sepexr_fires_on_same_line_statements() {
        let diags = lint_file(&*engine(), "x = 1; 2");
        assert!(has_id(&diags, "SEPEXR"), "got: {diags:?}");
    }

    #[test]
    fn sepexr_ok_on_separate_lines() {
        let diags = lint_file(&*engine(), "x = 1;\ny = 2;\n");
        assert!(!has_id(&diags, "SEPEXR"), "got: {diags:?}");
    }

    // -- NOLHS: assignment with empty left side -----------------------------

    #[test]
    fn nolhs_fires_on_empty_lhs() {
        let diags = lint_file(&*engine(), "= 5;\n");
        assert!(has_id(&diags, "NOLHS"), "got: {diags:?}");
    }

    #[test]
    fn nolhs_ok_on_normal_assignment() {
        let diags = lint_file(&*engine(), "x = 5;\n");
        assert!(!has_id(&diags, "NOLHS"), "got: {diags:?}");
    }

    // -- REDEF: identifier as both function name and variable ---------------

    #[test]
    fn redef_fires_on_function_and_variable() {
        let diags = lint_file(&*engine(), "function foo()\n    foo = 1;\nend\n");
        assert!(has_id(&diags, "REDEF"), "got: {diags:?}");
    }

    #[test]
    fn redef_ok_on_distinct_names() {
        let diags = lint_file(&*engine(), "function foo()\n    x = 1;\nend\n");
        assert!(!has_id(&diags, "REDEF"), "got: {diags:?}");
    }
}
