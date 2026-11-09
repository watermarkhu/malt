//! # CODEGEN_ENGINE: Code Generation Constraint Checks
//!
//! ```malt
//! id = "CODEGEN_ENGINE"
//! title = "MATLAB Code Generation Constraint Checks"
//! category = "code-generation"
//! severity = "error"
//! fix = false
//! icon = "lucide/braces"
//! slug = "codegen"
//! ```
//!
//! ## Rule
//!
//! Detects MATLAB constructs that are unsupported or problematic for MATLAB
//! Coder / code generation. MATLAB code intended to be compiled to C/C++ has
//! many restrictions compared to general MATLAB code; this engine flags
//! variable-size data, growing arrays, unsupported functions, cell arrays,
//! try-catch, imports, nested functions, scripts, and similar constraints.
//! All 14 checks share a single `CodegenEngine` that dispatches node-level
//! checks on `function_call`, `command`, `try_statement`, `for_statement`,
//! `assignment`, `cell`, and `arguments_statement` nodes, plus file-level
//! checks for nested functions and script-mode files. Each diagnostic carries
//! the specific check ID (e.g. `EMFCN`, `EMSCR`).
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | EMVDF | error | no | Code generation requires that all elements of a variable are defined before indexing into the variable. Fires on an indexed assignment `x(k) = ...` when `x` is not mentioned earlier in the enclosing function. |
//! | EMGRO | error | no | For code generation, grow an array by using 'end + 1' indexing. Fires on `x(end+k) = ...` with `k` other than 1, and on `x = [x, ...]` growth inside a loop. |
//! | EMFCN | error | no | This function is not supported in code generation. For example `eval`, `str2num`, `input`, `save`, `dir`. |
//! | EMCEL | error | no | Fixed-point conversion does not support cell arrays. |
//! | EMTC | error | no | TRY/CATCH is unsupported for code generation. |
//! | EMIMP | error | no | Code generation does not support import statements. |
//! | EMNST | error | no | Fixed-point conversion does not support nested functions. |
//! | EMSCR | error | no | Code generation does not support scripts. |
//! | EMPFR | error | no | HDL code generation does not support parfor statements. |
//! | EMRIFAV | error | no | Code generation does not support repeating arguments with validation. Fires on `arguments (Repeating)` blocks that declare validation. |
//! | EMLOAD | error | no | The output of a call to LOAD is not assigned to a variable. For code generation, assign the output of LOAD to a variable without subscripting. Fires on `load` used as a statement or command, and on `x(k) = load(...)`. |
//! | PRMNOIN | error | no | For code generation, specify a binaryOccupancyMap object in the constructor of the mobileRobotPRM object. Fires on `mobileRobotPRM()` called without arguments. |
//! | LOOPPRAGMAWITHOUTFOR | error | no | A coder.loop.Control transform must be immediately followed by a for loop. Fires on a `coder.loop.*` call statement not followed by `for`. |
//! | FPASE | warning | no | Direct assignment to a possible fixed-point type is not recommended. Use the subscripted assignment syntax 'var(:) =' instead. Fires on `y = ...` when `y` was earlier assigned `fi(...)`. |
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! x = [];
//! for i = 1:10
//!     x = [x, i];     % EMGRO: grow with end+1 indexing
//! end
//! y = {1, 2};         % EMCEL: cell array
//! z = str2num('1 2'); % EMFCN: unsupported function
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! x = zeros(1, 10);
//! for i = 1:10
//!     x(i) = i;
//! end
//! y = [1, 2];
//! z = str2double('1 2');
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.CODEGEN_ENGINE]
//! severity = "error"
//! skip_checks = ["EMSCR", "EMFCN"]
//! ```

mod check_arguments_statement;
mod check_assignment;
mod check_cell;
mod check_command;
mod check_for_statement;
mod check_function_call;
mod check_try_statement;

use malt_core::{Category, Config, Diagnostic, FileContext, NodeContext, Rule, Severity};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for the code generation engine.
///
/// Deserialized from the `[lint.rules.CODEGEN_ENGINE]` section in `.malt.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CodegenEngineConfig {
    /// Check IDs to skip (e.g., `["EMSCR", "EMFCN"]`).
    #[serde(default)]
    pub skip_checks: Vec<String>,
}

// ---------------------------------------------------------------------------
// Check metadata
// ---------------------------------------------------------------------------

/// Static metadata for a single code generation check.
struct CheckMeta {
    id: &'static str,
    description: &'static str,
}

