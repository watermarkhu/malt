//! # NOSEMI: Trailing Semicolon Missing
//!
//! ```malt
//! id = "NOSEMI"
//! title = "Trailing Semicolon Missing"
//! category = "formatting"
//! severity = "info"
//! fix = true
//! icon = "lucide/code"
//! ```
//!
//! ## Rule
//!
//! Flags MATLAB statements that produce output without a trailing semicolon.
//! In MATLAB, such statements print their result to the console, which is
//! almost always unintentional in production code and can cause significant
//! performance degradation in loops.
//!
//! ## Fix
//!
//! Inserts a trailing semicolon immediately after the statement, suppressing
//! the console output without changing program behavior.
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! x = compute_value()
//! data = load('file.mat')
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! x = compute_value();
//! data = load('file.mat');
//! ```
//!
//! ### Fixed
//!
//! ```diff
//! - x = compute_value()
//! + x = compute_value();
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.NOSEMI]
//! severity = "info"
//! ignore_functions = ["disp", "fprintf", "warning", "error"]
//! ```

use malt_core::{Category, Config, Diagnostic, Fix, NodeContext, Rule, Severity};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for NOSEMI.
///
/// Deserialized from the `[lint.rules.NOSEMI]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct NosemiConfig {
    /// Function names to ignore (statements calling these functions won't be
    /// flagged even without a semicolon). Useful for intentional output
    /// functions like `disp`, `fprintf`, etc.
    #[serde(default)]
    pub ignore_functions: Vec<String>,
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Rule NOSEMI: flags statements at the top level of a block that do not end
/// with a trailing semicolon.
///
/// Targets `assignment`, `function_call`, and `command` nodes. Only fires when
/// the node is a direct child of `source_file` or `block` (i.e., at statement
/// level, not as a sub-expression).
pub struct Nosemi {
    config: NosemiConfig,
}

/// Statement-level node types that should typically end with a semicolon.
const TARGET_NODES: &[&str] = &["assignment", "function_call", "command"];

/// Parent node types that indicate statement-level context.
const STATEMENT_PARENTS: &[&str] = &["source_file", "block"];

impl Nosemi {
    /// Factory constructor. Reads rule-specific params from config.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: NosemiConfig = config.rule_params("NOSEMI");
        Box::new(Self {
            config: rule_config,
        })
    }
}

