//! `check_error_nodes` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_error_nodes(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let last = Self::last_descendant(root);
        self.walk_error_nodes(root, source, last, &mut diagnostics);
        self.walk_dotted_name_value_calls(root, source, &mut diagnostics);
        Self::walk_end_in_control_header(root, &mut diagnostics);
        Self::walk_dotted_commands(root, source, &mut diagnostics);
        self.walk_source_forms(root, source, &mut diagnostics);

        // An unclosed bracket that ends the file: ENDPAR, NOPAR or EOLPAR at the opener.
        if let Some(last) = last {
            let start = last.start_byte();
            let end = last.end_byte().max(start + 1).min(source.len());
            let text = &source[start..end];
            let is_bracket_error = last.is_error() && Self::looks_like_missing_bracket(text);
            let missing_kind = Self::missing_bracket_opener(last.kind());
            if is_bracket_error || (last.is_missing() && missing_kind.is_some()) {
                let (opener, at) = if last.is_missing() {
                    let line_start = source[..start].rfind('\n').map_or(0, |i| i + 1);
                    let opener = last_unmatched_opener(&source[line_start..start]);
                    (opener.map(|i| line_start + i), start)
                } else {
                    (last_unmatched_opener(text).map(|i| start + i), end)
                };
                let diag = opener.and_then(|o| self.unclosed_bracket_diag(source, o, at));
                if let Some(d) = diag {
                    diagnostics.push(d);
                } else if self.is_check_enabled("ENDPAR") {
                    let bracket = Self::bracket_kinds(text)
                        .first()
                        .copied()
                        .or(missing_kind)
                        .unwrap_or("bracket");
                    let (line, column) = line_col(source, start);
                    diagnostics.push(Diagnostic {
                        rule_id: "ENDPAR",
                        message: format!(
                            "A '{bracket}' might be missing a closing '{bracket}', causing invalid syntax at end of file."
                        ),
                        severity: Severity::Error,
                        byte_range: start..end,
                        line,
                        column,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }

    /// The diagnostic for a bracket left open, its opener at byte `opener` and the
    /// error (or the file's end) at byte `at`. A `(` or `{` followed by `;` or `=` is
    /// NOPAR at that token; followed by a line end it is EOLPAR; at the end of the file
    /// it is ENDPAR. A `[` is ENDPAR only at the end of the file (`x = [1 2;`), and
    /// otherwise None, so that NOPAR2 applies (probed: b14, b16, b17, t04, b151).
    fn unclosed_bracket_diag(&self, source: &str, opener: usize, at: usize) -> Option<Diagnostic> {
        let bracket = match source.as_bytes()[opener] {
            b'(' => "(",
            b'{' => "{",
            _ => "[",
        };
        let closer = match bracket {
            "(" => ")",
            "{" => "}",
            _ => "]",
        };
        let rest = source.get(at..)?;
        let (rule_id, message) = if bracket == "[" {
            let eof_like = rest
                .bytes()
                .all(|b| matches!(b, b'\n' | b'\r' | b' ' | b'\t' | b';' | b','));
            if !eof_like {
                return None;
            }
            ("ENDPAR", format!("A '{bracket}' might be missing a closing '{closer}', causing invalid syntax at end of file."))
        } else {
            // The first `;` or `=` (not a comparison) on the opener's line is the token MATLAB names.
            let line = &source[opener + 1..];
            let line = &line[..line.find('\n').unwrap_or(line.len())];
            let bytes = line.as_bytes();
            let token = (0..bytes.len()).find_map(|i| match bytes[i] {
                b';' => Some(';'),
                b'=' if bytes.get(i + 1) != Some(&b'=')
                    && !(i > 0 && matches!(bytes[i - 1], b'=' | b'~' | b'<' | b'>')) =>
                {
                    Some('=')
                }
                _ => None,
            });
            match token {
                Some(c) => ("NOPAR", format!("A '{bracket}' might be missing a closing '{closer}', causing invalid syntax at '{c}'.")),
                None if source[opener..].contains('\n') => ("EOLPAR", format!("A '{bracket}' might be missing a closing '{closer}', causing invalid syntax at end of line.")),
                None => ("ENDPAR", format!("A '{bracket}' might be missing a closing '{closer}', causing invalid syntax at end of file.")),
            }
        };
        if !self.is_check_enabled(rule_id) {
            return None;
        }
        let (line, column) = line_col(source, opener);
        Some(Diagnostic {
            rule_id,
            message,
            severity: Severity::Error,
            byte_range: opener..opener + 1,
            line,
            column,
            fix: None,
        })
    }

    /// A dotted name followed by a word (`comm.gpu.AWGNChannel arg1`) cannot be a command in
    /// MATLAB: it is a syntax error at the word, on that line only. The grammar reads it as a
    /// command whose name is the first identifier and whose first argument is the rest of the
    /// dotted name, glued to it.
    fn walk_dotted_commands(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if let Some(cmd) = crate::analysis::command::Command::from_node(node, source) {
            // The syntax error is at the child after the glued dotted part.
            if let (true, Some(word)) = (cmd.is_dotted_misread(), node.child(2)) {
                diagnostics.push(Self::syner_at(source, word.start_byte()));
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_dotted_commands(child, source, diagnostics);
        }
    }

    /// The grammar parses `if end > 1` (an `end` outside any index in a control header),
    /// MATLAB does not: it is a parse error.
    fn walk_end_in_control_header(node: Node, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "end_keyword" && crate::language_spec::in_control_header(node) {
            let pos = node.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "SYNER",
                message: "Parse error at end: usage might be invalid MATLAB syntax.".to_string(),
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_end_in_control_header(child, diagnostics);
        }
    }

    pub(crate) fn walk_error_nodes(
        &self,
        node: Node,
        source: &str,
        last: Option<Node>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // An unterminated block comment (`%{` at the end of the file) is INBLK's.
        if node.is_error() && !source[node.start_byte()..].starts_with("%{") {
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
                let opener = last_unmatched_opener(text).map(|i| start + i);
                let name_value_call = Self::is_in_function_call_args(&node) && text.contains('=');
                if let Some(d) = opener.and_then(|o| self.unclosed_bracket_diag(source, o, end)) {
                    diagnostics.push(d);
                } else if self.is_check_enabled("NOPAR2") && !is_last && !name_value_call {
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
            } else if self.is_check_enabled("ENDCT")
                && !Self::is_keyword_misuse(&text.to_lowercase())
                && Self::looks_like_missing_end(text, &node)
            {
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
                // MATLAB reports the positional argument that follows, not the comma.
                let at = Self::next_argument_start(source, start);
                let (line, column) = line_col(source, at);
                diagnostics.push(Diagnostic {
                    rule_id: "FVAPN",
                    message: "Move name-value arguments that use the name=value syntax to the end of the argument list.".to_string(),
                    severity: Severity::Error,
                    byte_range: at..at + 1,
                    line,
                    column,
                    fix: None,
                });
            }
            // A name=value in a call that is not a statement: MATLAB reports SYNER at the
            // `=` (`x = f(a.b=1)`). Two values in a call (`f(1 2)`) are SYNER too, not FVSYN.
            else if self.is_check_enabled("SYNER")
                && Self::is_in_function_call_args(&node)
                && !Self::call_is_statement(&node)
                && !Self::call_is_brace(&node)
                && text.trim_start().starts_with('=')
                && !source[..start].trim_end().ends_with(['\'', '"'])
            {
                let at = start + text.len() - text.trim_start().len();
                diagnostics.push(Self::syner_at(source, at));
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
                let at = Self::syner_error_offset(&node, text, source);
                // The range stays the error's, so that a lexical check on the same token
                // still takes the SYNER's place (see mod.rs).
                let mut d = Self::syner_at(source, at);
                d.byte_range = start..end;
                diagnostics.push(d);
            }
        } else if node.is_missing() {
            if let Some(bracket) = Self::missing_bracket_opener(node.kind()) {
                let is_last = last.is_some_and(|l| l.id() == node.id());
                if !is_last {
                    let start = node.start_byte();
                    let end = node.end_byte().max(start + 1);
                    let pos = node.start_position();
                    // The opener on this line decides NOPAR, EOLPAR or ENDPAR (`x = f(1;`
                    // is NOPAR even though only `;` follows). Quoted names are FVAMI/FVACS.
                    let line_start = source[..start].rfind('\n').map_or(0, |i| i + 1);
                    let opener = last_unmatched_opener(&source[line_start..start])
                        .map(|i| line_start + i)
                        .filter(|&o| !source[o..start].contains(['\'', '"']));
                    if let Some(d) =
                        opener.and_then(|o| self.unclosed_bracket_diag(source, o, start))
                    {
                        diagnostics.push(d);
                    } else if Self::is_end_of_file(node, source) {
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
                        .is_some_and(|a| !matches!(a.kind(), "identifier" | "string"))
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
        // A `;` row separator in an output list is LHROW, not a parse error.
        if node
            .parent()
            .is_some_and(|p| p.kind() == "multioutput_variable")
            && source[node.start_byte()..node.end_byte().min(source.len())].trim() == ";"
        {
            return true;
        }
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
            || !Self::literal_end_offsets(text, node.start_byte(), source).is_empty()
            // A run of commas is NOCOMMA (`x = 1,, y = 2;`), not a parse error.
            || text.trim_start().starts_with(',')
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

impl SyntaxErrorsEngine {
    /// Errors MATLAB reports from the text, not from the parse tree: `i -= 1` (a command
    /// in the grammar), an `end` that follows a statement with no separator (with EOFER
    /// for the file), a `~` in a function's output list, and an `arguments (` line that
    /// is still open at its end of line.
    pub(crate) fn walk_source_forms(
        &self,
        root: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.walk_op_assign_commands(root, source, diagnostics);
        self.walk_function_output_tilde(root, source, diagnostics);
        self.check_unseparated_ends(root, source, diagnostics);
        self.check_arguments_unclosed(source, diagnostics);
    }

    /// `i += 1` and `i -= 1` parse as a command whose first argument is the operator.
    /// MATLAB reports SYNER at the `=` for the imaginary unit `i` or `j`, in scripts and
    /// functions alike (probed). Other names give REDEF when already assigned, else nothing.
    fn walk_op_assign_commands(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.kind() == "command" && self.is_check_enabled("SYNER") {
            let mut cursor = node.walk();
            let first = node
                .children(&mut cursor)
                .find(|c| c.kind() == "command_argument");
            let word = source[node.start_byte()..node.end_byte()]
                .split_whitespace()
                .next()
                .unwrap_or("");
            if let Some(arg) = first {
                let text = &source[arg.start_byte()..arg.end_byte()];
                if matches!(text, "+=" | "-=" | "*=" | "/=" | "^=") && matches!(word, "i" | "j") {
                    diagnostics.push(Self::syner_at(source, arg.end_byte() - 1));
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_op_assign_commands(child, source, diagnostics);
        }
    }

    /// A `~` in a function's output list (`function [a, ~] = f(x)`) is SYNER at the `~`.
    /// A `~` followed by a comma also gives SYNER at the `]`, and the function's own
    /// `end` gets one too (probed: b07, t17, t18).
    fn walk_function_output_tilde(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "function_definition" && self.is_check_enabled("SYNER") {
            let mut ignored = Vec::new();
            let mut cursor = node.walk();
            for output in node
                .children(&mut cursor)
                .filter(|c| c.kind() == "function_output")
            {
                Self::collect_kind(output, "ignored_argument", &mut ignored);
            }
            if !ignored.is_empty() {
                for tilde in &ignored {
                    diagnostics.push(Self::syner_at(source, tilde.start_byte()));
                    let after = &source[tilde.end_byte()..];
                    if after.trim_start_matches([' ', '\t']).starts_with(',') {
                        if let Some(rel) = after.find(']') {
                            diagnostics.push(Self::syner_at(source, tilde.end_byte() + rel));
                        }
                    }
                }
                let head = source[..node.end_byte()].trim_end();
                if head.ends_with("end") {
                    diagnostics.push(Self::syner_at(source, head.len() - 3));
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_function_output_tilde(child, source, diagnostics);
        }
    }

    fn collect_kind<'a>(node: Node<'a>, kind: &str, out: &mut Vec<Node<'a>>) {
        if node.kind() == kind {
            out.push(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_kind(child, kind, out);
        }
    }

    /// `if x, disp(1) end`: MATLAB reports SYNER at the `end`, which follows a statement
    /// with no separator, and EOFER for the file. Probed in scripts only: a block inside a
    /// function gives the SYNER without EOFER.
    fn check_unseparated_ends(&self, root: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let mut skip = Vec::new();
        Self::collect_skip_ranges(root, &mut skip);
        let ends = unseparated_block_ends(source, &skip);
        if ends.is_empty() {
            return;
        }
        if self.is_check_enabled("SYNER") {
            for &at in &ends {
                let (line, col) = line_col(source, at);
                // A SYNER earlier on the same line belongs to the same broken statement.
                diagnostics.retain(|d| !(d.rule_id == "SYNER" && d.line == line && d.column < col));
                diagnostics.push(Self::syner_at(source, at));
            }
        }
        if self.is_check_enabled("EOFER") && !Self::has_descendant_kind(root, "function_definition")
        {
            diagnostics.push(Diagnostic {
                rule_id: "EOFER",
                message: "Code analysis did not complete. File contains too many syntax errors."
                    .to_string(),
                severity: Severity::Error,
                byte_range: 0..1,
                line: 1,
                column: 1,
                fix: None,
            });
        }
    }

    /// `arguments (Input,` with the `(` still open at the end of its line: MATLAB reports
    /// SYNER at the line end after a comma (probed b11), and EOLPAR at the `(` otherwise
    /// (probed t20). The error nodes tree-sitter puts on the lines that follow are its
    /// own cascade and are dropped.
    fn check_arguments_unclosed(&self, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let mut line_start = 0;
        for (idx, line) in source.split('\n').enumerate() {
            let at_line = line_start;
            line_start += line.len() + 1;
            let Some(rest) = line.trim_start().strip_prefix("arguments") else {
                continue;
            };
            if !rest.starts_with([' ', '\t', '(']) {
                continue;
            }
            let Some(open) = line.find('(') else {
                continue;
            };
            if line[open + 1..].contains(')') {
                continue;
            }
            let line_no = idx + 1;
            diagnostics.retain(|d| {
                !(matches!(d.rule_id, "SYNER" | "NOPAR2")
                    && (line_no..=line_no + 2).contains(&d.line))
            });
            let code = line.trim_end();
            if code.ends_with(',') {
                if self.is_check_enabled("SYNER") {
                    let at = at_line + code.len() - 1;
                    let (line, column) = line_col(source, at);
                    diagnostics.push(Diagnostic {
                        rule_id: "SYNER",
                        message: "Parse error at <EOL>: usage might be invalid MATLAB syntax."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: at..at + 1,
                        line,
                        column,
                        fix: None,
                    });
                }
            } else if self.is_check_enabled("EOLPAR") {
                let at = at_line + open;
                let (line, column) = line_col(source, at);
                diagnostics.push(Diagnostic {
                    rule_id: "EOLPAR",
                    message: "A '(' might be missing a closing ')', causing invalid syntax at end of line.".to_string(),
                    severity: Severity::Error,
                    byte_range: at..at + 1,
                    line,
                    column,
                    fix: None,
                });
            }
        }
    }

    /// A SYNER at byte `at`, naming the token there the way MATLAB does: `'='` and `')'`
    /// quoted, identifiers and numbers bare, and keywords in capitals.
    pub(crate) fn syner_at(source: &str, at: usize) -> Diagnostic {
        let (token, len) = syner_token(&source[at..]);
        let (line, column) = line_col(source, at);
        Diagnostic {
            rule_id: "SYNER",
            message: format!("Parse error at {token}: usage might be invalid MATLAB syntax."),
            severity: Severity::Error,
            byte_range: at..at + len.max(1),
            line,
            column,
            fix: None,
        }
    }

    /// The byte MATLAB names for a parse error that tree-sitter reports from `node`: the
    /// first unmatched closing bracket (`x = [1 2 3]]`), a keyword after `=` (`x = for;`),
    /// the second of two adjacent values (`x = 1 2`), the second comma of `,,` (`f(1,,2)`),
    /// and otherwise the start of the error.
    fn syner_error_offset(node: &Node, text: &str, source: &str) -> usize {
        let start = node.start_byte();
        if let Some(i) = unmatched_closer(text).filter(|_| !Self::is_in_function_call_args(node)) {
            return start + i;
        }
        if let Some(i) = keyword_after_equals(text) {
            return start + i;
        }
        let word_len = source[start..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(source.len() - start);
        if word_len > 0 {
            let rest = &source[start + word_len..];
            let gap = rest.len() - rest.trim_start_matches([' ', '\t']).len();
            if gap > 0
                && rest[gap..].starts_with(|c: char| {
                    c.is_ascii_alphanumeric() || matches!(c, '_' | '~' | '@' | '\'' | '"')
                })
            {
                return start + word_len + gap;
            }
        }
        if source[start..].starts_with(",,") {
            return start + 1;
        }
        start
    }

    /// True when the call that holds `node` indexes with braces (`c{a=1}`), which is FVACI.
    fn call_is_brace(node: &Node) -> bool {
        let mut current = node.parent();
        while let Some(ancestor) = current {
            if ancestor.kind() == "function_call" {
                let mut cursor = ancestor.walk();
                return ancestor.children(&mut cursor).any(|c| c.kind() == "{");
            }
            current = ancestor.parent();
        }
        false
    }

    /// True when the call that holds `node` is a statement (`f(a.b=1);`), not a value.
    fn call_is_statement(node: &Node) -> bool {
        let mut current = node.parent();
        while let Some(ancestor) = current {
            if ancestor.kind() == "function_call" {
                return ancestor
                    .parent()
                    .is_some_and(|p| matches!(p.kind(), "source_file" | "block"));
            }
            current = ancestor.parent();
        }
        false
    }

    /// The start of the positional argument after a name=value argument (`'b'` in
    /// `f(1, Name=2, 'b')`): MATLAB reports FVAPN there.
    fn next_argument_start(source: &str, from: usize) -> usize {
        match source[from..].find(',') {
            Some(c) => {
                let after = from + c + 1;
                after + (source[after..].len() - source[after..].trim_start().len())
            }
            None => from,
        }
    }
}

/// The token MATLAB names at the start of `rest`, quoted when it is punctuation, and
/// its length in bytes.
fn syner_token(rest: &str) -> (String, usize) {
    let word_len = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    if word_len > 0 {
        let word = &rest[..word_len];
        let shown = if RESERVED_KEYWORDS.contains(&word.to_ascii_lowercase().as_str()) {
            word.to_ascii_uppercase()
        } else {
            word.to_string()
        };
        return (shown, word_len);
    }
    let len = match rest.chars().next() {
        None => return (String::new(), 0),
        Some(c) if matches!(c, '=' | '~' | '<' | '>' | '!') && rest[1..].starts_with('=') => 2,
        Some(c) => c.len_utf8(),
    };
    (format!("'{}'", &rest[..len]), len)
}

/// The offset of the first closing bracket in `text` that has no opener before it.
fn unmatched_closer(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in text.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// The offset of a reserved keyword (other than `end`) that follows `=` in `text`.
fn keyword_after_equals(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut prev_sig = None;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_alphabetic() {
            let s = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = text[s..i].to_ascii_lowercase();
            if prev_sig == Some(b'=') && word != "end" && RESERVED_KEYWORDS.contains(&word.as_str())
            {
                return Some(s);
            }
            prev_sig = Some(bytes[i - 1]);
            continue;
        }
        if !c.is_ascii_whitespace() {
            prev_sig = Some(c);
        }
        i += 1;
    }
    None
}

/// The byte offsets of `end` words that close a block after a statement on the same line
/// with no separator (`if x, disp(1) end`). Comments, strings, indexes (`x(end)`), fields
/// (`s.end`), a `catch` line and a previous keyword (`else end`) are not counted.
fn unseparated_block_ends(source: &str, skip: &[Range<usize>]) -> Vec<usize> {
    let bytes = source.as_bytes();
    source
        .match_indices("end")
        .map(|(at, _)| at)
        .filter(|&at| {
            let before_ok = at == 0
                || !(bytes[at - 1].is_ascii_alphanumeric() || matches!(bytes[at - 1], b'_' | b'.'));
            let after_ok = bytes
                .get(at + 3)
                .is_none_or(|b| !(b.is_ascii_alphanumeric() || *b == b'_'));
            if !(before_ok && after_ok) || SyntaxErrorsEngine::in_skip_range(at, skip) {
                return false;
            }
            let line_start = source[..at].rfind('\n').map_or(0, |i| i + 1);
            let prefix = source[line_start..at].trim_end_matches([' ', '\t']);
            let Some(prev) = prefix.chars().next_back() else {
                return false;
            };
            let last_word = prefix
                .rsplit(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .next()
                .unwrap_or("");
            let statement_before = prev.is_ascii_alphanumeric() || matches!(prev, ')' | ']' | '}');
            statement_before
                && !matches!(
                    last_word,
                    "else" | "try" | "otherwise" | "end" | "break" | "continue" | "return"
                )
                && !prefix.contains("catch")
                && !SyntaxErrorsEngine::inside_brackets(prefix)
                && prefix.matches('(').count() <= prefix.matches(')').count()
        })
        .collect()
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
        // Mid-file only: a `[` left open at the end of the file is ENDPAR (probed b17).
        let diags = lint_file(&*engine(), "y = [1 2;\nz = 3;\n");
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
        // Probed: `x = f(1;` and `x = f(1 + 2;` are NOPAR at the `;`, not ENDPAR.
        let diags = lint_file(&*engine(), "x = f(1 + 2;\n");
        assert!(has_id(&diags, "NOPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "ENDPAR"), "got: {diags:?}");
        assert!(!has_id(&diags, "NOPAR2"), "got: {diags:?}");
        assert!(!has_id(&diags, "EOLPAR"), "got: {diags:?}");
    }

    #[test]
    fn endpar_fires_on_missing_bracket_at_eof_no_newline() {
        // Probed (b-series): `x = f(1 + 2;` without a newline is NOPAR at the `;`.
        let diags = lint_file(&*engine(), "x = f(1 + 2;");
        assert!(has_id(&diags, "NOPAR"), "got: {diags:?}");
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