/// All code generation check definitions.
const CHECKS: &[CheckMeta] = &[
    CheckMeta {
        id: "EMVDF",
        description: "Code generation requires that all elements of a variable are defined before indexing into the variable.",
    },
    CheckMeta {
        id: "EMGRO",
        description: "For code generation, grow an array by using 'end + 1' indexing.",
    },
    CheckMeta {
        id: "EMFCN",
        description: "This function is not supported in code generation.",
    },
    CheckMeta {
        id: "EMCEL",
        description: "Fixed-point conversion does not support cell arrays.",
    },
    CheckMeta {
        id: "EMTC",
        description: "TRY/CATCH is unsupported for code generation.",
    },
    CheckMeta {
        id: "EMIMP",
        description: "Code generation does not support import statements.",
    },
    CheckMeta {
        id: "EMNST",
        description: "Fixed-point conversion does not support nested functions.",
    },
    CheckMeta {
        id: "EMSCR",
        description: "Code generation does not support scripts.",
    },
    CheckMeta {
        id: "EMPFR",
        description: "HDL code generation does not support parfor statements.",
    },
    CheckMeta {
        id: "EMRIFAV",
        description: "Code generation does not support repeating arguments with validation.",
    },
    CheckMeta {
        id: "EMLOAD",
        description: "The output of a call to LOAD is not assigned to a variable. For code generation, assign the output of LOAD to a variable without subscripting.",
    },
    CheckMeta {
        id: "PRMNOIN",
        description: "For code generation, specify a binaryOccupancyMap object in the constructor of the mobileRobotPRM object.",
    },
    CheckMeta {
        id: "LOOPPRAGMAWITHOUTFOR",
        description: "A coder.loop.Control transform must be immediately followed by a for loop.",
    },
    CheckMeta {
        id: "FPASE",
        description: "Direct assignment to a possible fixed-point type is not recommended. Use the subscripted assignment syntax 'var(:) =' instead.",
    },
];

/// Functions unsupported in code generation (reported as `EMFCN`).
///
/// Functions that code generation treats as extrinsic (`plot`, `figure`, `disp`,
/// ...) and `load` (handled by `EMLOAD`) are deliberately absent.
const UNSUPPORTED_FUNCTIONS: &[&str] = &[
    "eval",
    "evalc",
    "evalin",
    "assignin",
    "input",
    "keyboard",
    "dbstop",
    "dbclear",
    "dbcont",
    "dbstep",
    "dbup",
    "dbdown",
    "dbquit",
    "who",
    "whos",
    "clear",
    "clearvars",
    "pack",
    "diary",
    "save",
    "matfile",
    "cd",
    "ls",
    "dir",
    "mkdir",
    "rmdir",
    "javaObject",
    "javaMethod",
    "javaArray",
    "str2num",
    "actxserver",
    "matlabroot",
    "tempdir",
    "tempname",
    "run",
];

/// Look up check description by ID.
fn check_description(id: &str) -> &'static str {
    CHECKS
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.description)
        .unwrap_or("Code generation constraint violation")
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Engine covering the code generation checks.
///
/// Node-level checks fire on `function_call`, `command`, `try_statement`,
/// `for_statement`, `assignment`, and `cell`. File-level checks detect
/// nested functions and script-mode files.
pub struct CodegenEngine {
    config: CodegenEngineConfig,
}

/// Node types for node-level dispatch.
const TARGET_NODES: &[&str] = &[
    "function_call",
    "command",
    "try_statement",
    "for_statement",
    "assignment",
    "cell",
    "arguments_statement",
];

impl CodegenEngine {
    /// Factory constructor called by the rule registry.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: CodegenEngineConfig = config.rule_params("CODEGEN_ENGINE");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Whether a check ID is enabled (not in `skip_checks`).
    pub(crate) fn is_enabled(&self, check_id: &str) -> bool {
        !self.config.skip_checks.iter().any(|s| s == check_id)
    }

    // ---------------------------------------------------------------------------
    // File-level checks
    // ---------------------------------------------------------------------------

    /// File-level checks: EMNST (nested functions) and EMSCR (scripts).
    fn check_file_level(&self, tree: &tree_sitter::Tree, _source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let root = tree.root_node();

        // EMSCR: Script detection — a file whose first meaningful statement is
        // not a `function_definition` or `class_definition`.
        if self.is_enabled("EMSCR") && is_script_file(&root) {
            let start = root.start_position();
            diags.push(Diagnostic {
                rule_id: "EMSCR",
                message: check_description("EMSCR").to_string(),
                severity: Severity::Error,
                byte_range: 0..root.end_byte().min(1),
                line: start.row + 1,
                column: start.column + 1,
                fix: None,
            });
        }

        // EMNST: Nested function detection
        if self.is_enabled("EMNST") {
            Self::find_nested_functions(&root, 0, &mut diags);
        }

        diags
    }