impl Rule for Nosemi {
    fn id(&self) -> &'static str {
        "NOSEMI"
    }

    fn description(&self) -> &'static str {
        "Statement without trailing semicolon may produce unintended console output"
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn category(&self) -> Category {
        Category::Formatting
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        let node = ctx.node;

        // Only fire at statement level: parent must be source_file or block.
        // This prevents flagging function_call nodes that are sub-expressions
        // (e.g., `x = foo(bar(1))` — bar(1) is not a statement).
        let is_statement_level = node
            .parent()
            .map(|p| STATEMENT_PARENTS.contains(&p.kind()))
            .unwrap_or(false);

        if !is_statement_level {
            return Vec::new();
        }

        // If this is a function_call or command, check if the function name
        // is in the ignore list.
        if !self.config.ignore_functions.is_empty() {
            if let Some(func_name) = extract_function_name(node, ctx.source) {
                if self.config.ignore_functions.iter().any(|f| f == func_name) {
                    return Vec::new();
                }
            }
        }

        // Check if the statement is followed by a semicolon.
        let has_semicolon = has_trailing_semicolon(node, ctx.source);

        if has_semicolon {
            return Vec::new();
        }

        // Compute position for the diagnostic.
        let start = node.start_position();
        let end_byte = node.end_byte();

        vec![Diagnostic {
            rule_id: self.id(),
            message: "Extra semicolon is unnecessary.".to_string(),
            severity: self.severity(),
            byte_range: node.start_byte()..end_byte,
            line: start.row + 1, // tree-sitter is 0-indexed
            column: start.column + 1,
            fix: Some(Fix::insert(end_byte, ";")),
        }]
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the function name from a `function_call` or `command` node.
fn extract_function_name<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    match node.kind() {
        "function_call" => {
            // The `name` field of a function_call node.
            let name_node = node.child_by_field_name("name")?;
            // For simple calls, the name is an identifier.
            // For method calls (obj.method), take the full text.
            Some(&source[name_node.start_byte()..name_node.end_byte()])
        }
        "command" => {
            // The first child of a command is the command_name.
            let name_node = node.child(0)?;
            if name_node.kind() == "command_name" {
                Some(&source[name_node.start_byte()..name_node.end_byte()])
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Determine whether a statement node is followed by a semicolon.
///
/// In tree-sitter-matlab's grammar, semicolons (`;`) and newlines are statement
/// terminators that appear as sibling nodes after the statement. We check the
/// source text immediately following the node for a `;` (skipping whitespace
/// on the same line).
fn has_trailing_semicolon(node: tree_sitter::Node, source: &str) -> bool {
    // Strategy 1: Check next named/unnamed siblings for ";"
    let mut sibling = node.next_sibling();
    while let Some(sib) = sibling {
        let kind = sib.kind();
        // Skip over comment nodes that might appear between statement and semicolon
        if kind == "comment" || kind == "line_continuation" {
            sibling = sib.next_sibling();
            continue;
        }
        // If the next meaningful sibling is ";", the statement is terminated.
        if kind == ";" {
            return true;
        }
        // Any other sibling means no semicolon follows.
        break;
    }

    // Strategy 2: Fallback — scan the source text after the node end.
    // This handles cases where the semicolon might not be a separate node.
    let after = &source[node.end_byte()..];
    for ch in after.chars() {
        match ch {
            ';' => return true,
            // Stop at newline — semicolon must be on the same line.
            '\n' | '\r' => return false,
            // Skip inline whitespace.
            ' ' | '\t' => continue,
            // Any other character means something else follows.
            _ => return false,
        }
    }

    false
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
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        Nosemi::from_config(&Config::default())
    }

    // -- assignment ----------------------------------------------------------

    #[test]
    fn assignment_without_semicolon_fires() {
        let diags = lint_nodes(&*engine(), "x = 5\n");
        assert!(has_id(&diags, "NOSEMI"), "got: {diags:?}");
    }

    #[test]
    fn assignment_with_semicolon_ok() {
        let diags = lint_nodes(&*engine(), "x = 5;\n");
        assert!(!has_id(&diags, "NOSEMI"), "got: {diags:?}");
    }

    #[test]
    fn assignment_in_function_without_semicolon_fires() {
        let source = "function f()\n    x = compute_value()\nend\n";
        let diags = lint_nodes(&*engine(), source);
        assert!(has_id(&diags, "NOSEMI"), "got: {diags:?}");
    }

    #[test]
    fn assignment_in_function_with_semicolon_ok() {
        let source = "function f()\n    x = compute_value();\nend\n";
        let diags = lint_nodes(&*engine(), source);
        assert!(!has_id(&diags, "NOSEMI"), "got: {diags:?}");
    }

    // -- function_call -------------------------------------------------------

    #[test]
    fn function_call_without_semicolon_fires() {
        let diags = lint_nodes(&*engine(), "compute_value()\n");
        assert!(has_id(&diags, "NOSEMI"), "got: {diags:?}");
    }

    #[test]
    fn function_call_with_semicolon_ok() {
        let diags = lint_nodes(&*engine(), "compute_value();\n");
        assert!(!has_id(&diags, "NOSEMI"), "got: {diags:?}");
    }

    #[test]
    fn subexpression_call_not_flagged() {
        let diags = lint_nodes(&*engine(), "x = compute_value(helper(1));\n");
        assert!(!has_id(&diags, "NOSEMI"), "got: {diags:?}");
    }

    // -- command -------------------------------------------------------------

    #[test]
    fn command_without_semicolon_fires() {
        let diags = lint_nodes(&*engine(), "disp hello\n");
        assert!(has_id(&diags, "NOSEMI"), "got: {diags:?}");
    }

    // -- fix -----------------------------------------------------------------

    #[test]
    fn diagnostic_carries_semicolon_insertion_fix() {
        let diags = lint_nodes(&*engine(), "x = 5\n");
        let diag = diags
            .iter()
            .find(|d| d.rule_id == "NOSEMI")
            .expect("NOSEMI should fire");
        let fix = diag.fix.as_ref().expect("NOSEMI should carry an auto-fix");
        assert_eq!(fix.replacement, ";");
        assert_eq!(
            fix.byte_range.start, fix.byte_range.end,
            "insertion must be zero-width"
        );
    }

    #[test]
    fn fix_applied_result_has_semicolon() {
        let source = "x = 5\n";
        let diags = lint_nodes(&*engine(), source);
        let diag = diags
            .iter()
            .find(|d| d.rule_id == "NOSEMI")
            .expect("NOSEMI should fire");
        let fix = diag.fix.as_ref().unwrap();
        let mut fixed = String::from(source);
        fixed.replace_range(fix.byte_range.clone(), &fix.replacement);
        assert!(fixed.ends_with(";\n"), "got: {fixed:?}");
    }

    // -- ignore_functions config ---------------------------------------------

    #[test]
    fn ignored_function_call_not_flagged() {
        let config = Config::from_toml(
            "[lint.rules.NOSEMI]\nseverity = \"info\"\nignore_functions = [\"disp\", \"fprintf\"]\n",
        )
        .expect("valid config");
        let rule = Nosemi::from_config(&config);
        let diags = lint_nodes(&*rule, "disp('hello')\n");
        assert!(!has_id(&diags, "NOSEMI"), "got: {diags:?}");
    }

    #[test]
    fn ignored_function_command_not_flagged() {
        let config = Config::from_toml(
            "[lint.rules.NOSEMI]\nseverity = \"info\"\nignore_functions = [\"disp\", \"fprintf\"]\n",
        )
        .expect("valid config");
        let rule = Nosemi::from_config(&config);
        let diags = lint_nodes(&*rule, "disp hello\n");
        assert!(!has_id(&diags, "NOSEMI"), "got: {diags:?}");
    }

    #[test]
    fn non_ignored_function_still_flagged() {
        let config = Config::from_toml(
            "[lint.rules.NOSEMI]\nseverity = \"info\"\nignore_functions = [\"disp\", \"fprintf\"]\n",
        )
        .expect("valid config");
        let rule = Nosemi::from_config(&config);
        let diags = lint_nodes(&*rule, "disp('hello')\ncompute_value()\n");
        assert!(has_id(&diags, "NOSEMI"), "got: {diags:?}");
    }
}
