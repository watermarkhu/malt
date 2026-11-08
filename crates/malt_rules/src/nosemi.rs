//! # NOSEMI: Extra Semicolon
//!
//! ```malt
//! id = "NOSEMI"
//! title = "Extra Semicolon"
//! category = "formatting"
//! severity = "info"
//! fix = true
//! icon = "lucide/code"
//! ```
//!
//! ## Rule
//!
//! Flags a semicolon that does nothing: one that follows another semicolon
//! (`x = 1;;`), or that ends a block statement or its header line (`end;`,
//! `if ready;`, `function f;`). A semicolon that is the only separator on its
//! line, as in `if ready; go(); end`, is not extra and is left alone.
//!
//! A statement that is *missing* its semicolon is a different finding: see
//! NOPTS, NOPRT and PRTCAL.
//!
//! ## Fix
//!
//! Deletes the extra semicolon (and the blanks in front of it).
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! x = compute_value();;
//! if ready;
//!     go();
//! end;
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! x = compute_value();
//! if ready
//!     go();
//! end
//! ```
//!
//! ### Fixed
//!
//! ```diff
//! - x = compute_value();;
//! + x = compute_value();
//! - if ready;
//! + if ready
//! - end;
//! + end
//! ```

use malt_core::{Category, Config, Diagnostic, Fix, NodeContext, Rule, Severity};
use serde::Deserialize;
use tree_sitter::Node;

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for NOSEMI.
///
/// Deserialized from the `[lint.rules.NOSEMI]` section in `.malt.toml`. The
/// rule has no parameters today; the struct keeps adding one non-breaking.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct NosemiConfig {}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Rule NOSEMI: flags semicolons that serve no purpose.
pub struct Nosemi {
    _config: NosemiConfig,
}

/// Nodes whose children are a sequence of statements and separators.
const SEQUENCES: &[&str] = &["source_file", "block", "class_definition"];

/// Nodes with a header line that a `;` may (needlessly) terminate:
/// `if x;`, `for i = 1:3;`, `function f;`, `else;`, ...
const HEADERS: &[&str] = &[
    "if_statement",
    "elseif_clause",
    "else_clause",
    "while_statement",
    "for_statement",
    "switch_statement",
    "case_clause",
    "otherwise_clause",
    "try_statement",
    "catch_clause",
    "function_definition",
];

/// Statements closed by `end`: a `;` right behind one is extra.
const END_CLOSED: &[&str] = &[
    "if_statement",
    "while_statement",
    "for_statement",
    "switch_statement",
    "try_statement",
    "function_definition",
    "class_definition",
    "properties",
    "methods",
    "events",
    "enumeration",
    "spmd_statement",
];

impl Nosemi {
    /// Factory constructor. Reads rule-specific params from config.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: NosemiConfig = config.rule_params("NOSEMI");
        Box::new(Self {
            _config: rule_config,
        })
    }
}

impl Rule for Nosemi {
    fn id(&self) -> &'static str {
        "NOSEMI"
    }

    fn description(&self) -> &'static str {
        "Extra semicolon is unnecessary"
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn category(&self) -> Category {
        Category::Formatting
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        const TARGETS: &[&str] = &[
            "source_file",
            "block",
            "class_definition",
            "if_statement",
            "elseif_clause",
            "else_clause",
            "while_statement",
            "for_statement",
            "switch_statement",
            "case_clause",
            "otherwise_clause",
            "try_statement",
            "catch_clause",
            "function_definition",
        ];
        TARGETS
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        let node = ctx.node;
        let source = ctx.source;
        let kind = node.kind();
        let mut diagnostics = Vec::new();

        let mut cursor = node.walk();
        for token in node.children(&mut cursor).filter(|c| c.kind() == ";") {
            let extra = if SEQUENCES.contains(&kind) {
                follows_extra_context(token, source)
            } else if HEADERS.contains(&kind) {
                // Directly under a header node, a `;` terminates the header line.
                ends_its_line(source, token.end_byte())
            } else {
                false
            };
            if extra {
                diagnostics.push(diagnostic(token, source));
            }
        }
        diagnostics
    }
}

/// Whether a separator in a statement sequence is extra: it repeats the one
/// before it on the same line, or it closes a block statement (`end;`).
fn follows_extra_context(token: Node, source: &str) -> bool {
    let Some(prev) = token.prev_sibling() else {
        return false;
    };
    if prev.kind() == ";" {
        let gap = &source[prev.end_byte()..token.start_byte()];
        return gap.chars().all(|c| c == ' ' || c == '\t');
    }
    // `end; z = 3` needs its `;`: without it `end z = 3` is not a statement.
    prev.is_named() && END_CLOSED.contains(&prev.kind()) && ends_its_line(source, token.end_byte())
}

