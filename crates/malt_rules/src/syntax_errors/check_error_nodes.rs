//! `check_error_nodes` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_error_nodes(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let last = Self::last_descendant(root);
        self.walk_error_nodes(root, source, last, &mut diagnostics);
        self.walk_dotted_name_value_calls(root, source, &mut diagnostics);

        // ENDPAR: Check if the file ends with a missing closing bracket.
        if self.is_check_enabled("ENDPAR") {
            if let Some(last) = last {
                let start = last.start_byte();
                let end = last.end_byte().max(start + 1);
                let text = &source[start..end.min(source.len())];
                let is_bracket_error = last.is_error() && Self::looks_like_missing_bracket(text);
                let missing_kind = Self::missing_bracket_opener(last.kind());
                if is_bracket_error || (last.is_missing() && missing_kind.is_some()) {
                    let pos = last.start_position();
                    let bracket = Self::bracket_kinds(text)
                        .first()
                        .copied()
                        .or(missing_kind)
                        .unwrap_or("bracket");
                    diagnostics.push(Diagnostic {
                            rule_id: "ENDPAR",
                            message: format!(
                                "A {bracket} might be missing a closing {bracket}, causing invalid syntax at end of file."
                            ),
                            severity: Severity::Error,
                            byte_range: start..end,
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                }
            }
        }

        diagnostics
    }

    pub(crate) fn walk_error_nodes(
        &self,
        node: Node,
        source: &str,
        last: Option<Node>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.is_error() {
            let start = node.start_byte();
            let end = node.end_byte().max(start + 1);
            let pos = node.start_position();
            let text = &source[start..end.min(source.len())];

            // Classify the error node.
            let endct_variant = if self.is_check_enabled("ENDCT") {
                Self::classify_missing_end(&node, source)
            } else {
                None
            };

            // FVSYN: `{@...}` validation syntax in an arguments block.
            if Self::is_lambda_in_arguments(&node, source) {
                if self.is_check_enabled("FVSYN") {
                    diagnostics.push(Diagnostic {
                        rule_id: "FVSYN",
                        message: "Invalid function argument syntax at '@'.".to_string(),
                        severity: Severity::Error,
                        byte_range: start..end,
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
            // NOPAR2: Looks like a missing closing bracket (not in a property: MCPLD).
            else if !Self::inside_property(node) && Self::looks_like_missing_bracket(text) {
                let is_last = last.is_some_and(|l| l.id() == node.id());
                if self.is_check_enabled("NOPAR2") && !is_last {
                    let bracket = Self::bracket_kinds(text)
                        .first()
                        .copied()
                        .unwrap_or("bracket");
                    diagnostics.push(Diagnostic {
                            rule_id: "NOPAR2",
                            message: format!(
                                "A {bracket} might be missing a closing {bracket}, causing invalid syntax at {bracket} on line {}.",
                                pos.row + 1
                            ),
                            severity: Severity::Error,
                            byte_range: start..end,
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                }
            }
            // ENDCT: Looks like a missing END.
            else if let Some((variant_id, variant_msg)) = endct_variant {
                if self.is_check_enabled(variant_id) {
                    diagnostics.push(Diagnostic {
                        rule_id: variant_id,
                        message: variant_msg,
                        severity: Severity::Error,
                        byte_range: start..end,
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                } else {
                    diagnostics.push(Diagnostic {
                        rule_id: "ENDCT",
                        message: endct_message(text),
                        severity: Severity::Error,
                        byte_range: start..end,
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            } else if self.is_check_enabled("ENDCT") && Self::looks_like_missing_end(text, &node) {
                diagnostics.push(Diagnostic {
                    rule_id: "ENDCT",
                    message: endct_message(text),
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
            // FVAPN: a positional argument after a name=value argument in a call.
            // Other ERROR nodes in a call's argument list fall through to SYNER.
            else if self.is_check_enabled("FVAPN")
                && Self::is_in_function_call_args(&node)
                && Self::follows_name_value_argument(&node, source)
            {
                diagnostics.push(Diagnostic {
                    rule_id: "FVAPN",
                    message: "Move name-value arguments that use the name=value syntax to the end of the argument list.".to_string(),
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
            // FVSYN: Invalid function argument syntax inside a call's argument list.
            // A value glued to a name (`1x=2`) is a SYNER instead (MATLAB: at `x`).
            else if self.is_check_enabled("FVSYN")
                && Self::is_in_function_call_args(&node)
                && !text.contains('=')
                && !source[end..].starts_with(char::is_alphabetic)
            {
                diagnostics.push(Diagnostic {
                    rule_id: "FVSYN",
                    message: format!(
                        "Invalid function argument syntax at {}.",
                        text.chars().take(40).collect::<String>().trim()
                    ),
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
            // ENDCT2: a one-line `try` whose `catch ID end` has no separator before
            // the END. MATLAB reports the missing END at the TRY, not a parse error.
            else if is_try_catch_end_line(text) {
                if self.is_check_enabled("ENDCT2") {
                    diagnostics.push(Diagnostic {
                        rule_id: "ENDCT2",
                        message: format!(
                            "An END might be missing (after line {}), possibly matching TRY.",
                            pos.row + 1
                        ),
                        severity: Severity::Error,
                        byte_range: start..end,
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
            // SYNER: Generic syntax error.
            else if self.is_check_enabled("SYNER")
                && !Self::is_unreported_syner(&node, text, source)
            {
                let snippet: String = text.chars().take(40).collect();
                let msg = if snippet.is_empty() {
                    "Parse error: usage might be invalid MATLAB syntax.".to_string()
                } else {
                    format!("Parse error at {snippet}: usage might be invalid MATLAB syntax.")
                };
                diagnostics.push(Diagnostic {
                    rule_id: "SYNER",
                    message: msg,
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        } else if node.is_missing() {
            if let Some(bracket) = Self::missing_bracket_opener(node.kind()) {
                let is_last = last.is_some_and(|l| l.id() == node.id());
                if !is_last {
                    let start = node.start_byte();
                    let end = node.end_byte().max(start + 1);
                    let pos = node.start_position();
                    if Self::is_end_of_file(node, source) {
                        if self.is_check_enabled("ENDPAR") {
                            diagnostics.push(Diagnostic {
                                    rule_id: "ENDPAR",
                                    message: format!(
                                        "A '{bracket}' might be missing a closing '{closer}', causing invalid syntax at end of file.",
                                        closer = node.kind()
                                    ),
                                    severity: Severity::Error,
                                    byte_range: start..end,
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                        }
                    } else if Self::is_line_ending(node, source) {
                        if self.is_check_enabled("EOLPAR") {
                            diagnostics.push(Diagnostic {
                                    rule_id: "EOLPAR",
                                    message: format!(
                                        "A '{bracket}' might be missing a closing '{closer}', causing invalid syntax at end of line.",
                                        closer = node.kind()
                                    ),
                                    severity: Severity::Error,
                                    byte_range: start..end,
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                        }
                    } else if let Some((open, token)) = self.nopar_opener(node, source, bracket) {
                        diagnostics.push(Diagnostic {
                            rule_id: "NOPAR",
                            message: format!(
                                "A '{bracket}' might be missing a closing '{closer}', causing invalid syntax at '{token}'.",
                                closer = node.kind()
                            ),
                            severity: Severity::Error,
                            byte_range: open..open + 1,
                            line: pos.row + 1,
                            column: open + 1 - source[..open].rfind('\n').map_or(0, |i| i + 1),
                            fix: None,
                        });
                    } else if self.is_check_enabled("NOPAR2") {
                        diagnostics.push(Diagnostic {
                                rule_id: "NOPAR2",
                                message: format!(
                                    "A {bracket} might be missing a closing {bracket}, causing invalid syntax at {bracket} on line {}.",
                                    pos.row + 1
                                ),
                                severity: Severity::Error,
                                byte_range: start..end,
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                    }
                }
            } else if node.kind() == "end"
                && source[node.start_byte()..]
                    .trim_start()
                    .starts_with("function")
                && Self::enclosing_arguments_statement(&node).is_some()
            {
                // A missing END before a FUNCTION inside an arguments block: MATLAB
                // reports ENDCT3 at the ARGUMENTS keyword and no SYNER here.
                if let (true, Some(block)) = (
                    self.is_check_enabled("ENDCT3"),
                    Self::enclosing_arguments_statement(&node),
                ) {
                    let pos = block.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "ENDCT3",
                        message: format!(
                            "An END might be missing (before FUNCTION on line {}), possibly matching ARGUMENTS.",
                            node.start_position().row + 1
                        ),
                        severity: Severity::Error,
                        byte_range: block.byte_range(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            } else if self.is_check_enabled("SYNER")
                // MATLAB does not report a missing `,` or `;` as a syntax error
                // (probe: `x = 1; 2` with no final newline gets no SYNER).
                && !matches!(node.kind(), "," | ";")
            {
                let start = node.start_byte();
                let end = node.end_byte().max(start + 1);
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "SYNER",
                    message: format!(
                        "Parse error at {}: usage might be invalid MATLAB syntax.",
                        node.kind()
                    ),
                    severity: Severity::Error,
                    byte_range: start..end,
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_error_nodes(child, source, last, diagnostics);
        }
    }
}

impl SyntaxErrorsEngine {
    /// A statement-level call with a dotted name in name=value form, `f(a.b=1);`.
    /// Tree-sitter leaves `=1` as an ERROR after the arguments. MATLAB reports
    /// NOPAR at the `(` (the `=` is the offending token) and SYNER at the `)`
    /// that closes the call. The same call inside an assignment (`x = f(a.b=1)`)
    /// is not covered here.
    fn walk_dotted_name_value_calls(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "function_call"
            && matches!(
                node.parent().map(|p| p.kind()),
                Some("source_file" | "block")
            )
        {
            let name = node.child_by_field_name("name");
            // The closing `)` is an anonymous child after the ERROR node.
            let mut cursor = node.walk();
            let error = node.children(&mut cursor).find(|c| c.is_error());
            let has_dotted_argument = node.children(&mut node.walk()).any(|c| {
                c.kind() == "arguments"
                    && c.named_child(c.named_child_count().saturating_sub(1))
                        .is_some_and(|a| a.kind() == "field_expression")
            });
            if let (Some(name), Some(error), true) = (name, error, has_dotted_argument) {
                let open = name.end_byte();
                let is_name_value_error = error.is_error()
                    && source[error.start_byte()..].starts_with('=')
                    && source.as_bytes().get(open) == Some(&b'(');
                let close = source[error.end_byte()..]
                    .find(')')
                    .map(|i| error.end_byte() + i);
                if let (true, Some(close)) = (is_name_value_error, close) {
                    if self.is_check_enabled("NOPAR") {
                        let line_start = source[..open].rfind('\n').map_or(0, |i| i + 1);
                        let pos = name.end_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "NOPAR",
                            message: "A '(' might be missing a closing ')', causing invalid syntax at '='.".to_string(),
                            severity: Severity::Error,
                            byte_range: open..open + 1,
                            line: pos.row + 1,
                            column: open + 1 - line_start,
                            fix: None,
                        });
                    }
                    if self.is_check_enabled("SYNER") {
                        let line_start = source[..close].rfind('\n').map_or(0, |i| i + 1);
                        let line = source[..close].matches('\n').count() + 1;
                        diagnostics.push(Diagnostic {
                            rule_id: "SYNER",
                            message: "Parse error at ')': usage might be invalid MATLAB syntax."
                                .to_string(),
                            severity: Severity::Error,
                            byte_range: close..close + 1,
                            line,
                            column: close + 1 - line_start,
                            fix: None,
                        });
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_dotted_name_value_calls(child, source, diagnostics);
        }
    }

    /// The `arguments` statement that encloses `node`, if any.
    fn enclosing_arguments_statement<'a>(node: &Node<'a>) -> Option<Node<'a>> {
        let mut current = node.parent();
        while let Some(ancestor) = current {
            if ancestor.kind() == "arguments_statement" {
                return Some(ancestor);
            }
            current = ancestor.parent();
        }
        None
    }

    /// True for a lambda (`@`) validator inside an `arguments` block.
    fn is_lambda_in_arguments(node: &Node, source: &str) -> bool {
        source[node.start_byte()..].starts_with('@')
            && Self::enclosing_arguments_statement(node).is_some()
    }

    /// True when the ERROR node follows a `name=value` argument of the same call.
    fn follows_name_value_argument(node: &Node, source: &str) -> bool {
        let mut current = node.parent();
        while let Some(ancestor) = current {
            if ancestor.kind() == "function_call" {
                return source[ancestor.start_byte()..node.start_byte()].contains('=');
            }
            current = ancestor.parent();
        }
        false
    }

    /// True for an ERROR node that MATLAB accepts. Tree-sitter does not parse
    /// these, so SYNER would be a false positive. Probed against R2026b:
    /// - validation syntax inside an `arguments` block (`x (1,2) double`,
    ///   `arguments (Input = true)`, `opts.?cls = 1`);
    /// - colon chains such as `1:2:3:4` (MATLAB reports M3COL instead);
    /// - `spmd (1, 2, 3)` (MATLAB reports SPDEC3 instead).
    ///
    /// NOPAR: an unclosed `(` or `{` whose closer is missing on the opener's
    /// line, so the offending token is on that line too. Returns the opener's
    /// byte offset and the offending token. Probed against R2026b: a later line
    /// gives NOPAR2, the end of the line EOLPAR, and `[` is not covered.
    fn nopar_opener(&self, node: Node, source: &str, bracket: &str) -> Option<(usize, String)> {
        if !self.is_check_enabled("NOPAR") || !matches!(bracket, "(" | "{") {
            return None;
        }
        let at = node.start_byte();
        let line_start = source[..at].rfind('\n').map_or(0, |i| i + 1);
        let open = line_start + source[line_start..at].find(bracket)?;
        let rest = &source[at..];
        let token: String = if rest.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
            rest.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect()
        } else {
            rest.chars().take(1).collect()
        };
        if token.is_empty() || token.starts_with(char::is_whitespace) {
            return None;
        }
        Some((open, token))
    }

    fn is_known_valid_error(node: &Node, source: &str) -> bool {
        let mut current = node.parent();
        while let Some(ancestor) = current {
            // An attribute list that does not parse (a line break in it) is a SYNER.
            if ancestor.kind() == "attributes" {
                break;
            }
            if ancestor.kind() == "arguments_statement" {
                return true;
            }
            current = ancestor.parent();
        }
        if Self::is_colon_chain(node, source) {
            return true;
        }
        let line_start = source[..node.start_byte()].rfind('\n').map_or(0, |i| i + 1);
        let line = source[line_start..]
            .lines()
            .next()
            .unwrap_or("")
            .trim_start();
        line.strip_prefix("spmd")
            .is_some_and(|rest| rest.starts_with([' ', '\t', '(']))
    }

    /// True when the error token sits in a colon chain of plain operands, such
    /// as `1:2:3:4` (the error token is `:4`), or `:3:4:5` at the start of a range.
    fn is_colon_chain(node: &Node, source: &str) -> bool {
        let is_operand_char = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | ':');
        let start = source[..node.start_byte()]
            .char_indices()
            .rev()
            .take_while(|(_, c)| is_operand_char(*c))
            .last()
            .map_or(node.start_byte(), |(i, _)| i);
        let end = source[node.start_byte()..]
            .find(|c: char| !is_operand_char(c))
            .map_or(source.len(), |i| node.start_byte() + i);
        let run = &source[start..end];
        run.matches(':').count() >= 2 && !run.contains("::") && !run.ends_with(':')
    }
}

impl SyntaxErrorsEngine {
    /// True when an ERROR node must not be reported as SYNER: a name=value call
    /// argument, a known valid form, or text that MATLAB reports under another ID.
    fn is_unreported_syner(node: &Node, text: &str, source: &str) -> bool {
        (Self::is_in_function_call_args(node) && text.contains('='))
            || Self::is_known_valid_error(node, source)
            || is_unreported_syner_text(text)
            || is_value_end_error(node, text, source)
            // A property with a validator or an index is MCPLD, not a parse error.
            || Self::inside_property(*node)
    }
}

/// True for a one-line `try` whose `catch ID end` has no separator before the END
/// (`try, x = 1; catch ME end`). The ERROR text starts at the `try`.
fn is_try_catch_end_line(text: &str) -> bool {
    let line = text.lines().next().unwrap_or("").trim_start();
    let Some(rest) = line.strip_prefix("try") else {
        return false;
    };
    if rest.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
        return false;
    }
    let Some(pos) = rest.find("catch") else {
        return false;
    };
    let after = rest[pos + "catch".len()..].trim_start();
    let ident_len = after
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(after.len());
    let tail = &after[ident_len..];
    if ident_len == 0 || !tail.starts_with(char::is_whitespace) {
        return false;
    }
    let tail = tail.trim_start();
    tail.strip_prefix("end")
        .is_some_and(|r| !r.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
}

/// True for an ERROR node that holds a value `end` after `=` or `(`, such as
/// `y = end + 1;` or `y = (end);`. The grammar ends the assignment at the `end`;
/// MATLAB reports IDXCOLND there, not a parse error, and check_end_dfs reports it.
/// A SYNER here would also drop that IDXCOLND with the rest of the statement.
fn is_value_end_error(node: &Node, text: &str, source: &str) -> bool {
    // The error span can run on past `end` into the operator that follows (`end*`).
    let body = text.trim_end_matches(|c: char| {
        c == ')' || c == '*' || c == '/' || c == '^' || c == '+' || c == '-' || c.is_whitespace()
    });
    let Some(before) = body.strip_suffix("end") else {
        return false;
    };
    let prefix = source[..node.start_byte() + before.len()].trim_end();
    let prefix = prefix.strip_suffix('(').map_or(prefix, str::trim_end);
    let Some(before_eq) = prefix.strip_suffix('=') else {
        return false;
    };
    // `==`, `~=`, `<=`, `>=`, `+=` and the like are operators, not assignments.
    !before_eq.trim_end().ends_with([
        '=', '~', '<', '>', '!', '+', '-', '*', '/', '^', '&', '|', ':', '.',
    ])
}

/// True for ERROR text that MATLAB does not report as SYNER: a multi-line body that
/// starts at `function` (tree-sitter could not parse it, and the token MATLAB names
/// is inside it), text with a NUL byte (MATLAB reads it as the end of the text), and
/// `= end` (IDXCOLND in MATLAB, not a parse error).
fn is_unreported_syner_text(text: &str) -> bool {
    let function_body = text.contains('\n')
        && text
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("function");
    function_body
        || text.contains('\0')
        || text.trim().eq_ignore_ascii_case("= end")
        || is_colon_chain(text)
}

/// True for a one-line colon chain of four or more elements (`(1:2:3:4)`). The
/// grammar cannot parse it; MATLAB reports M3COL there and no SYNER (probed).
fn is_colon_chain(text: &str) -> bool {
    !text.contains('\n')
        && text.matches(':').count() >= 3
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || " _:.()[]=\t".contains(c))
}

/// The message for a missing END, naming the block keyword it probably closes.
fn endct_message(text: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "function", "if", "for", "parfor", "while", "switch", "try", "spmd",
    ];
    let keyword = text
        .split(|c: char| !c.is_alphanumeric())
        .find(|w| KEYWORDS.contains(w));
    match keyword {
        Some(k) => format!("An END might be missing, possibly matching {k}."),
        None => "An END might be missing.".to_string(),
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

    fn engine_with_disabled(checks: &[&str]) -> Box<dyn Rule> {
        let disabled = checks
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let config = Config::from_toml(&format!(
            "[lint.rules.SYNTAX_ERRORS_ENGINE]\ndisabled_checks = [{disabled}]\n"
        ))
        .unwrap();
        SyntaxErrorsEngine::from_config(&config)
    }

    // -- SYNER: generic syntax error ----------------------------------------

    #[test]
    fn syner_fires_on_parse_error() {
        let diags = lint_file(&*engine(), "x = ;\n");
        assert!(has_id(&diags, "SYNER"), "got: {diags:?}");
    }

    #[test]
    fn syner_ok_on_valid_source() {
        let diags = lint_file(&*engine(), "x = 5;\ny = x + 1;\n");
        assert!(!has_id(&diags, "SYNER"), "got: {diags:?}");
    }

    #[test]
    fn syner_ok_on_forms_that_matlab_accepts() {
        // Probed against MATLAB R2026b: none of these gets SYNER. The colon
        // chain is M3COL, `spmd (...)` is SPDEC3, and a missing final newline
        // after `2` is not reported.
        for source in [
            "a = 1:2:3:4:5;\n",
            "b = a(1:2:3:4);\n",
            "spmd (1, 2, 3)\nend\n",
            "x = 1; 2",
            "function r = f(x)\narguments\n    x (-1,2)\n    y {@(v) v > 0}\nend\nr = x;\nend\n",
            "function r = f(x)\narguments (Input = true)\n    x (1,:) double\nend\nr = x;\nend\n",
        ] {
            let diags = lint_file(&*engine(), source);
            assert!(!has_id(&diags, "SYNER"), "{source:?}: got: {diags:?}");
        }
    }

    #[test]
    fn syner_silent_where_a_lexical_check_names_the_error() {
        // MATLAB reports BADNE and TWOCM here, not a parse error. (`0xZ1` is
        // BADHBH in MATLAB, but malt does not emit BADHBH for it yet, so it is
        // not covered by this test.)
        for source in ["x = 1 != 2;\n", "x = [1,,2];\n"] {
            let diags = lint_file(&*engine(), source);
            assert!(!has_id(&diags, "SYNER"), "{source:?}: got: {diags:?}");
        }
        let diags = lint_file(&*engine(), "x = 1 != 2;\n");
        assert!(has_id(&diags, "BADNE"), "got: {diags:?}");
    }

    #[test]
    fn syner_not_reported_after_unterminated_string() {
        // MATLAB reports only STRIN for this file (probe); the parse error on the
        // next line is a cascade from the unterminated quote.
        let diags = lint_file(&*engine(), "x = 'abc\nc = \"q\";\n");
        assert!(has_id(&diags, "STRIN"), "got: {diags:?}");
        assert!(!has_id(&diags, "SYNER"), "got: {diags:?}");
    }

    // -- NOPAR2: missing closing bracket (mid-file) --------------------------

    #[test]
    fn nopar2_fires_on_unclosed_bracket_mid_file() {
        let diags = lint_file(&*engine(), "y = [1 2;\n");
        assert!(has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn nopar_fires_on_missing_paren_before_semicolon() {
        // MATLAB reports the `;` on the opener's line as the offending token: NOPAR.
        let diags = lint_file(&*engine(), "x = f(1;\ny = 2;\n");
        assert!(has_id(&diags, "NOPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn nopar2_ok_on_balanced_brackets() {
        let diags = lint_file(&*engine(), "y = [1 2];\n");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    // -- EOLPAR: missing closing bracket at end of line ----------------------

    #[test]
    fn missing_paren_before_newline_is_not_nopar() {
        let diags = lint_file(&*engine(), "function foo()\n    x = f(1\n    y = 2;\nend\n");
        assert!(!has_id(&diags, "NOPAR"), "got: {diags:?}");
    }

    #[test]
    fn missing_paren_before_newline_at_top_level_is_not_nopar() {
        let diags = lint_file(&*engine(), "x = f(1\ny = 2;\n");
        assert!(!has_id(&diags, "NOPAR"), "got: {diags:?}");
    }

    #[test]
    fn eolpar_ok_on_balanced_parens() {
        let diags = lint_file(&*engine(), "x = f(1);\n");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
    }

    // -- ENDPAR: missing closing bracket at end of file ----------------------

    #[test]
    fn endpar_fires_on_missing_paren_at_eof() {
        let diags = lint_file(&*engine(), "x = f(1 + 2;\n");
        assert!(has_id(&diags, "ENDPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
    }

    #[test]
    fn endpar_fires_on_missing_bracket_at_eof_no_newline() {
        let diags = lint_file(&*engine(), "x = f(1 + 2;");
        assert!(has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn endpar_ok_on_complete_file() {
        let diags = lint_file(&*engine(), "x = [1, 2];\n");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    // -- Missing bracket variants: must fire / must not fire -----------------

    #[test]
    fn missing_bracket_variants_must_fire_example() {
        let diags = lint_file(
            &*engine(),
            "x = f(1;\nfunction foo()\n    x = f(1;\n    y = 2;\nend\nx = f(1 + 2;\n",
        );
        assert!(has_id(&diags, "NOPAR"), "got: {diags:?}");
        assert!(has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn missing_bracket_variants_must_not_fire_example() {
        let diags = lint_file(&*engine(), "x = f(1);\nx = [1, 2];\nx = f(g(1));\n");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    #[test]
    fn nopar_fires_on_unclosed_call() {
        let diags = lint_file(&*engine(), "x = f(1;\ny = 1; 2");
        assert!(has_id(&diags, "NOPAR"), "got: {diags:?}");
    }

    #[test]
    fn missing_bracket_variants_respect_disabled_checks() {
        let engine = engine_with_disabled(&["NOPAR2"]);
        let diags = lint_file(&*engine, "y = [1 2;\n");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");

        let engine = engine_with_disabled(&["EOLPAR"]);
        let diags = lint_file(&*engine, "x = f(1;\ny = 2;\n");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");

        let engine = engine_with_disabled(&["ENDPAR"]);
        let diags = lint_file(&*engine, "x = f(1 + 2;\n");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
    }

    // -- ENDCT family: possible missing `end` -------------------------------

    #[test]
    fn endct2_fires_on_unterminated_if() {
        let diags = lint_file(&*engine(), "if x > 0\n    y = 1;\n");
        assert!(has_id(&diags, "ENDCT2"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
    }

    #[test]
    fn endct2_fires_on_unterminated_loops() {
        let diags = lint_file(&*engine(), "for i = 1:10\n    disp(i);\n");
        assert!(has_id(&diags, "ENDCT2"), "got: {diags:?}");
        let diags = lint_file(&*engine(), "while x\n    y = 1;\n");
        assert!(has_id(&diags, "ENDCT2"), "got: {diags:?}");
        let diags = lint_file(&*engine(), "switch x\n    case 1\n        y = 1;\n");
        assert!(has_id(&diags, "ENDCT2"), "got: {diags:?}");
    }

    #[test]
    fn endct2_ok_on_terminated_if() {
        let diags = lint_file(&*engine(), "if x > 0\n    y = 1;\nend\n");
        assert!(!has_id(&diags, "ENDCT2"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
    }

    #[test]
    fn endct3_fires_on_stray_else_followed_by_opener() {
        let diags = lint_file(&*engine(), "else\nif x\n");
        assert!(has_id(&diags, "ENDCT3"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
    }

    #[test]
    fn stray_elseif_in_function_is_not_endct() {
        // MATLAB probe: a stray `elseif` after a closed block is SYNER at that line.
        // It must not read as a missing `end` (`elseif` contains `if`).
        let diags = lint_file(
            &*engine(),
            "function t1(x, y)\nwhile true\n    x = x + 1;\nend\nelseif x, a = 2; end\n",
        );
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT2"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT3"), "got: {diags:?}");
    }

    #[test]
    fn endct3_ok_without_following_opener() {
        let diags = lint_file(&*engine(), "if x\n    y = 1;\nend\nelse\n");
        assert!(!has_id(&diags, "ENDCT3"), "got: {diags:?}");
    }

    #[test]
    fn endct4_fires_on_classdef_without_methods() {
        let diags = lint_file(&*engine(), "classdef Foo\n  function f()\n  end\nend\n");
        assert!(has_id(&diags, "ENDCT4"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
    }

    #[test]
    fn endct4_fires_on_missing_methods_block() {
        let diags = lint_file(
            &*engine(),
            "function f()\n    if x\n        y = 1;\nclassdef Foo\n    function f()\n    end\nend\n",
        );
        assert!(has_id(&diags, "ENDCT4"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT2"), "got: {diags:?}");
    }

    #[test]
    fn endct_family_ok_on_valid_code() {
        let diags = lint_file(
            &*engine(),
            "function f()\n    if x\n        y = 1;\n    end\nend\nclassdef Foo\n    methods\n        function f()\n            x = 1;\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT2"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT3"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT4"), "got: {diags:?}");
    }

    #[test]
    fn endct_variant_disabled_falls_back_to_generic() {
        let engine = engine_with_disabled(&["ENDCT2"]);
        let diags = lint_file(&*engine, "if x > 0\n    y = 1;\n");
        assert!(has_id(&diags, "ENDCT"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT2"), "got: {diags:?}");
    }

    #[test]
    fn endct3_disabled_falls_back_to_generic() {
        let engine = engine_with_disabled(&["ENDCT3"]);
        let diags = lint_file(&*engine, "else\nif x\n");
        assert!(has_id(&diags, "ENDCT"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT3"), "got: {diags:?}");
    }

    #[test]
    fn endct_disabled_suppresses_variants() {
        let engine = engine_with_disabled(&["ENDCT"]);
        let diags = lint_file(
            &*engine,
            "if x > 0\n    y = 1;\nelse\nif x\nclassdef Foo\n  function f()\n  end\nend\n",
        );
        assert!(!has_id(&diags, "ENDCT"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT2"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT3"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDCT4"), "got: {diags:?}");
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        SyntaxErrorsEngine::from_config(&Config::default())
    }

    #[test]
    fn endct3_fires_for_function_inside_arguments_block() {
        let src = "function r = f(a)\n    arguments\n        a\n    function q = g(), q = 1; end\n    end\n    r = 1;\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "ENDCT3"));
        // The stray final `end` still gets SYNER (MATLAB reports it too); line 4 must not.
        assert!(
            !diags.iter().any(|d| d.rule_id == "SYNER" && d.line == 4),
            "got: {diags:?}"
        );
    }

    #[test]
    fn endct3_does_not_fire_for_closed_arguments_block() {
        let src = "function r = f(a)\n    arguments\n        a\n    end\n    function q = g(), q = 1; end\n    r = 1;\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "ENDCT3"));
    }

    #[test]
    fn fvsyn_fires_for_lambda_validator_in_arguments_block() {
        let src = "function f(x)\n    arguments\n        x {@(v) v > 0}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "FVSYN"));
        assert!(!has_id(&diags, "NOPAR2"));
    }

    #[test]
    fn fvapn_fires_for_positional_after_name_value_in_call() {
        let diags = lint_file(&*engine(), "f(1, Name=2, 'b', 3);\n");
        assert!(has_id(&diags, "FVAPN"));
        assert!(!has_id(&diags, "FVSYN"));
    }

    #[test]
    fn fvapn_does_not_fire_for_name_value_at_end_of_call() {
        let diags = lint_file(&*engine(), "f(1, Name=2);\n");
        assert!(!has_id(&diags, "FVAPN"));
    }
}

#[cfg(test)]
mod dotted_name_value_tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        SyntaxErrorsEngine::from_config(&Config::default())
    }

    #[test]
    fn dotted_name_value_call_at_statement_level_is_nopar_and_syner() {
        // MATLAB probe on `f(a.b=1);`: NOPAR at `(` (col 2), SYNER at `)` (col 8).
        let diags = lint_file(&*engine(), "f(a.b=1);\n");
        let nopar: Vec<_> = diags.iter().filter(|d| d.rule_id == "NOPAR").collect();
        let syner: Vec<_> = diags.iter().filter(|d| d.rule_id == "SYNER").collect();
        assert_eq!(nopar.len(), 1, "got: {diags:?}");
        assert_eq!((nopar[0].line, nopar[0].column), (1, 2));
        assert_eq!(syner.len(), 1, "got: {diags:?}");
        assert_eq!((syner[0].line, syner[0].column), (1, 8));
    }

    #[test]
    fn dotted_name_value_call_without_semicolon_is_nopar_and_syner() {
        // MATLAB probe on `f(a.b=1)` with a newline: the same two findings.
        let diags = lint_file(&*engine(), "f(a.b=1)\n");
        assert!(has_id(&diags, "NOPAR"), "got: {diags:?}");
        assert!(has_id(&diags, "SYNER"), "got: {diags:?}");
    }

    #[test]
    fn plain_name_value_call_is_not_an_error() {
        // `Name=1` is a valid name-value argument (MATLAB accepts it).
        let diags = lint_file(&*engine(), "f(Name=1);\n");
        assert!(!has_id(&diags, "NOPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "SYNER"), "got: {diags:?}");
    }

    #[test]
    fn value_end_after_assignment_or_paren_is_not_syner() {
        // Probed: MATLAB reports IDXCOLND here, no parse error (a01, a02, a03, a04).
        for source in [
            "y = end + 1;\n",
            "y = (end);\n",
            "y = end*2;\n",
            "y = end;\n",
            "y = end %#ok<IDXCOLND>\n",
            "function r = f(x)\nr = end + 1;\nend\n",
        ] {
            let diags = lint_file(&*engine(), source);
            assert!(!has_id(&diags, "SYNER"), "{source:?} got: {diags:?}");
        }
    }

    #[test]
    fn end_after_operator_or_in_comparison_stays_syner() {
        // Probed: `1 + end`, `-end` and `1:end` are parse errors in MATLAB (a05, a06, a13).
        for source in [
            "y = 1 + end;\n",
            "y = -end;\n",
            "x = 1:3; y = x(end) + end;\n",
        ] {
            let diags = lint_file(&*engine(), source);
            assert!(has_id(&diags, "SYNER"), "{source:?} got: {diags:?}");
        }
        let diags = lint_file(&*engine(), "x = 1:3; y = x(end) == 1;\n");
        assert!(!has_id(&diags, "SYNER"), "got: {diags:?}");
    }

    #[test]
    fn one_line_try_catch_id_end_reports_endct2() {
        // Probed: `try, x = 1; catch ME end` gives ENDCT2 at the TRY (c01).
        let diags = lint_file(&*engine(), "try, x = 1; catch ME end\n");
        assert!(has_id(&diags, "ENDCT2"), "got: {diags:?}");
        assert!(!has_id(&diags, "SYNER"), "got: {diags:?}");
    }

    #[test]
    fn one_line_try_catch_with_separator_is_not_endct2() {
        // Probed: a comma after the catch identifier makes the END valid (tc05).
        for source in [
            "try, x = 1; catch ME, end\n",
            "try, x = 1; catch ME, disp(1); end\n",
            "try, x = 1; catch end\n",
        ] {
            let diags = lint_file(&*engine(), source);
            assert!(!has_id(&diags, "ENDCT2"), "{source:?} got: {diags:?}");
        }
    }
}

#[cfg(test)]
mod declaration_shape_tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        SyntaxErrorsEngine::from_config(&Config::default())
    }

    fn function_with_arguments(block: &str) -> String {
        format!("function r = f(x, y)\narguments{block}\n    x double\nend\nr = x;\nend\n")
    }

    #[test]
    fn valid_arguments_attribute_values_are_not_syner() {
        // MATLAB probes: each value gives ATLAB (or nothing), never SYNER or NOPAR2.
        for attrs in [
            " (Input = 'abc')",
            " (Input = [1 2])",
            " (Input = {1, 2})",
            " (Input = \"str\")",
            " (Input = -1)",
            " (Input = 1:3)",
            " (Input = ?pkg.Cls)",
            " (Input = @(v) v)",
            " (Input=true,Repeating=1)",
            " (Input = ~true)",
            " (Input = 1, Foo = 2)",
        ] {
            let source = function_with_arguments(attrs);
            let diags = lint_file(&*engine(), &source);
            assert!(!has_id(&diags, "SYNER"), "{attrs}: got: {diags:?}");
            assert!(!has_id(&diags, "NOPAR2"), "{attrs}: got: {diags:?}");
        }
    }

    #[test]
    fn class_name_default_in_arguments_is_not_syner() {
        // MATLAB probe: FVNDE and FVORDN, no SYNER. The declaration after it stays in the block.
        let source = "function r = f(x, y)\narguments\n    y.?matlab.unittest.TestCase = 1\n    x double\nend\nr = x;\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(!has_id(&diags, "SYNER"), "got: {diags:?}");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
    }

    #[test]
    fn multi_line_attribute_list_is_still_syner() {
        // Genuinely invalid (MATLAB reports SYNER at the end of the first line).
        let source = "function r = f(x)\narguments (Input,\n    Repeating)\n    x double\nend\nr = x;\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(has_id(&diags, "SYNER"), "got: {diags:?}");
    }

    #[test]
    fn validator_with_function_handle_in_property_is_mcpld_not_syner() {
        // MATLAB probe: MCPLD at `@` (line 3), no SYNER.
        let source = "classdef C\n    properties\n        p {@(v) v > 0} = 1\n        q double\n    end\nend\n";
        let diags = lint_file(&*engine(), source);
        assert!(has_id(&diags, "MCPLD"), "got: {diags:?}");
        assert!(!has_id(&diags, "SYNER"), "got: {diags:?}");
    }
}
