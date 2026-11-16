//! `check_statements` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_statements(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Collect function names and variable assignments for REDEF.
        let mut function_names: HashSet<String> = HashSet::new();
        let mut variable_names: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut redef_reported: HashSet<String> = HashSet::new();

        // REDEF: a top-level function defined again later in the file.
        let mut first_definitions: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() != "function_definition" {
                continue;
            }
            let Some(name_node) = child.child_by_field_name("name") else {
                continue;
            };
            let name = source[name_node.start_byte()..name_node.end_byte()].to_string();
            if let Some(&first_line) = first_definitions.get(&name) {
                if self.is_check_enabled("REDEF") {
                    let pos = name_node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "REDEF",
                        message: format!(
                            "The current use of '{name}' is inconsistent with its previous use or definition (line {first_line})."
                        ),
                        severity: Severity::Error,
                        byte_range: name_node.byte_range(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            } else {
                first_definitions.insert(name, name_node.start_position().row + 1);
            }
        }

        // REDEF: a method defined twice in a class's methods blocks, constructors
        // included (probed: a second `DupCtor(x)` constructor is reported at its line).
        fn methods_of<'a>(node: Node<'a>, in_methods: bool, out: &mut Vec<Node<'a>>) {
            if node.kind() == "function_definition" {
                if in_methods {
                    out.push(node);
                }
                return;
            }
            let in_methods = in_methods || node.kind() == "methods";
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                methods_of(child, in_methods, out);
            }
        }
        let mut cursor = root.walk();
        for class in root
            .children(&mut cursor)
            .filter(|c| c.kind() == "class_definition")
        {
            let mut methods = Vec::new();
            methods_of(class, false, &mut methods);
            let mut first_lines: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for method in methods {
                let Some(name_node) = method.child_by_field_name("name") else {
                    continue;
                };
                let name = source[name_node.start_byte()..name_node.end_byte()].to_string();
                let pos = name_node.start_position();
                match first_lines.get(&name) {
                    Some(&first_line) => {
                        if self.is_check_enabled("REDEF") {
                            diagnostics.push(Diagnostic {
                                rule_id: "REDEF",
                                message: format!(
                                    "The current use of '{name}' is inconsistent with its previous use or definition (line {first_line})."
                                ),
                                severity: Severity::Error,
                                byte_range: name_node.byte_range(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    }
                    None => {
                        first_lines.insert(name, pos.row + 1);
                    }
                }
            }
        }

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
        variable_names: &mut std::collections::HashMap<String, usize>,
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
                    && variable_names.contains_key(&name)
                    && !redef_reported.contains(&name)
                {
                    let pos = name_node.start_position();
                    let def_line = variable_names[&name];
                    diagnostics.push(Diagnostic {
                        rule_id: "REDEF",
                        message: format!(
                            "The current use of '{}' is inconsistent with its previous use or definition (line {}).",
                            name, def_line
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

        // REDEF: a command-syntax word that is a variable (`psd x` after `psd = 5`).
        if kind == "command" && self.is_check_enabled("REDEF") {
            let text = &source[node.start_byte()..node.end_byte()];
            let command_word = text.split_whitespace().next().filter(|w| {
                w.chars().all(|c| c.is_alphanumeric() || c == '_')
                    && text[w.len()..].starts_with([' ', '\t'])
            });
            if let Some(name) = command_word {
                // `i += 1` and `j -= 1` are SYNER (walk_op_assign_commands), not REDEF.
                if let Some(&def_line) = variable_names
                    .get(name)
                    .filter(|_| !matches!(name, "i" | "j"))
                {
                    if !redef_reported.contains(name) {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "REDEF",
                            message: format!(
                                "The current use of '{name}' is inconsistent with its previous use or definition (line {def_line})."
                            ),
                            severity: Severity::Error,
                            byte_range: node.byte_range(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                        redef_reported.insert(name.to_string());
                    }
                }
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
                                "The current use of '{}' is inconsistent with its previous use or definition (line {}).",
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
                    variable_names
                        .entry(name)
                        .or_insert(lhs.start_position().row + 1);
                }
            }
        }

        // REDEF: a `for` loop variable is a definition too (`k += 1` in its body
        // is inconsistent with `for k = 1:3`, as with `k = 1; k += 1`).
        if kind == "for_statement" && self.is_check_enabled("REDEF") {
            let mut cursor = node.walk();
            let iterator = node
                .children(&mut cursor)
                .find(|child| child.kind() == "iterator");
            if let Some(name_node) = iterator.and_then(|it| it.named_child(0)) {
                if name_node.kind() == "identifier" {
                    let name = source[name_node.start_byte()..name_node.end_byte()].to_string();
                    variable_names
                        .entry(name)
                        .or_insert(name_node.start_position().row + 1);
                }
            }
        }

        // SEPEXR: Missing separator between statements.
        // Check consecutive children of block/source_file that are statements
        // on the same line without separator.
        if self.is_check_enabled("SEPEXR") && (kind == "source_file" || kind == "block") {
            // Compare statements only. The `;` and `,` separators are children
            // of the block too (anonymous nodes): pairing a separator with the
            // next statement would see an empty gap and report a separator
            // as missing on every `x = 1; y = 2;`.
            let mut cursor = node.walk();
            let children: Vec<Node> = node
                .children(&mut cursor)
                .filter(|child| child.is_named())
                .collect();

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

        // A function body is its own scope: its variables are not compared with the
        // file's functions or with another function's variables (probed: `function
        // f(); x = 1; end` next to `function x(); end` is silent).
        if kind == "function_definition" {
            let mut scope_functions = HashSet::new();
            let mut scope_variables = std::collections::HashMap::new();
            let mut scope_reported = HashSet::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.walk_statements(
                    child,
                    source,
                    diagnostics,
                    &mut scope_functions,
                    &mut scope_variables,
                    &mut scope_reported,
                );
            }
            return;
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
    fn sepexr_leaves_missing_separators_to_the_parser() {
        // A statement run into the previous one is a parse error, reported as
        // SYNER; and a body that follows its condition is SEPEX's business.
        for src in ["if x y = 1; end\n", "x = 1; 2\n"] {
            let diags = lint_file(&*engine(), src);
            assert!(!has_id(&diags, "SEPEXR"), "{src:?} got: {diags:?}");
        }
    }

    #[test]
    fn sepexr_silent_when_a_separator_is_present() {
        for src in [
            "x = 1; y = 2;\n",
            "x = 1;y = 2;\n",
            "x = 1, y = 2;\n",
            "x = 1;; y = 2;\n",
            "disp(1); disp(2);\n",
            "if a, b = 1; end; z = 3;\n",
        ] {
            let diags = lint_file(&*engine(), src);
            assert!(!has_id(&diags, "SEPEXR"), "{src:?} got: {diags:?}");
        }
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
    fn redef_ok_for_function_name_assigned_in_its_body() {
        // Probed (fnclash.m, rd_self_x.m): MATLAB is silent; only NASGU/FNDEF.
        let diags = lint_file(&*engine(), "function foo()\n    foo = 1;\nend\n");
        assert!(!has_id(&diags, "REDEF"), "got: {diags:?}");
    }

    #[test]
    fn redef_ok_on_distinct_names() {
        let diags = lint_file(&*engine(), "function foo()\n    x = 1;\nend\n");
        assert!(!has_id(&diags, "REDEF"), "got: {diags:?}");
    }
}

#[cfg(test)]
mod redef_tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        SyntaxErrorsEngine::from_config(&Config::default())
    }

    #[test]
    fn redef_fires_for_duplicate_top_level_function() {
        let diags = lint_file(&*engine(), "function f1()\nend\nfunction f1()\nend\n");
        assert!(has_id(&diags, "REDEF"));
    }

    #[test]
    fn redef_does_not_fire_for_distinct_functions() {
        let diags = lint_file(&*engine(), "function f1()\nend\nfunction f2()\nend\n");
        assert!(!has_id(&diags, "REDEF"));
    }

    #[test]
    fn redef_fires_for_command_syntax_on_variable() {
        let diags = lint_file(&*engine(), "psd = 5;\npsd x\n");
        assert!(has_id(&diags, "REDEF"));
    }

    #[test]
    fn redef_fires_for_augmented_assignment_on_loop_variable() {
        // MATLAB probe: `for k=1:3` then `k += 1;` in the body reports REDEF at
        // `k` on the `+=` line (`k += 1` parses as a command here).
        let diags = lint_file(&*engine(), "for k=1:3\n  k += 1;\nend\n");
        let redef = diags.iter().find(|d| d.rule_id == "REDEF");
        assert_eq!(redef.map(|d| d.line), Some(2), "got: {diags:?}");
    }

    #[test]
    fn redef_fires_for_augmented_assignment_after_the_loop() {
        // MATLAB probe: the loop variable stays defined after the loop ends.
        let diags = lint_file(&*engine(), "for k=1:3\nend\nk += 1;\n");
        assert!(has_id(&diags, "REDEF"), "got: {diags:?}");
    }

    #[test]
    fn redef_ok_for_loop_variable_used_as_an_index() {
        // An assignment through the loop variable is not a command word.
        let diags = lint_file(&*engine(), "for k=1:3\n  k(2) = 1;\nend\n");
        assert!(!has_id(&diags, "REDEF"), "got: {diags:?}");
    }
}

#[cfg(test)]
mod redef_scope_tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        SyntaxErrorsEngine::from_config(&Config::default())
    }

    fn redef_lines(src: &str) -> Vec<usize> {
        lint_file(&*engine(), src)
            .iter()
            .filter(|d| d.rule_id == "REDEF")
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn redef_fires_for_duplicate_constructor() {
        // Probed (DupCtor.m): the second constructor, line 5, is reported.
        let src = "classdef DupCtor\nmethods\nfunction obj = DupCtor()\nend\nfunction obj = DupCtor(x)\nend\nend\nend\n";
        assert_eq!(redef_lines(src), vec![5]);
    }

    #[test]
    fn redef_fires_for_duplicate_method() {
        // Probed (DupMeth.m): the second `f`, line 6.
        let src = "classdef DupMeth\nmethods\nfunction r = f(obj)\nr = 1;\nend\nfunction r = f(obj)\nr = 2;\nend\nend\nend\n";
        assert_eq!(redef_lines(src), vec![6]);
    }

    #[test]
    fn redef_ok_for_distinct_methods_and_constructor() {
        let src = "classdef Ok\nmethods\nfunction obj = Ok()\nend\nfunction r = g(obj)\nr = 1;\nend\nend\nend\n";
        assert!(redef_lines(src).is_empty());
    }

    #[test]
    fn redef_ok_for_variable_and_function_in_different_scopes() {
        // Probed (rd_scope_two.m, rd_var_fn.m): MATLAB is silent.
        let src = "function a()\nx = 1;\nend\nfunction b()\nx = 2;\nend\nfunction x()\nend\n";
        assert!(redef_lines(src).is_empty(), "{:?}", redef_lines(src));
    }

    #[test]
    fn redef_ok_for_ans_function_assigning_ans() {
        // Probed (rd_ansf.m): MATLAB reports FCNANS, not REDEF.
        assert!(redef_lines("function ans()\nans = 2;\nend\n").is_empty());
    }

    #[test]
    fn redef_fires_for_script_variable_then_function() {
        // Probed (rd_script_fn.m): the function name at line 2 is redefined.
        assert_eq!(redef_lines("x = 1;\nfunction x()\nend\n"), vec![2]);
    }

    #[test]
    fn redef_fires_for_command_word_in_a_function() {
        // Probed (rd_cmd.m): `psd x` after `psd = 5` inside a function.
        assert_eq!(
            redef_lines("function rd_cmd()\npsd = 5;\npsd x\nend\n"),
            vec![3]
        );
        assert!(has_id(
            &lint_file(&*engine(), "function f()\npsd = 5;\npsd x\nend\n"),
            "REDEF"
        ));
    }
}