    /// Recursively find nested function definitions (depth > 1).
    fn find_nested_functions(node: &tree_sitter::Node, depth: usize, diags: &mut Vec<Diagnostic>) {
        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                if child.kind() == "function_definition" {
                    if depth >= 1 {
                        // This is a nested function
                        diags.push(make_diag("EMNST", child));
                    }
                    // Recurse into this function to find deeper nesting
                    Self::find_nested_functions(&child, depth + 1, diags);
                } else {
                    Self::find_nested_functions(&child, depth, diags);
                }
            }
        }
    }
}

impl Rule for CodegenEngine {
    fn id(&self) -> &'static str {
        "CODEGEN_ENGINE"
    }

    fn description(&self) -> &'static str {
        "MATLAB code generation constraint checks"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn category(&self) -> Category {
        Category::CodeGeneration
    }

    fn enabled_by_default(&self) -> bool {
        // Only relevant for MATLAB Coder targets; opt in explicitly.
        false
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        match ctx.node.kind() {
            "function_call" => self.check_function_call(ctx.node, ctx.source),
            "command" => self.check_command(ctx.node, ctx.source),
            "try_statement" => self.check_try_statement(ctx.node),
            "for_statement" => self.check_for_statement(ctx.node, ctx.source),
            "assignment" => self.check_assignment(ctx.node, ctx.source),
            "cell" => self.check_cell(ctx.node),
            "arguments_statement" => self.check_arguments_statement(ctx.node, ctx.source),
            _ => Vec::new(),
        }
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        self.check_file_level(ctx.tree, ctx.source)
    }
}

// ---------------------------------------------------------------------------
// Helper: diagnostic construction
// ---------------------------------------------------------------------------

/// Build a diagnostic from a check ID and node.
pub(crate) fn make_diag(check_id: &'static str, node: tree_sitter::Node) -> Diagnostic {
    let start = node.start_position();
    Diagnostic {
        rule_id: check_id,
        message: check_description(check_id).to_string(),
        // Every code-generation check is an error except FPASE.
        severity: if check_id == "FPASE" {
            Severity::Warning
        } else {
            Severity::Error
        },
        byte_range: node.start_byte()..node.end_byte(),
        line: start.row + 1,
        column: start.column + 1,
        fix: None,
    }
}

// ---------------------------------------------------------------------------
// Helper: node text extraction
// ---------------------------------------------------------------------------

/// Extract the raw text of a node.
pub(crate) fn node_text<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// Extract function name from a `function_call` node.
pub(crate) fn extract_func_name<'a>(
    node: tree_sitter::Node<'a>,
    source: &'a str,
) -> Option<&'a str> {
    if node.kind() != "function_call" {
        return None;
    }
    let name_node = node.child_by_field_name("name")?;
    Some(&source[name_node.start_byte()..name_node.end_byte()])
}

/// Extract command name from a `command` node.
pub(crate) fn extract_command_name<'a>(
    node: tree_sitter::Node<'a>,
    source: &'a str,
) -> Option<&'a str> {
    if node.kind() != "command" {
        return None;
    }
    let name_node = node.child(0)?;
    if name_node.kind() == "command_name" {
        Some(&source[name_node.start_byte()..name_node.end_byte()])
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Helper: pattern detection
// ---------------------------------------------------------------------------

/// Check if a node is inside a loop.
pub(crate) fn is_inside_loop(node: tree_sitter::Node) -> bool {
    let mut current = node.parent();
    while let Some(p) = current {
        match p.kind() {
            "for_statement" | "while_statement" => return true,
            "function_definition" => return false,
            _ => {}
        }
        current = p.parent();
    }
    false
}

/// Check if a matrix literal contains a variable name as an element.
pub(crate) fn matrix_contains_var(node: tree_sitter::Node, source: &str, var_name: &str) -> bool {
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" && node_text(child, source) == var_name {
                return true;
            }
            if child.kind() == "row" && matrix_contains_var(child, source, var_name) {
                return true;
            }
        }
    }
    false
}

/// Returns `true` if a `source_file` is a script (top-level statements that are
/// not a `function_definition` or `class_definition`).
fn is_script_file(root: &tree_sitter::Node) -> bool {
    let count = root.child_count();
    for i in 0..count {
        let Some(child) = root.child(i) else {
            continue;
        };
        match child.kind() {
            "comment" | "line_continuation" => continue,
            "function_definition" | "class_definition" => return false,
            // Any other top-level statement means the file is a script.
            _ => return true,
        }
    }
    false
}

