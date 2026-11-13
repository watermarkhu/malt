//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// IDXCOLND: END operator used outside of an indexing expression.
    pub(crate) fn check_end_operator(
        &self,
        root: tree_sitter::Node,
        _source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        Self::check_end_dfs(root, false, diagnostics);
    }

    /// DFS to find `end` keywords used as values outside of indexing contexts.
    pub(crate) fn check_end_dfs(
        node: tree_sitter::Node,
        in_index: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "function_call" => {
                // The arguments of a function_call could be indexing context
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    // Arguments inside function_call are in indexing context
                    // (since function_call is also used for array indexing)
                    Self::check_end_dfs(child, true, diagnostics);
                }
                return;
            }
            "cell_index" => {
                // Cell indexing — end is valid here
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    Self::check_end_dfs(child, true, diagnostics);
                }
                return;
            }
            "end" | "end_operator" if !in_index => {
                // `end` used as a value (not as block terminator)
                let pos = node.start_position();
                // Only flag if this looks like an end used as a value expression,
                // not a block-closing end keyword
                let parent = node.parent();
                // A bare `end` token directly under an ERROR node closes a
                // broken block (`elseif ... else ... end` at statement level,
                // bugs__f1.m:28). A value `end` is wrapped in `end_keyword`.
                let stray_block_end =
                    node.kind() == "end" && parent.is_some_and(|p| p.kind() == "ERROR");
                let is_block_end = stray_block_end
                    || parent
                        .map(|p| {
                            matches!(
                                p.kind(),
                                "function_definition"
                                    | "if_statement"
                                    | "for_statement"
                                    | "while_statement"
                                    | "switch_statement"
                                    | "try_statement"
                                    | "class_definition"
                                    | "properties"
                                    | "methods"
                                    | "events"
                                    | "enumeration"
                                    | "arguments_statement"
                                    | "parfor"
                                    | "spmd_statement"
                                    | "elseif_clause"
                                    | "else_clause"
                                    | "case_clause"
                                    | "otherwise_clause"
                                    | "catch_clause"
                            )
                        })
                        .unwrap_or(false);

                if !is_block_end && !in_control_header(node) {
                    diagnostics.push(Diagnostic {
                        rule_id: "IDXCOLND",
                        message: "The END operator must be used within an array index expression."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::check_end_dfs(child, in_index, diagnostics);
        }
    }
}

/// Whether an `end` sits in the header of an `if`, `elseif`, `while`, `for` or `switch`
/// statement, where the parse fails. MATLAB reports that as SYNER, not as IDXCOLND
/// (probed: `if end > 1`).
pub(crate) fn in_control_header(node: tree_sitter::Node) -> bool {
    let mut child = node;
    while let Some(parent) = child.parent() {
        match parent.kind() {
            "if_statement" | "elseif_clause" | "while_statement" => {
                return parent
                    .child_by_field_name("condition")
                    .is_some_and(|c| c.id() == child.id());
            }
            "for_statement" | "switch_statement" => return child.kind() != "block",
            "block"
            | "source_file"
            | "function_definition"
            | "class_definition"
            | "function_call"
            | "cell_index"
            | "arguments" => return false,
            _ => child = parent,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::language_spec::LanguageSpecEngine;

    fn idxcolnd_lines(source: &str) -> Vec<usize> {
        let tree = crate::language_spec::tests::parse_matlab(source);
        let mut diags = Vec::new();
        LanguageSpecEngine::check_end_dfs(tree.root_node(), false, &mut diags);
        diags.iter().map(|d| d.line).collect()
    }

    #[test]
    fn test_idxcolnd_fires_end_value_in_if_body() {
        // Probed (e23): `y = end;` inside an `if` body is IDXCOLND at line 3.
        let source = "function e23(x)\nif x\ny = end;\nend\nend\n";
        assert_eq!(idxcolnd_lines(source), vec![3]);
    }

    #[test]
    fn test_idxcolnd_silent_end_in_if_header() {
        // Probed (e07): `if end > 1` is a parse error (SYNER), not IDXCOLND.
        let source = "function e07(x)\nif end > 1\ndisp(1)\nend\nend\n";
        assert!(idxcolnd_lines(source).is_empty());
    }

    #[test]
    fn test_idxcolnd_fires_end_value_in_broken_statement() {
        // MATLAB: `g = end;` is IDXCOLND (t__t1.m:14); the value `end` is
        // wrapped in `end_keyword` by the grammar.
        let source = "function t1(x)\ng = end;\nend\n";
        assert_eq!(idxcolnd_lines(source), vec![2]);
    }

    #[test]
    fn test_idxcolnd_silent_on_stray_block_end() {
        // MATLAB: the stray `end` closing a broken `elseif ... else ... end`
        // line is SYNER only (bugs__f1.m:28).
        let source = "\
function t1(x, y)
a = 1;
elseif y, a=1; else a=1; end
";
        assert!(idxcolnd_lines(source).is_empty());
    }
}
