//! # SYNTAX_ERRORS_ENGINE: Parser-Level and Semantic Syntax Validation
//!
//! ```malt
//! id = "SYNTAX_ERRORS_ENGINE"
//! title = "Syntax Errors"
//! category = "syntax-errors"
//! severity = "error"
//! fix = true
//! icon = "lucide/x-circle"
//! slug = "syntax-errors"
//! ```
//!
//! ## Rule
//!
//! Detects syntax errors in MATLAB source files. The engine combines
//! tree-sitter `ERROR`/`MISSING` node analysis with semantic checks the parser
//! cannot catch on its own: invalid operators, malformed file and function
//! names, structural violations, unterminated strings and comments, and
//! invalid numeric literals.
//!
//! A single file-level engine (`SyntaxErrorsEngine`) dispatches all 48 checks
//! in one pass over the parse tree, plus targeted source scans that skip over
//! comments and strings. Each diagnostic carries the specific check ID
//! (e.g. `SYNER`, `BADNE`, `SEPEXR`).
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | SYNER | error | no | Parse error at VAR_RESERVED_WORD: usage might be invalid MATLAB syntax. |
//! | BDFIL | error | no | Invalid MATLAB file name. MATLAB file names must start with a letter, contain only letters, numbers, or underscores, and have no more than VAR_NUMBER characters. |
//! | BADNE | error | yes | 'Not Equals' is spelled ~= in MATLAB, not !=. |
//! | BADOT | error | no | Use of two dots (..) is an invalid MATLAB construction. |
//! | TWOCM | error | no | A comma cannot immediately follow another comma. |
//! | CLIS | error | no | Defining a class in script is not allowed. |
//! | CLTWO | error | no | Only one class definition is allowed per file, and it must come at the head of the file. |
//! | SOFOC | error | no | Statement outside a class definition is not allowed. |
//! | SEMFU | error | no | Script file must contain executable code. Remove empty statements to make this file a function file. |
//! | FNDOT | error | no | Function name can only contain dots if it is a class method. |
//! | FNSWA | error | no | Function name must start with alphabetic character. |
//! | NOPAR2 | error | no | A VAR_NAME might be missing a closing VAR_NAME, causing invalid syntax at VAR_NAME on line VAR_NUMBER. |
//! | NOPAR | error | no | A VAR_NAME might be missing a closing VAR_NAME, causing invalid syntax at VAR_NAME. |
//! | EOLPAR | error | no | A VAR_NAME might be missing a closing VAR_NAME, causing invalid syntax at end of line. |
//! | ENDPAR | error | no | A VAR_NAME might be missing a closing VAR_NAME, causing invalid syntax at end of file. |
//! | ENDCT | error | no | An END might be missing, possibly matching VAR_RESERVED_WORD. |
//! | ENDCT2 | error | no | An END might be missing (after line VAR_RESERVED_WORD), possibly matching VAR_NUMBER. |
//! | ENDCT3 | error | no | An END might be missing (before VAR_RESERVED_WORD on line VAR_NUMBER), possibly matching VAR_RESERVED_WORD. |
//! | ENDCT4 | error | no | A METHODS block or END might be missing before the function definition. This might be causing additional error messages. |
//! | NOLHS | error | no | Left side of an assignment is empty. |
//! | BADCH | error | no | Invalid text character(s). |
//! | BADSP | error | no | Invalid text character(s). The text contains an unsupported non-ASCII whitespace character. |
//! | BADCT | error | no | Unicode explicit directional formatting characters are not supported. |
//! | REDEF | error | no | The current use of VAR_NAME is inconsistent with its previous use or definition (line VAR_NUMBER). |
//! | SEPEXR | error | yes | Use a newline, semicolon, or comma before this statement. |
//! | SEPEXC | error | no | Use comma to separate between input arguments. |
//! | SBTMP | error | no | Invalid array indexing or function call. Chaining outputs after parenthesis is not supported. |
//! | FVSYN | error | no | Invalid function argument syntax at VAR_RESERVED_WORD. |
//! | FVAPN | error | no | Move name-value arguments that use the name=value syntax to the end of the argument list. |
//! | FVACI | error | no | Use of name-value arguments in cell indexing is not supported. |
//! | FVACS | error | no | Using a character vector or string as a name in name=value syntax is not supported. Remove the quotes around the name. |
//! | FVAMI | error | no | Name in name-value argument syntax must be a valid MATLAB identifier. |
//! | UNSET | error | no | Invalid use of VAR_OPERATOR on the left side of an assignment. |
//! | LHROW | error | no | The left side of an assignment cannot have multiple rows (';'). |
//! | RESWD | error | no | Invalid use of a reserved word. |
//! | SYNEND | error | no | Invalid use for END operator. |
//! | MCPLD | error | no | Invalid property syntax at VAR_RESERVED_WORD. |
//! | BADNOT | error | no | Using ~ to ignore a value is not permitted in this context. |
//! | BADNOTLHS | error | no | Invalid use of logical not operator (~) on left side of an assignment. To use ~ to ignore function outputs, separate output variables with commas. |
//! | STRIN | error | no | A quoted character vector is unterminated. |
//! | DOUQT | error | no | A double quoted string is unterminated. |
//! | INBLK | error | no | A block comment is unterminated at the end of the file. |
//! | BADFP | error | no | Invalid floating-point constant. |
//! | BADHBH | error | no | Invalid digit in hexadecimal literal. Supported hexadecimal digits are 0-9 and A-F. Supported type suffixes are u8, u16, u32, u64, and s8, s16, s32, s64. |
//! | BADHBB | error | no | Invalid digit in binary literal. Supported binary digits are 0 and 1. Supported type suffixes are u8, u16, u32, u64, and s8, s16, s32, s64. |
//! | BADHBHT | error | no | Hexadecimal literal has too many digits for specified type suffix. |
//! | BADHBBT | error | no | Binary literal has too many digits for specified type suffix. |
//! | HEXTOOLONG | error | no | Hexadecimal literal has too many digits. |
//! | BINARYTOOLONG | error | no | Binary literal has too many digits. |
//!
//! ## Fix
//!
//! Two checks carry automatic fixes:
//!
//! - `BADNE` rewrites `!=` to `~=`, the MATLAB not-equal operator.
//! - `SEPEXR` inserts a `;` between two statements that share a line with no
//!   separator.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! x = 1 != 2;      % BADNE: `!=` is not MATLAB
//! a = 1 b = 2;     % SEPEXR: statements on one line without separator
//! if x > 0
//!     y = 'unterminated;   % STRIN
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! x = 1 ~= 2;
//! a = 1; b = 2;
//! if x > 0
//!     y = 'terminated';
//! end
//! ```
//!
//! ### Fixed
//!
//! ```diff
//! - x = 1 != 2;
//! + x = 1 ~= 2;
//! - a = 1 b = 2;
//! + a = 1; b = 2;
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.SYNTAX_ERRORS_ENGINE]
//! disabled_checks = []
//! ```

