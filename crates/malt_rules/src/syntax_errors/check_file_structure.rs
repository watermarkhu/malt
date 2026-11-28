//! `check_file_structure`: one check of the Syntax Errors engine.

use super::*;

const CLIS_MESSAGE: &str = "Defining a class in script is not allowed.";
const CLTWO_MESSAGE: &str =
    "Only one class definition is allowed per file, and it must come at the head of the file.";
const SOFOC_MESSAGE: &str = "Statement outside a class definition is not allowed.";
const SEMFU_MESSAGE: &str = "Script file must contain executable code. Remove empty statements to make this file a function file.";

/// A top-level node that does nothing: a comment, a blank, or a bare `;`.
fn is_empty_statement(node: Node, source: &str) -> bool {
    if node.kind() == "comment" {
        return true;
    }
    if node.is_error() {
        return false;
    }
    let text = source[node.start_byte()..node.end_byte()].trim();
    text.is_empty() || text == ";"
}

impl SyntaxErrorsEngine {
    pub(crate) fn check_file_structure(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut cursor = root.walk();
        let children: Vec<Node> = root.children(&mut cursor).collect();
        let at = |rule_id: &'static str, message: &str, node: Node| {
            let pos = node.start_position();
            Diagnostic {
                rule_id,
                message: message.to_string(),
                severity: Severity::Error,
                byte_range: node.byte_range(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            }
        };

        // CLTWO: every class after the first, at its name. Probed: with three
        // classes in a file, the second and the third are reported.
        if self.is_check_enabled("CLTWO") {
            for class in children
                .iter()
                .filter(|c| c.kind() == "class_definition")
                .skip(1)
            {
                let name = class.child_by_field_name("name").unwrap_or(*class);
                diagnostics.push(at("CLTWO", CLTWO_MESSAGE, name));
            }
        }

        // CLIS: a class after script statements, reported at the `classdef` line.
        // Probed: `x = 1;` then `classdef` is CLIS. Function definitions do not make
        // a script, and a class that heads the file is a class file (SOFOC below).
        if self.is_check_enabled("CLIS") {
            let mut script_statement = false;
            for child in &children {
                if child.kind() == "class_definition" {
                    if script_statement {
                        diagnostics.push(at("CLIS", CLIS_MESSAGE, *child));
                        break;
                    }
                } else if !child.is_error()
                    && child.kind() != "function_definition"
                    && !is_empty_statement(*child, source)
                {
                    script_statement = true;
                }
            }
        }

        // SOFOC: a class file (the class heads the file) with a statement after the
        // class. Probed: one report, at the first such statement; functions after the
        // class are allowed.
        if self.is_check_enabled("SOFOC") {
            let first = children.iter().find(|c| !is_empty_statement(**c, source));
            if let Some(class) = first.filter(|c| c.kind() == "class_definition") {
                let statement = children
                    .iter()
                    .skip_while(|c| c.id() != class.id())
                    .skip(1)
                    .copied()
                    .find(|c| {
                        !c.is_error()
                            && !is_empty_statement(*c, source)
                            && !matches!(c.kind(), "function_definition" | "class_definition")
                    });
                if let Some(statement) = statement {
                    diagnostics.push(at("SOFOC", SOFOC_MESSAGE, statement));
                }
            }
        }

        // SEMFU: a script whose statements are only bare `;` and function
        // definitions, with a `;` before its first function. Probed: `;` then a
        // function reports at the `;`. `;` alone, or `;` with `x = 1;`, does not, and
        // a file that starts with a function is a function file.
        if self.is_check_enabled("SEMFU") {
            let mut first_semicolon = None;
            let mut has_function = false;
            let mut only_empty_and_functions = true;
            for child in &children {
                if child.kind() == "comment" {
                    continue;
                }
                let text = source[child.start_byte()..child.end_byte()].trim();
                if text.is_empty() {
                    continue;
                }
                if child.kind() == "function_definition" {
                    has_function = true;
                } else if !child.is_error() && text == ";" {
                    if !has_function && first_semicolon.is_none() {
                        first_semicolon = Some(*child);
                    }
                } else {
                    only_empty_and_functions = false;
                    break;
                }
            }
            if let Some(semicolon) = first_semicolon {
                if has_function && only_empty_and_functions {
                    diagnostics.push(at("SEMFU", SEMFU_MESSAGE, semicolon));
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

    /// The line of the first diagnostic with `id`, if any.
    fn line_of(diags: &[Diagnostic], id: &str) -> Option<usize> {
        diags.iter().find(|d| d.rule_id == id).map(|d| d.line)
    }

    // -- CLIS: class after script statements --------------------------------

    #[test]
    fn clis_fires_on_class_after_script_statement() {
        // Probed: MATLAB reports CLIS at the `classdef` line (2).
        let diags = lint_file(&*engine(), "x = 1;\nclassdef Foo\nend\n");
        assert_eq!(line_of(&diags, "CLIS"), Some(2), "got: {diags:?}");
        assert!(!has_id(&diags, "SOFOC"), "got: {diags:?}");
    }

    #[test]
    fn clis_ok_on_plain_class_file() {
        let diags = lint_file(&*engine(), "classdef Foo\nend\n");
        assert!(!has_id(&diags, "CLIS"), "got: {diags:?}");
    }

    #[test]
    fn clis_ok_on_class_heading_a_file_with_statements_after_it() {
        // Probed (CLIS_test.m): a class that heads the file is SOFOC, not CLIS.
        let diags = lint_file(&*engine(), "classdef Foo\nend\nx = 5;\n");
        assert!(!has_id(&diags, "CLIS"), "got: {diags:?}");
    }

    // -- CLTWO: multiple class definitions ----------------------------------

    #[test]
    fn cltwo_fires_on_two_classes_at_the_second_name() {
        // Probed (Two_a.m): the second class is reported at its name, line 3.
        let diags = lint_file(&*engine(), "classdef Foo\nend\nclassdef Bar\nend\n");
        assert_eq!(line_of(&diags, "CLTWO"), Some(3), "got: {diags:?}");
    }

    #[test]
    fn cltwo_fires_for_each_class_after_the_first() {
        // Probed (Two_b2.m): three classes report lines 3 and 5.
        let diags = lint_file(
            &*engine(),
            "classdef Foo\nend\nclassdef Bar\nend\nclassdef Baz\nend\n",
        );
        let lines: Vec<usize> = diags
            .iter()
            .filter(|d| d.rule_id == "CLTWO")
            .map(|d| d.line)
            .collect();
        assert_eq!(lines, vec![3, 5], "got: {diags:?}");
    }

    #[test]
    fn cltwo_ok_on_single_class() {
        let diags = lint_file(&*engine(), "classdef Foo\nend\n");
        assert!(!has_id(&diags, "CLTWO"), "got: {diags:?}");
    }

    // -- SOFOC: statements outside class in class file ----------------------

    #[test]
    fn sofoc_fires_on_statement_after_class_at_its_line() {
        // Probed (SofocX.m): the function is fine, the statement on line 5 is SOFOC.
        let diags = lint_file(&*engine(), "classdef Foo\nend\nfunction f()\nend\nx = 5;\n");
        assert_eq!(line_of(&diags, "SOFOC"), Some(5), "got: {diags:?}");
    }

    #[test]
    fn sofoc_reports_only_the_first_statement_after_class() {
        // Probed (ClisD.m): `x = 1; y = 2;` after a class is one SOFOC, on line 3.
        let diags = lint_file(&*engine(), "classdef Foo\nend\nx = 1;\ny = 2;\n");
        let count = diags.iter().filter(|d| d.rule_id == "SOFOC").count();
        assert_eq!(
            (count, line_of(&diags, "SOFOC")),
            (1, Some(3)),
            "got: {diags:?}"
        );
    }

    #[test]
    fn sofoc_ok_on_class_with_methods() {
        let diags = lint_file(
            &*engine(),
            "classdef Foo\nmethods\nfunction f()\nend\nend\nend\n",
        );
        assert!(!has_id(&diags, "SOFOC"), "got: {diags:?}");
    }

    #[test]
    fn sofoc_ok_on_class_followed_only_by_functions() {
        // Probed (ClisE.m): a function after the class is silent.
        let diags = lint_file(&*engine(), "classdef Foo\nend\nfunction f()\nend\n");
        assert!(!has_id(&diags, "SOFOC"), "got: {diags:?}");
    }

    // -- SEMFU: file with only empty statements -----------------------------

    #[test]
    fn semfu_ok_on_comment_only_file() {
        // Probed (semfu_comment_only.m): MATLAB is silent on a file of comments.
        let diags = lint_file(&*engine(), "% just a comment\n");
        assert!(!has_id(&diags, "SEMFU"), "got: {diags:?}");
    }

    #[test]
    fn semfu_fires_on_semicolon_then_function_at_the_semicolon() {
        // Probed (func_after_semi.m): the leading `;` makes a script with no code.
        let diags = lint_file(&*engine(), ";\nfunction f()\nend\n");
        assert_eq!(line_of(&diags, "SEMFU"), Some(1), "got: {diags:?}");
    }

    #[test]
    fn semfu_points_at_the_first_semicolon_after_comments() {
        // Probed (semfu_f.m): the first `;` is on line 3.
        let diags = lint_file(&*engine(), "% c\n\n;\n;\nfunction f()\nend\n");
        assert_eq!(line_of(&diags, "SEMFU"), Some(3), "got: {diags:?}");
    }

    #[test]
    fn semfu_ok_on_semicolons_alone_and_with_code() {
        // Probed: MATLAB is silent on `;` alone and on `;` followed by `x = 1;`.
        for src in [";\n\n;\n", ";\nx = 1;\n", "function f()\nend\n"] {
            let diags = lint_file(&*engine(), src);
            assert!(!has_id(&diags, "SEMFU"), "{src:?} got: {diags:?}");
        }
    }

    #[test]
    fn semfu_ok_on_real_content() {
        let diags = lint_file(&*engine(), "x = 5;\n");
        assert!(!has_id(&diags, "SEMFU"), "got: {diags:?}");
    }
}