/// Whether nothing but blanks or a comment follows `pos` on its line.
fn ends_its_line(source: &str, pos: usize) -> bool {
    let rest = source[pos..].split('\n').next().unwrap_or("").trim();
    rest.is_empty() || rest.starts_with('%')
}

fn diagnostic(token: Node, source: &str) -> Diagnostic {
    // Take the blanks in front of the `;` along: `end ;` becomes `end`.
    let bytes = source.as_bytes();
    let mut start = token.start_byte();
    while start > 0 && matches!(bytes[start - 1], b' ' | b'\t') {
        start -= 1;
    }
    let pos = token.start_position();
    Diagnostic {
        rule_id: "NOSEMI",
        message: "Extra semicolon is unnecessary.".to_string(),
        severity: Severity::Info,
        byte_range: token.start_byte()..token.end_byte(),
        line: pos.row + 1,
        column: pos.column + 1,
        fix: Some(Fix::new(start..token.end_byte(), "")),
    }
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new("NOSEMI", Nosemi::from_config));

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::lint_nodes;
    use malt_core::{apply_fixes, Config};

    fn engine() -> Box<dyn Rule> {
        Nosemi::from_config(&Config::default())
    }

    fn nosemi(source: &str) -> Vec<Diagnostic> {
        lint_nodes(&*engine(), source)
            .into_iter()
            .filter(|d| d.rule_id == "NOSEMI")
            .collect()
    }

    fn fixed(source: &str) -> String {
        apply_fixes(source, &nosemi(source))
    }

    #[test]
    fn flags_repeated_semicolons() {
        let d = nosemi("x = 1;;\n");
        assert_eq!(d.len(), 1);
        assert_eq!((d[0].line, d[0].column), (1, 7));
        assert_eq!(nosemi("x = 1;;;\n").len(), 2);
        assert_eq!(nosemi("x = 1; ;\n").len(), 1);
        assert_eq!(nosemi("x = 1;; y = 2;\n").len(), 1);
    }

    #[test]
    fn flags_semicolons_after_end_and_headers() {
        // `if x;`, `end;`
        assert_eq!(nosemi("if x;\n  y = 1;\nend;\n").len(), 2);
        assert_eq!(nosemi("function f;\nend;\n").len(), 2);
        assert_eq!(nosemi("for i = 1:3;\n  disp(i);\nend\n").len(), 1);
        assert_eq!(nosemi("if x\n  a = 1;\nelse;\n  a = 2;\nend\n").len(), 1);
        assert_eq!(
            nosemi("end_marker = 1;\nwhile k;\n  k = 0;\nend % done\n").len(),
            1
        );
    }

    #[test]
    fn a_missing_semicolon_is_not_this_rules_business() {
        // That is NOPTS / NOPRT / PRTCAL. NOSEMI used to flag it, with this
        // very message, and inserted a second `;` next to theirs.
        assert!(nosemi("x = 1\n").is_empty());
        assert!(nosemi("disp(1)\n").is_empty());
    }

    #[test]
    fn leaves_needed_semicolons_alone() {
        for src in [
            "x = 1;\n",
            "x = 1; y = 2;\n",
            // the only separator between the condition and the body
            "if x; y = 1; end\n",
            "if x, y = 1; end\n",
            "while k; k = 0; end\n",
            // `end` followed by another statement on the line
            "if a, b; end; z = 3;\n",
            // a shell escape owns its semicolon: it is part of the command
            "!ls;\n",
            "x = 1;\n!echo hi;\n",
        ] {
            assert!(nosemi(src).is_empty(), "{src:?} got: {:?}", nosemi(src));
        }
    }

    #[test]
    fn fix_deletes_the_semicolon_and_converges() {
        for (src, expected) in [
            ("x = 1;;\n", "x = 1;\n"),
            ("x = 1; ;\n", "x = 1;\n"),
            ("x = 1;;; y = 2;\n", "x = 1; y = 2;\n"),
            ("if x;\n  y = 1;\nend;\n", "if x\n  y = 1;\nend\n"),
            ("if x ; % c\n  y = 1;\nend ;\n", "if x % c\n  y = 1;\nend\n"),
            ("function f;\nend;\n", "function f\nend\n"),
        ] {
            let once = fixed(src);
            assert_eq!(once, expected, "{src:?}");
            assert!(nosemi(&once).is_empty(), "{once:?} must be clean");
        }
    }
}