/// Check if the next statement after a node is a for_statement.
pub(crate) fn is_followed_by_for(node: tree_sitter::Node) -> bool {
    let mut sibling = node.next_sibling();
    while let Some(sib) = sibling {
        match sib.kind() {
            "for_statement" => return true,
            ";" | "," | "\n" | "\r\n" | "comment" | "line_continuation" => {
                sibling = sib.next_sibling();
            }
            _ => return false,
        }
    }
    false
}

/// Number of arguments passed in a `function_call` node (0 for `f()`).
pub(crate) fn call_arg_count(node: tree_sitter::Node) -> usize {
    (0..node.child_count())
        .filter_map(|i| node.child(i))
        .find(|c| c.kind() == "arguments")
        .map(|args| {
            (0..args.named_child_count())
                .filter_map(|i| args.named_child(i))
                .filter(|c| c.kind() != "comment")
                .count()
        })
        .unwrap_or(0)
}

/// Nearest enclosing `function_definition`, or the root node.
pub(crate) fn enclosing_scope(node: tree_sitter::Node) -> tree_sitter::Node {
    let mut current = node;
    while let Some(p) = current.parent() {
        if p.kind() == "function_definition" {
            return p;
        }
        current = p;
    }
    current
}

/// Whether an identifier named `name` occurs inside `scope` before byte
/// offset `before` (parameters, outputs, assignments, declarations, uses).
pub(crate) fn has_earlier_mention(
    scope: tree_sitter::Node,
    source: &str,
    name: &str,
    before: usize,
) -> bool {
    if scope.start_byte() >= before {
        return false;
    }
    if scope.kind() == "identifier" {
        return scope.end_byte() <= before && node_text(scope, source) == name;
    }
    (0..scope.child_count())
        .filter_map(|i| scope.child(i))
        .any(|c| has_earlier_mention(c, source, name, before))
}

/// Whether the text of an indexing expression grows past the end by
/// something other than exactly `end+1` (e.g. `end+2`, `end+n`).
pub(crate) fn has_end_plus_other(text: &str) -> bool {
    let norm: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let mut rest = norm.as_str();
    while let Some(pos) = rest.find("end+") {
        let after = &rest[pos + 4..];
        let is_one = after.starts_with('1')
            && !after[1..]
                .chars()
                .next()
                .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.');
        if !is_one {
            return true;
        }
        rest = after;
    }
    false
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "CODEGEN_ENGINE",
    CodegenEngine::from_config
));

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) fn engine() -> Box<dyn Rule> {
    CodegenEngine::from_config(&Config::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file, lint_nodes};
    use malt_core::Config;

    // -- EMSCR (file-level) -----------------------------------------------------

    #[test]
    fn emscr_fires_on_script_file() {
        let src = "x = 1;\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "EMSCR"), "got: {diags:?}");
    }

    #[test]
    fn emscr_not_fire_on_function_file() {
        let src = "function f()\n    x = 1;\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "EMSCR"), "got: {diags:?}");
    }

    #[test]
    fn emscr_not_fire_on_class_file() {
        let src = "classdef Foo\n    methods\n        function f()\n        end\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "EMSCR"), "got: {diags:?}");
    }

    // -- EMNST (file-level) -------------------------------------------------------

    #[test]
    fn emnst_fires_on_nested_function() {
        let src = "function a()\n    function b()\n        x = 1;\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "EMNST"), "got: {diags:?}");
    }

    #[test]
    fn emnst_not_fire_on_flat_functions() {
        let src = "function a()\n    x = 1;\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "EMNST"), "got: {diags:?}");
    }

    // -- Config-respected test -----------------------------------------------------

    #[test]
    fn config_skip_checks_disables_individual_checks() {
        let config = Config::from_toml(
            r#"
[lint.rules.CODEGEN_ENGINE]
skip_checks = ["EMSCR", "EMFCN"]
"#,
        )
        .unwrap();
        let rule = CodegenEngine::from_config(&config);

        // Skipped file-level check no longer fires.
        let src = "x = 1;\n";
        let diags = lint_file(&*rule, src);
        assert!(!has_id(&diags, "EMSCR"), "got: {diags:?}");

        // Skipped node-level check no longer fires.
        let src = "eval('x');\n";
        let diags = lint_nodes(&*rule, src);
        assert!(!has_id(&diags, "EMFCN"), "got: {diags:?}");

        // Non-skipped checks still fire.
        let src = "try\n    x = 1;\ncatch\n    y = 2;\nend\n";
        let diags = lint_nodes(&*rule, src);
        assert!(has_id(&diags, "EMTC"), "got: {diags:?}");
    }
}