mod check_assignment_lhs;
mod check_call_syntax;
mod check_case_separators;
mod check_directional_formatting;
mod check_error_nodes;
mod check_file;
mod check_file_name;
mod check_file_structure;
mod check_function_names;
mod check_name_value;
mod check_number_literals;
mod check_reserved_and_not;
mod check_sepexc;
mod check_statements;
mod check_unterminated;
mod check_validation_order;

use std::collections::HashSet;
use std::ops::Range;

use malt_core::{Category, Config, Diagnostic, FileContext, Rule, Severity};
use serde::Deserialize;
use tree_sitter::Node;

/// expressions in MATLAB.
const RESERVED_KEYWORDS: &[&str] = &[
    "else",
    "end",
    "if",
    "for",
    "while",
    "switch",
    "case",
    "otherwise",
    "classdef",
    "function",
    "try",
    "catch",
    "break",
    "continue",
    "return",
    "persistent",
    "global",
    "parfor",
    "spmd",
    "properties",
    "methods",
    "events",
    "enumeration",
];

/// Keywords that open a block which must be closed with a matching `end`.
const BLOCK_OPENERS: &[&str] = &[
    "if",
    "for",
    "while",
    "switch",
    "try",
    "function",
    "classdef",
    "properties",
    "methods",
    "events",
    "enumeration",
];

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Deserialized from the `[lint.rules.SYNTAX_ERRORS_ENGINE]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SyntaxErrorsConfig {
    /// List of individual check IDs to disable (e.g., `["BADOT", "TWOCM"]`).
    #[serde(default)]
    pub disabled_checks: Vec<String>,
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// semantic syntax validation in a single pass.
pub struct SyntaxErrorsEngine {
    /// Set of disabled check IDs for fast lookup.
    disabled: HashSet<String>,
}

impl SyntaxErrorsEngine {
    /// Factory constructor. Reads rule-specific params from config.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: SyntaxErrorsConfig = config.rule_params("SYNTAX_ERRORS_ENGINE");
        let disabled: HashSet<String> = rule_config.disabled_checks.into_iter().collect();
        Box::new(Self { disabled })
    }

    /// Returns true if the given check ID is enabled.
    fn is_check_enabled(&self, check_id: &str) -> bool {
        !self.disabled.contains(check_id)
    }

    // -----------------------------------------------------------------------
    // Source text scanning (BADNE, BADOT, TWOCM, BADCH, BADSP)
    // -----------------------------------------------------------------------

    /// source-level scanning.
    pub(crate) fn collect_skip_ranges(node: Node, ranges: &mut Vec<Range<usize>>) {
        if node.kind() == "comment" || node.kind() == "string" {
            ranges.push(node.byte_range());
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_skip_ranges(child, ranges);
        }
    }

    /// Check if a byte position is inside a skip range.
    pub(crate) fn in_skip_range(pos: usize, ranges: &[Range<usize>]) -> bool {
        ranges.iter().any(|r| r.contains(&pos))
    }

    /// True if `prefix` (the source of a line up to a position) leaves a `[` or `{` open.
    pub(crate) fn inside_brackets(prefix: &str) -> bool {
        let opens = prefix.chars().filter(|&c| c == '[' || c == '{').count();
        let closes = prefix.chars().filter(|&c| c == ']' || c == '}').count();
        opens > closes
    }

    // -----------------------------------------------------------------------
    // Tree ERROR/MISSING node analysis (SYNER, NOPAR2, EOLPAR, ENDPAR, ENDCT)
    // -----------------------------------------------------------------------

    /// Heuristic: does this error text look like an unclosed bracket?
    pub(crate) fn looks_like_missing_bracket(text: &str) -> bool {
        let opens: usize = text
            .chars()
            .filter(|&c| c == '(' || c == '[' || c == '{')
            .count();
        let closes: usize = text
            .chars()
            .filter(|&c| c == ')' || c == ']' || c == '}')
            .count();
        opens > closes
    }

    /// the given error text.
    pub(crate) fn bracket_kinds(text: &str) -> Vec<&str> {
        let mut kinds = Vec::new();
        for (open, close, kind) in [('(', ')', "("), ('[', ']', "["), ('{', '}', "{")] {
            let opens = text.chars().filter(|&c| c == open).count();
            let closes = text.chars().filter(|&c| c == close).count();
            if opens > closes {
                kinds.push(kind);
            }
        }
        kinds
    }

    /// Returns the matching opener for a missing closing bracket kind.
    pub(crate) fn missing_bracket_opener(kind: &str) -> Option<&'static str> {
        match kind {
            ")" => Some("("),
            "]" => Some("["),
            "}" => Some("{"),
            _ => None,
        }
    }

    /// before end of file.
    pub(crate) fn is_end_of_file(node: Node, source: &str) -> bool {
        source[node.end_byte()..]
            .bytes()
            .all(|b| matches!(b, b'\n' | b'\r' | b' ' | b'\t' | b';' | b','))
    }

    /// whitespace follows it up to the next newline). A `;` or `,` does not end
    /// the line: MATLAB reports `x = f(1;` as NOPAR at the `;`, not EOLPAR.
    pub(crate) fn is_line_ending(node: Node, source: &str) -> bool {
        let rest = &source[node.end_byte()..];
        let line_end = rest.find('\n').unwrap_or(rest.len());
        rest[..line_end]
            .bytes()
            .all(|b| matches!(b, b'\r' | b' ' | b'\t'))
    }

    /// Heuristic: does this error look like a missing `end`?
    pub(crate) fn looks_like_missing_end(text: &str, node: &Node) -> bool {
        // Check text for block openers without matching end.
        let lower = text.to_lowercase();
        let has_opener = BLOCK_OPENERS
            .iter()
            .any(|kw| Self::contains_whole_word(&lower, kw));
        let has_end = lower.contains("end");

        if has_opener && !has_end {
            return true;
        }

        // Check if previous sibling is a block statement.
        if let Some(prev) = node.prev_sibling() {
            let kind = prev.kind();
            if BLOCK_OPENERS.contains(&kind) {
                return true;
            }
        }

        false
    }

    /// the call.
    pub(crate) fn is_in_function_call_args(node: &Node) -> bool {
        let mut current = node.parent();
        while let Some(ancestor) = current {
            match ancestor.kind() {
                "arguments" => {
                    return ancestor
                        .parent()
                        .is_some_and(|p| p.kind() == "function_call");
                }
                "function_call" => {
                    // The ERROR is a direct child of the call. It is within the
                    // argument list when the call also has an `arguments` child.
                    let mut cursor = ancestor.walk();
                    return ancestor
                        .children(&mut cursor)
                        .any(|c| c.kind() == "arguments");
                }
                _ => {}
            }
            current = ancestor.parent();
        }
        false
    }

    /// A block keyword used as a value or a name (`x = for;`, `if = 1`): a plain syntax
    /// error in MATLAB, not a missing END. `lower` is the lower-cased ERROR text.
    pub(crate) fn is_keyword_misuse(lower: &str) -> bool {
        let t = lower.trim();
        let value = t.strip_prefix('=').map(str::trim).unwrap_or(t);
        let value = value.trim_end_matches([';', ',']).trim();
        let assigned_keyword = BLOCK_OPENERS.contains(&value) && t.starts_with('=');
        let keyword_name = BLOCK_OPENERS.iter().any(|kw| {
            t.strip_prefix(kw).is_some_and(|rest| {
                let rest = rest.trim_start();
                rest.starts_with('=') && !rest.starts_with("==")
            })
        });
        assigned_keyword || keyword_name
    }

    /// that starts with a block-opening keyword).
    pub(crate) fn classify_missing_end(
        node: &Node,
        source: &str,
    ) -> Option<(&'static str, String)> {
        let start = node.start_byte();
        let node_end = node.end_byte().min(source.len());
        let text = &source[start..node_end];
        let lower = text.to_lowercase();
        let line = node.start_position().row + 1;

        // ENDCT4: ERROR text contains "classdef", a function_definition
        // follows it as a sibling, and no "methods" appears between them.
        if lower.contains("classdef") {
            let mut sibling = node.next_sibling();
            while let Some(sib) = sibling {
                if sib.kind() == "function_definition" && sib.start_byte() > start {
                    let between = &source[node_end..sib.start_byte()];
                    if !between.to_lowercase().contains("methods") {
                        return Some((
                            "ENDCT4",
                            "A METHODS block or END might be missing before the function definition. This might be causing additional error messages.".to_string(),
                        ));
                    }
                    break;
                }
                sibling = sib.next_sibling();
            }
        }

        if Self::is_keyword_misuse(&lower) {
            return None;
        }

        // ENDCT2: a block opener appears in the ERROR text or its previous
        // sibling, and the ERROR does not contain `end`.
        let has_end = lower.contains("end");
        if !has_end {
            // Whole words only: `elseif` is not an `if` opener.
            let opener_in_text = BLOCK_OPENERS
                .iter()
                .any(|kw| Self::contains_whole_word(&lower, kw));
            let opener_in_prev = node
                .prev_sibling()
                .map(|prev| {
                    let prev_text = source[prev.start_byte()..prev.end_byte()].to_lowercase();
                    BLOCK_OPENERS
                        .iter()
                        .any(|kw| Self::contains_whole_word(&prev_text, kw))
                })
                .unwrap_or(false);
            if opener_in_text || opener_in_prev {
                let raw_end = node.end_position().row;
                let end_line = if source[..node_end].ends_with('\n') {
                    raw_end
                } else {
                    raw_end + 1
                };
                let opener_line = if opener_in_text {
                    line
                } else {
                    node.prev_sibling()
                        .map(|p| p.start_position().row + 1)
                        .unwrap_or(line)
                };
                return Some((
                    "ENDCT2",
                    format!(
                        "An END might be missing (after line {end_line}), possibly matching {opener_line}."
                    ),
                ));
            }
        }

        // ENDCT3: the ERROR is followed by a sibling that starts with a
        // block-opening keyword. An ERROR that starts with `function` is a whole
        // function header and body that tree-sitter could not parse, not a
        // function after an open block: MATLAB reports nothing for it.
        let starts_with_function = text
            .split_whitespace()
            .next()
            .is_some_and(|w| w.eq_ignore_ascii_case("function"));
        let mut sibling = node.next_sibling();
        while let Some(sib) = sibling.filter(|_| !starts_with_function) {
            let sib_text = &source[sib.start_byte()..sib.end_byte()];
            match sib_text.split_whitespace().next() {
                Some(first) if BLOCK_OPENERS.contains(&first.to_lowercase().as_str()) => {
                    let kw = text.split_whitespace().next().unwrap_or("").to_uppercase();
                    let matched = first.to_uppercase();
                    return Some((
                        "ENDCT3",
                        format!(
                            "An END might be missing (before {kw} on line {line}), possibly matching {matched}."
                        ),
                    ));
                }
                Some(_) => break,
                None => {}
            }
            sibling = sib.next_sibling();
        }

        None
    }

    // -----------------------------------------------------------------------
    // File structure validation (BDFIL, CLIS, CLTWO, SOFOC, SEMFU)
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Function name validation (FNDOT, FNSWA)
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Statement validation (NOLHS, SEPEXR, REDEF)
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Assignment LHS validation (UNSET, LHROW)
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Reserved words and invalid ~ usage (RESWD, SYNEND, MCPLD, BADNOT,
    // BADNOTLHS)
    // -----------------------------------------------------------------------

    /// Whole-word substring check: `needle` must not be part of a larger word.
    pub(crate) fn contains_whole_word(haystack: &str, needle: &str) -> bool {
        let mut search_start = 0;
        while let Some(rel) = haystack[search_start..].find(needle) {
            let idx = search_start + rel;
            let prev = haystack[..idx].chars().next_back();
            let next = haystack[idx + needle.len()..].chars().next();
            let left_ok = prev.is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_');
            let right_ok = next.is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_');
            if left_ok && right_ok {
                return true;
            }
            search_start = idx + needle.len();
        }
        false
    }

    /// Returns true if `node` or any of its descendants has the given kind.
    pub(crate) fn has_descendant_kind(node: Node, kind: &str) -> bool {
        if node.kind() == kind {
            return true;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if Self::has_descendant_kind(child, kind) {
                return true;
            }
        }
        false
    }

    /// `properties` block.
    pub(crate) fn inside_property(node: Node) -> bool {
        // Probed: MATLAB's MCPLD is silent in `arguments` blocks, which reuse the
        // `property` node for their entries (`x (1,:)`, `opts.?double = 3`).
        let mut current = node.parent();
        let mut in_property = false;
        while let Some(n) = current {
            match n.kind() {
                "arguments" | "arguments_statement" | "function_definition" => return false,
                "property" => in_property = true,
                "properties" => return in_property,
                _ => {}
            }
            current = n.parent();
        }
        false
    }

    /// Returns the nearest ancestor of `node` with the given kind.
    fn ancestor_of_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
        let mut current = node.parent();
        while let Some(n) = current {
            if n.kind() == kind {
                return Some(n);
            }
            current = n.parent();
        }
        None
    }

    /// Returns the text of the first child of `node` with the given kind.
    fn first_child_text<'tree>(
        node: Node<'tree>,
        kind: &str,
        source: &'tree str,
    ) -> Option<&'tree str> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == kind {
                return Some(&source[child.start_byte()..child.end_byte()]);
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Unterminated strings and comments (STRIN, DOUQT, INBLK)
    // -----------------------------------------------------------------------

    /// transpose operator, not a string delimiter.
    pub(crate) fn is_transpose_position(source: &str, pos: usize) -> bool {
        match source[..pos].chars().next_back() {
            Some(c) => c.is_ascii_alphanumeric() || c == ')' || c == ']' || c == '}' || c == '.',
            None => false,
        }
    }

    // -----------------------------------------------------------------------
    // Number literal validation (BADFP, BADHBH, BADHBB, BADHBHT, BADHBBT,
    // HEXTOOLONG, BINARYTOOLONG)
    // -----------------------------------------------------------------------

    /// Collect all `number` nodes and all ERROR nodes in document order.
    fn collect_numbers_and_errors<'a>(
        node: Node<'a>,
        numbers: &mut Vec<Node<'a>>,
        errors: &mut Vec<Node<'a>>,
    ) {
        if node.kind() == "number" {
            numbers.push(node);
        }
        if node.is_error() {
            errors.push(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_numbers_and_errors(child, numbers, errors);
        }
    }

    /// True if the node is contained within an ERROR node.
    pub(crate) fn inside_error(mut node: Node) -> bool {
        while let Some(parent) = node.parent() {
            if parent.is_error() {
                return true;
            }
            node = parent;
        }
        false
    }

    /// Parse a hex or binary literal into (is_hex, digit count, type suffix).
    pub(crate) fn parse_hex_binary_literal(text: &str) -> Option<(bool, usize, Option<&str>)> {
        let (is_hex, rest) = if text.starts_with("0x") || text.starts_with("0X") {
            (true, &text[2..])
        } else if text.starts_with("0b") || text.starts_with("0B") {
            (false, &text[2..])
        } else {
            return None;
        };

        if rest.is_empty() {
            return None;
        }

        let suffix = if rest.len() >= 3 && Self::suffix_bits(&rest[rest.len() - 3..]).is_some() {
            Some(&rest[rest.len() - 3..])
        } else if rest.len() >= 2 && Self::suffix_bits(&rest[rest.len() - 2..]).is_some() {
            Some(&rest[rest.len() - 2..])
        } else {
            None
        };

        let digits = match suffix {
            Some(s) => &rest[..rest.len() - s.len()],
            None => rest,
        };

        if digits.is_empty() {
            return None;
        }

        Some((is_hex, digits.chars().count(), suffix))
    }

    /// Bit width for a valid type suffix (`u8`/`s8` ... `u64`/`s64`), or None.
    pub(crate) fn suffix_bits(suffix: &str) -> Option<usize> {
        if suffix.len() < 2 {
            return None;
        }
        let kind = suffix.as_bytes()[0];
        let is_valid_kind = kind == b'u' || kind == b'U' || kind == b's' || kind == b'S';
        if !is_valid_kind {
            return None;
        }
        match &suffix[1..] {
            "8" => Some(8),
            "16" => Some(16),
            "32" => Some(32),
            "64" => Some(64),
            _ => None,
        }
    }

    /// Maximum number of digits a type suffix allows for a literal.
    pub(crate) fn max_digits_for_suffix(suffix: &str, is_hex: bool) -> Option<usize> {
        let bits = Self::suffix_bits(suffix)?;
        if is_hex {
            Some(bits / 4)
        } else {
            Some(bits)
        }
    }

    /// Build an Error-severity diagnostic pointing at a number literal node.
    pub(crate) fn number_diagnostic(
        rule_id: &'static str,
        message: &str,
        node: Node,
    ) -> Diagnostic {
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
    }

    // -----------------------------------------------------------------------
    // Call/argument syntax validation (SBTMP, FVACI, FVACS, FVAMI)
    // -----------------------------------------------------------------------

    pub(crate) fn push_name_value_diagnostic(
        &self,
        name: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // A valid MATLAB identifier is a legitimate name=value name.
        if name.kind() == "identifier" {
            return;
        }
        // Only a quoted name is reported here. `f(a.b=1)`, `f(1=2)` and the like are
        // NOPAR/SYNER in MATLAB (walk_dotted_name_value_calls), not FVAMI.
        if name.kind() != "string" && !Self::has_string_child(name) {
            return;
        }
        // A quoted name with identifier text is FVACS (`'a'=1`); other text is FVAMI
        // (`'Bad Name'=1`, probed).
        let text = &source[name.start_byte()..name.end_byte()];
        let inner = text.trim_matches(['\'', '"']);
        let is_identifier = inner.starts_with(|c: char| c.is_ascii_alphabetic())
            && inner.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        let check_id = if is_identifier { "FVACS" } else { "FVAMI" };
        if !self.is_check_enabled(check_id) {
            return;
        }
        let pos = name.start_position();
        let message = if is_identifier {
            "Using a character vector or string as a name in name=value syntax is not supported. Remove the quotes around the name."
        } else {
            "Name in name-value argument syntax must be a valid MATLAB identifier."
        };
        diagnostics.push(Diagnostic {
            rule_id: check_id,
            message: message.to_string(),
            severity: Severity::Error,
            byte_range: name.byte_range(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        });
    }

    pub(crate) fn find_validation_functions(node: Node) -> Option<Node> {
        if node.kind() == "validation_functions" {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = Self::find_validation_functions(child) {
                return Some(found);
            }
        }
        None
    }

    pub(crate) fn arguments_child(node: Node) -> Option<Node> {
        (0..node.child_count()).find_map(|i| node.child(i).filter(|c| c.kind() == "arguments"))
    }

    pub(crate) fn has_eq_token(node: Node) -> bool {
        (0..node.child_count()).any(|i| node.child(i).is_some_and(|c| c.kind() == "="))
    }

    pub(crate) fn has_string_child(node: Node) -> bool {
        (0..node.child_count()).any(|i| node.child(i).is_some_and(|c| c.kind() == "string"))
    }

    pub(crate) fn last_named_child(node: Node) -> Option<Node> {
        let mut last = None;
        for i in 0..node.child_count() {
            if let Some(c) = node.child(i) {
                if c.is_named() {
                    last = Some(c);
                }
            }
        }
        last
    }

    pub(crate) fn call_error_nodes(call: Node) -> Vec<Node> {
        let mut errors = Vec::new();
        for i in 0..call.child_count() {
            if let Some(c) = call.child(i) {
                if c.is_error() {
                    errors.push(c);
                }
                if c.kind() == "arguments" {
                    for j in 0..c.child_count() {
                        if let Some(cc) = c.child(j) {
                            if cc.is_error() {
                                errors.push(cc);
                            }
                        }
                    }
                }
            }
        }
        errors
    }

    pub(crate) fn last_descendant(node: Node) -> Option<Node> {
        let child_count = node.child_count();
        if child_count == 0 {
            return Some(node);
        }
        let last_child = node.child(child_count - 1)?;
        Self::last_descendant(last_child)
    }
}

impl Rule for SyntaxErrorsEngine {
    fn id(&self) -> &'static str {
        "SYNTAX_ERRORS_ENGINE"
    }

    fn description(&self) -> &'static str {
        "Parser-level and semantic syntax validation"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn category(&self) -> Category {
        Category::SyntaxErrors
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        // File-level only — no node subscriptions.
        &[]
    }

    fn can_be_disabled(&self) -> bool {
        true
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let root = ctx.tree.root_node();
        let source = ctx.source;

        // 1. File name validation (BDFIL).
        diagnostics.extend(self.check_file_name(ctx));

        // 2. Collect skip ranges (comments and strings) for source scanning.
        let mut skip_ranges = Vec::new();
        Self::collect_skip_ranges(root, &mut skip_ranges);

        // 3. Source text scanning (BADNE, BADOT, TWOCM, BADCH, BADSP).
        diagnostics.extend(self.scan_source_text(source, &skip_ranges));

        // 4. Unicode directional formatting characters (BADCT).
        diagnostics.extend(self.check_directional_formatting(source, &skip_ranges));

        // 5. Tree ERROR/MISSING node analysis (SYNER, NOPAR2, EOLPAR, ENDPAR, ENDCT).
        diagnostics.extend(self.check_error_nodes(root, source));

        // 5b. SEPEXC: whitespace-separated input names in a function header. MATLAB
        //     reports these instead of the generic SYNER on the same line.
        let sepexc = self.check_sepexc(source);
        diagnostics.retain(|d| !(d.rule_id == "SYNER" && sepexc.iter().any(|s| s.line == d.line)));
        diagnostics.extend(sepexc);

        // 6. Number literal validation (BADFP, BADHBH, BADHBB, BADHBHT,
        //     BADHBBT, HEXTOOLONG, BINARYTOOLONG).
        diagnostics.extend(self.check_number_literals(root, source));

        // 7. Unterminated strings and comments (STRIN, DOUQT, INBLK).
        diagnostics.extend(self.check_unterminated(root, source));

        // 8. File structure validation (CLIS, CLTWO, SOFOC, SEMFU).
        diagnostics.extend(self.check_file_structure(root, source));

        // 9. Function name validation (FNDOT, FNSWA).
        diagnostics.extend(self.check_function_names(root, source));

        // 10. Statement validation (NOLHS, SEPEXR, REDEF).
        diagnostics.extend(self.check_statements(root, source));

        // 10b. SEPEXR for a `case` value followed by a statement on its line
        //      (`case 1 disp(1)`). The grammar makes these lines ERROR nodes, so
        //      the SYNER on the same line goes.
        let case_separators = self.check_case_separators(root, source);
        let case_lines: Vec<usize> = case_separators.iter().map(|d| d.line).collect();
        diagnostics.retain(|d| !(d.rule_id == "SYNER" && case_lines.contains(&d.line)));
        diagnostics.extend(case_separators);

        // 11. Assignment LHS validation (UNSET, LHROW).
        diagnostics.extend(self.check_assignment_lhs(root, source));

        // 12. Reserved words and invalid ~ usage (RESWD, SYNEND, MCPLD,
        //     BADNOT, BADNOTLHS).
        diagnostics.extend(self.check_reserved_and_not(root, source));

        // 13. Call/argument syntax (SBTMP, FVACI, FVACS, FVAMI).
        diagnostics.extend(self.check_call_syntax(root, source));

        // 14. Argument validation order (FVSYN, and MCPLD in properties).
        diagnostics.extend(self.check_validation_order(root, source));

        // 15. A generic SYNER inside a token that a lexical check already reports
        //     is a duplicate: MATLAB names the specific check (`1 != 2` is BADNE,
        //     `[1,,2]` is TWOCM, `0xZ1` is BADHBH), not a parse error.
        let specific: Vec<Range<usize>> = diagnostics
            .iter()
            .filter(|d| LEXICAL_CHECKS.contains(&d.rule_id))
            .map(|d| d.byte_range.clone())
            .collect();
        // An unterminated string makes tree-sitter misread the rest of the file.
        // MATLAB reports only the STRIN, so SYNER after it is a cascade.
        let unterminated = diagnostics
            .iter()
            .filter(|d| matches!(d.rule_id, "STRIN" | "DOUQT"))
            .map(|d| d.byte_range.start)
            .min();
        diagnostics.retain(|d| {
            d.rule_id != "SYNER"
                || (unterminated.is_none_or(|s| d.byte_range.start < s)
                    && !specific
                        .iter()
                        .any(|r| r.start < d.byte_range.end && d.byte_range.start < r.end))
        });

        // A `,,` inside parentheses is reported at its second comma (check_file): the SYNER
        // that tree-sitter puts on the first comma is a duplicate.
        diagnostics.retain(|d| {
            !(d.rule_id == "SYNER"
                && source.as_bytes().get(d.byte_range.start) == Some(&b',')
                && source.as_bytes().get(d.byte_range.start + 1) == Some(&b','))
        });
        // One diagnostic per check ID and position: several error nodes can share a token.
        let mut seen = HashSet::new();
        diagnostics.retain(|d| {
            !matches!(
                d.rule_id,
                "SYNER" | "NOPAR" | "NOPAR2" | "EOLPAR" | "ENDPAR" | "BADNOT"
            ) || seen.insert((d.rule_id, d.byte_range.start))
        });

        diagnostics
    }
}

/// Checks that report a specific lexical error. A SYNER on the same token is a duplicate.
const LEXICAL_CHECKS: &[&str] = &[
    "BADNE",
    "TWOCM",
    "BADCH",
    "BADSP",
    "BADCT",
    "BADFP",
    "BADHBH",
    "BADHBB",
    "BADHBHT",
    "BADHBBT",
    "HEXTOOLONG",
    "BINARYTOOLONG",
    "STRIN",
    "DOUQT",
    "LHROW",
    "BADNOT",
];

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "SYNTAX_ERRORS_ENGINE",
    SyntaxErrorsEngine::from_config
));

/// The 1-based line and column of byte `at` in `source`.
pub(crate) fn line_col(source: &str, at: usize) -> (usize, usize) {
    let line_start = source[..at].rfind('\n').map_or(0, |i| i + 1);
    (source[..at].matches('\n').count() + 1, at - line_start + 1)
}

/// The offset of the last opening bracket in `text` that is still open at its end.
pub(crate) fn last_unmatched_opener(text: &str) -> Option<usize> {
    let mut open = Vec::new();
    for (i, c) in text.char_indices() {
        match c {
            '(' | '[' | '{' => open.push(i),
            ')' | ']' | '}' => {
                open.pop();
            }
            _ => {}
        }
    }
    open.pop()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

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

    // -- disabled_checks config ---------------------------------------------

    #[test]
    fn disabled_checks_turn_off_checks() {
        let engine = engine_with_disabled(&["BADNE", "TWOCM"]);
        let diags = lint_file(&*engine, "x = 1 != 2; y = [1,,2];\n");
        assert!(!has_id(&diags, "BADNE"), "got: {diags:?}");
        assert!(!has_id(&diags, "TWOCM"), "got: {diags:?}");
    }
}
