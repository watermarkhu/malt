//! # 4.5-C: Forward-compat version gates and import validation
//!
//! These generic compatibility checks detect language features gated on a
//! minimum MATLAB version (forward compatibility) plus malformed `import`
//! arguments that the name-lookup table cannot match:
//!
//! - **FCLEN/FCCPV/FCDQS/FCFAV/FCHBL/FCLFS/FCNVA** — language constructs that
//!   require a newer MATLAB release.
//! - **IMPIVD/IMPKEY** — malformed / reserved-word `import` arguments.
//! - **REDEFGI/REDEFGG** — `global` declaration misuse (file-level).
//! - **NSTIMP** — nested functions inheriting `import` statements (file-level).
//!
//! The unset/shared-variable behavior-change checks (SHVAI, IDISVARHIGH,
//! IDISVARLOW) live in the sibling `check_unset_vars` module (Phase 7).

use super::super::CompatibilityEngine;
use malt_core::{Diagnostic, Severity};
use std::collections::HashSet;
use tree_sitter::Node;

/// A long identifier is any `identifier` node exceeding 63 characters.
const FCLEN_MAX: usize = 63;

/// Messages for the version-gate checks.
const FCLEN_MSG: &str = "Identifiers longer than 63 characters are not supported before R2025a.";
const FCCPV_MSG: &str = "Class property validation is not available before R2017a.";
const FCDQS_MSG: &str = "Double-quoted strings are not available before R2017a. Use character vectors for scalar strings or cell arrays of character vectors for string arrays.";
const FCFAV_MSG: &str = "Function argument validation is not available before R2019b.";
const FCHBL_MSG: &str = "Hexadecimal and binary literals are not available before R2019b. Use 'hex2dec' and 'bin2dec' instead.";
const FCLFS_MSG: &str = "Local functions in a script are not available before R2016b.";
const FCNVA_MSG: &str =
    "Name=Value syntax is not available before R2021a. Use comma-separated syntax instead.";
const IMPIVD_MSG: &str =
    "Malformed import argument VAR_NAME will not be supported in a future release.";
const IMPKEY_MSG: &str = "Importing VAR_NAME will not be supported in a future release because VAR_NAME is a reserved word.";
const REDEFGI_MSG: &str = "Declaring an input or output variable to be global might not be supported in a future release.";
const REDEFGG_MSG: &str =
    "Declaring a variable to be global more than once might not be supported in a future release.";
const NSTIMP_MSG: &str =
    "Nested functions now inherit import statements from this parent function. If the nested functions intend to call functions on the path, ensure that the imported namespaces do not contain functions with the same name.";

/// MATLAB reserved words (subset — for IMPKEY).
const RESERVED: &[&str] = &[
    "function",
    "end",
    "if",
    "else",
    "elseif",
    "for",
    "while",
    "switch",
    "case",
    "otherwise",
    "return",
    "break",
    "continue",
    "global",
    "persistent",
    "try",
    "catch",
    "classdef",
    "properties",
    "methods",
    "events",
    "enumeration",
    "arguments",
    "parfor",
    "spmd",
    "import",
    "true",
    "false",
];

/// Dispatch a node to the relevant check.
pub(crate) fn collect_checks(
    engine: &CompatibilityEngine,
    node: Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match node.kind() {
        "identifier" => engine.check_fclen(node, source, diagnostics),
        "string" => engine.check_fcdqs(node, source, diagnostics),
        "number" => engine.check_fchbl(node, source, diagnostics),
        "arguments_statement" => engine.check_fcfav(node, diagnostics),
        "properties" => engine.check_fccpv(node, diagnostics),
        "function_definition" => engine.check_fclfs(node, diagnostics),
        "arguments" => engine.check_fcnva(node, diagnostics),
        "command" => engine.check_import_command(node, source, diagnostics),
        _ => {}
    }
}

/// File-level checks: REDEFGI, REDEFGG (global misuse) and NSTIMP (nested
/// functions inheriting import statements).
pub(crate) fn collect_file_checks(
    engine: &CompatibilityEngine,
    tree: &tree_sitter::Tree,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    engine.check_global_redefinitions(tree.root_node(), source, diagnostics);
    engine.check_nested_import_inheritance(tree.root_node(), source, diagnostics);
}

impl CompatibilityEngine {
    /// FCLEN: an identifier longer than 63 characters.
    fn check_fclen(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let text = &source[node.start_byte()..node.end_byte()];
        if text.len() > FCLEN_MAX {
            push_diag("FCLEN", FCLEN_MSG, node, Severity::Warning, diagnostics);
        }
    }

    /// FCDQS: a double-quoted string literal.
    fn check_fcdqs(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let text = &source[node.start_byte()..node.end_byte()];
        if text.starts_with('"') {
            push_diag("FCDQS", FCDQS_MSG, node, Severity::Error, diagnostics);
        }
    }

    /// FCHBL: a hex (`0x`) or binary (`0b`) literal.
    fn check_fchbl(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let text = &source[node.start_byte()..node.end_byte()];
        if text.starts_with("0x") || text.starts_with("0b") {
            push_diag("FCHBL", FCHBL_MSG, node, Severity::Error, diagnostics);
        }
    }

    /// FCFAV: a function `arguments` block (argument validation).
    fn check_fcfav(&self, node: Node, diagnostics: &mut Vec<Diagnostic>) {
        push_diag("FCFAV", FCFAV_MSG, node, Severity::Error, diagnostics);
    }

    /// FCCPV: a class property with validation (type constraint).
    fn check_fccpv(&self, node: Node, diagnostics: &mut Vec<Diagnostic>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "property" {
                // A property with a type constraint has >1 named child
                // (name identifier + type/validation).
                let named: Vec<_> = child
                    .children(&mut child.walk())
                    .filter(|c| c.is_named())
                    .collect();
                if named.len() >= 2 {
                    push_diag("FCCPV", FCCPV_MSG, node, Severity::Error, diagnostics);
                    return;
                }
            }
        }
    }

    /// FCLFS: a script file containing local function definitions.
    fn check_fclfs(&self, node: Node, diagnostics: &mut Vec<Diagnostic>) {
        // A script with local functions has top-level statements BEFORE the
        // function_definition (both direct children of source_file). A pure
        // function file starts with the function definition, so a preceding
        // non-function sibling marks this as a script local function.
        if let Some(parent) = node.parent() {
            if parent.kind() == "source_file" {
                if let Some(prev) = node.prev_sibling() {
                    if prev.kind() != "function_definition" {
                        push_diag("FCLFS", FCLFS_MSG, node, Severity::Error, diagnostics);
                    }
                }
            }
        }
    }

    /// FCNVA: `Name=Value` syntax in a call's arguments.
    fn check_fcnva(&self, node: Node, diagnostics: &mut Vec<Diagnostic>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "=" {
                push_diag("FCNVA", FCNVA_MSG, node, Severity::Error, diagnostics);
                return;
            }
        }
    }

    /// Import handling: NSTIMP (nested inherit), IMPIVD/IMPKEY (malformed /
    /// reserved-word args).
    fn check_import_command(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let Some(name) = node.child(0) else {
            return;
        };
        if name.kind() != "command_name" {
            return;
        }
        let cmd = &source[name.start_byte()..name.end_byte()];
        if cmd != "import" {
            return;
        }
        // Collect the imported package argument.
        let mut args: Vec<String> = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "command_argument" {
                args.push(source[child.start_byte()..child.end_byte()].to_string());
            }
        }
        for arg in &args {
            // IMPKEY: importing a reserved word.
            let base = arg.split('.').next().unwrap_or(&arg[..]);
            if RESERVED.contains(&base) {
                push_diag(
                    "IMPKEY",
                    IMPKEY_MSG.replace("VAR_NAME", arg),
                    node,
                    Severity::Warning,
                    diagnostics,
                );
                return;
            }
            // IMPIVD: malformed import argument (no package structure).
            if !arg.contains('.') && arg.chars().any(|c| !c.is_alphanumeric() && c != '_') {
                push_diag(
                    "IMPIVD",
                    IMPIVD_MSG.replace("VAR_NAME", arg),
                    node,
                    Severity::Warning,
                    diagnostics,
                );
                return;
            }
        }
    }

    /// REDEFGI/REDEFGG: a `global` declaration names a function input/output,
    /// or the same variable is declared global more than once in a scope.
    fn check_global_redefinitions(
        &self,
        root: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Walk all global_operator nodes in the file.
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if node.kind() == "global_operator" {
                // Collect the declared names.
                let mut names: Vec<Node> = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        names.push(child);
                    }
                }
                if !names.is_empty() {
                    self.check_global_names(node, &names, source, diagnostics);
                }
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }
    }

    /// Emit REDEFGI/REDEFGG for one `global` statement.
    fn check_global_names(
        &self,
        global_node: Node,
        names: &[Node],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // REDEFGI: a declared name matches a function input/output. Only makes
        // sense inside a function_definition.
        if let Some(func) = enclosing_function(global_node) {
            let signature: HashSet<&str> = function_signature_names(func, source);
            for &name in names {
                let text = &source[name.start_byte()..name.end_byte()];
                if signature.contains(text) {
                    push_diag(
                        "REDEFGI",
                        REDEFGI_MSG,
                        global_node,
                        Severity::Warning,
                        diagnostics,
                    );
                    break;
                }
            }
        }
        // REDEFGG: the same name declared global more than once in the file.
        // Match the declaration against all other global declarations.
        for &name in names {
            let text = &source[name.start_byte()..name.end_byte()];
            if global_declared_earlier(global_node, text, source) {
                push_diag("REDEFGG", REDEFGG_MSG, name, Severity::Warning, diagnostics);
            }
        }
    }

    /// NSTIMP: a function with nested functions declares an `import` statement.
    fn check_nested_import_inheritance(
        &self,
        root: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Walk function_definition nodes that directly contain nested
        // function_definition children (parent functions).
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if node.kind() == "function_definition" && has_nested_function(node) {
                self.check_function_import(node, source, diagnostics);
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }
    }

    /// Emit NSTIMP when a parent function's block contains an `import` command.
    fn check_function_import(&self, func: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        // The function's body block: direct child named `block`.
        if let Some(block) = block_of(func) {
            if block_has_import(block, source) {
                push_diag("NSTIMP", NSTIMP_MSG, func, Severity::Warning, diagnostics);
            }
        }
    }
}

/// Push a diagnostic at the node's location.
fn push_diag(
    id: &'static str,
    message: impl Into<String>,
    node: Node,
    severity: Severity,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let pos = node.start_position();
    diagnostics.push(Diagnostic {
        rule_id: id,
        message: message.into(),
        severity,
        byte_range: node.start_byte()..node.end_byte(),
        line: pos.row + 1,
        column: pos.column + 1,
        fix: None,
    });
}

/// The nearest enclosing `function_definition` ancestor, if any.
fn enclosing_function<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut cur = node.parent();
    while let Some(parent) = cur {
        if parent.kind() == "function_definition" {
            return Some(parent);
        }
        cur = parent.parent();
    }
    None
}

/// The input and output variable names of a function definition.
fn function_signature_names<'a>(func: Node<'a>, source: &'a str) -> HashSet<&'a str> {
    let mut names = HashSet::new();
    let mut cursor = func.walk();
    for child in func.children(&mut cursor) {
        match child.kind() {
            // Outputs: `function y = f(x)` has a function_output node.
            "function_output" => {
                collect_identifiers(child, source, &mut names);
            }
            // Inputs: the identifiers inside the function_arguments parens.
            "function_arguments" => {
                collect_identifiers(child, source, &mut names);
            }
            _ => {}
        }
    }
    names
}

/// Collect all identifier texts under `node` into `out`.
fn collect_identifiers<'a>(node: Node<'a>, source: &'a str, out: &mut HashSet<&'a str>) {
    let mut stack = vec![node];
    while let Some(cur) = stack.pop() {
        if cur.kind() == "identifier" {
            out.insert(&source[cur.start_byte()..cur.end_byte()]);
            continue;
        }
        let mut cursor = cur.walk();
        for child in cur.children(&mut cursor) {
            stack.push(child);
        }
    }
}

/// True when the same variable name was declared `global` by an earlier
/// statement in the file.
fn global_declared_earlier<'a>(current: Node<'a>, name: &str, source: &'a str) -> bool {
    // Walk every earlier sibling (and earlier siblings of ancestors) looking
    // for a global_operator that declares `name`.
    let mut cur = current;
    loop {
        if let Some(prev) = cur.prev_sibling() {
            if node_subtree_has_global(prev, name, source) {
                return true;
            }
            cur = prev;
        } else {
            match cur.parent() {
                Some(parent) => cur = parent,
                None => return false,
            }
        }
    }
}

/// True when any `global_operator` under `node` (excluding nested function
/// scopes) declares `name`.
fn node_subtree_has_global(node: Node, name: &str, source: &str) -> bool {
    // Only look at the direct children of a block, so nested function bodies
    // (which are children of the block too) are excluded: stop descending
    // into function_definition nodes.
    if node.kind() == "function_definition" {
        return false;
    }
    let mut stack = vec![node];
    while let Some(cur) = stack.pop() {
        if cur.kind() == "global_operator" && global_contains(cur, name, source) {
            return true;
        }
        let mut cursor = cur.walk();
        for child in cur.children(&mut cursor) {
            if child.kind() == "function_definition" {
                continue;
            }
            stack.push(child);
        }
    }
    false
}

/// True when a `global_operator` node declares `name`.
fn global_contains(node: Node, name: &str, source: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" && &source[child.start_byte()..child.end_byte()] == name {
            return true;
        }
    }
    false
}

/// True when `func` directly contains a nested `function_definition` (i.e. a
/// `function_definition` child inside its body `block`).
fn has_nested_function(func: Node) -> bool {
    let Some(block) = block_of(func) else {
        return false;
    };
    let mut cursor = block.walk();
    for child in block.children(&mut cursor) {
        if child.kind() == "function_definition" {
            return true;
        }
    }
    false
}

/// The body `block` node of a function definition, if any.
fn block_of(func: Node) -> Option<Node> {
    func.children(&mut func.walk())
        .find(|child| child.kind() == "block")
}

/// True when the block directly contains an `import` command (not inside a
/// nested function body).
fn block_has_import(block: Node, source: &str) -> bool {
    let mut cursor = block.walk();
    for child in block.children(&mut cursor) {
        if child.kind() == "command" {
            if let Some(name) = child.child(0) {
                if name.kind() == "command_name"
                    && &source[name.start_byte()..name.end_byte()] == "import"
                {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::compatibility::CompatibilityEngine;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn malt_core::Rule> {
        CompatibilityEngine::from_config(&Config::default())
    }

    #[test]
    fn fclen_long_identifier_fires() {
        let d = lint_file(&*engine(), &format!("x{} = 1;\n", "a".repeat(64)));
        assert!(has_id(&d, "FCLEN"), "got: {d:?}");
    }

    #[test]
    fn fclen_short_identifier_not_fires() {
        let d = lint_file(&*engine(), "x = 1;\n");
        assert!(!has_id(&d, "FCLEN"), "got: {d:?}");
    }

    #[test]
    fn fcdqs_double_quoted_fires() {
        let d = lint_file(&*engine(), "x = \"hello\";\n");
        assert!(has_id(&d, "FCDQS"), "got: {d:?}");
    }

    #[test]
    fn fcdqs_single_quoted_not_fires() {
        let d = lint_file(&*engine(), "x = 'hello';\n");
        assert!(!has_id(&d, "FCDQS"), "got: {d:?}");
    }

    #[test]
    fn fchbl_hex_fires() {
        let d = lint_file(&*engine(), "x = 0xFF;\n");
        assert!(has_id(&d, "FCHBL"), "got: {d:?}");
    }

    #[test]
    fn fchbl_binary_fires() {
        let d = lint_file(&*engine(), "x = 0b1010;\n");
        assert!(has_id(&d, "FCHBL"), "got: {d:?}");
    }

    #[test]
    fn fchbl_decimal_not_fires() {
        let d = lint_file(&*engine(), "x = 42;\n");
        assert!(!has_id(&d, "FCHBL"), "got: {d:?}");
    }

    #[test]
    fn fcfav_arguments_block_fires() {
        let d = lint_file(
            &*engine(),
            "function f(x)\n  arguments\n    x double\n  end\nend\n",
        );
        assert!(has_id(&d, "FCFAV"), "got: {d:?}");
    }

    #[test]
    fn fccpv_validated_property_fires() {
        let d = lint_file(
            &*engine(),
            "classdef C\n properties\n  x double = 1\n end\nend\n",
        );
        assert!(has_id(&d, "FCCPV"), "got: {d:?}");
    }

    #[test]
    fn fccpv_plain_property_not_fires() {
        let d = lint_file(&*engine(), "classdef C\n properties\n  x\n end\nend\n");
        assert!(!has_id(&d, "FCCPV"), "got: {d:?}");
    }

    #[test]
    fn fclfs_script_local_function_fires() {
        let d = lint_file(&*engine(), "x = 1;\nfunction y = helper(x)\n y = x;\nend\n");
        assert!(has_id(&d, "FCLFS"), "got: {d:?}");
    }

    #[test]
    fn fcnva_name_value_fires() {
        let d = lint_file(&*engine(), "f(Name=value);\n");
        assert!(has_id(&d, "FCNVA"), "got: {d:?}");
    }

    #[test]
    fn fcnva_comma_separated_not_fires() {
        let d = lint_file(&*engine(), "f('Name', value);\n");
        assert!(!has_id(&d, "FCNVA"), "got: {d:?}");
    }

    #[test]
    fn impkey_reserved_import_fires() {
        let d = lint_file(&*engine(), "import if;\n");
        assert!(has_id(&d, "IMPKEY"), "got: {d:?}");
    }

    #[test]
    fn normal_import_not_fires() {
        let d = lint_file(&*engine(), "import mypkg.*;\n");
        assert!(!has_id(&d, "IMPKEY"), "got: {d:?}");
        assert!(!has_id(&d, "IMPIVD"), "got: {d:?}");
    }

    #[test]
    fn fclen_assignment_long_identifier_fires() {
        let d = lint_file(&*engine(), &format!("{} = 1;\n", "y".repeat(70)));
        assert!(has_id(&d, "FCLEN"), "got: {d:?}");
    }
}

#[cfg(test)]
mod scope_tests {
    use crate::compatibility::CompatibilityEngine;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn malt_core::Rule> {
        CompatibilityEngine::from_config(&Config::default())
    }

    #[test]
    fn redefgi_global_input_fires() {
        let d = lint_file(&*engine(), "function f(a)\n global a;\nend\n");
        assert!(has_id(&d, "REDEFGI"), "got: {d:?}");
    }

    #[test]
    fn redefgi_global_output_fires() {
        let d = lint_file(&*engine(), "function y = f()\n global y;\nend\n");
        assert!(has_id(&d, "REDEFGI"), "got: {d:?}");
    }

    #[test]
    fn redefgi_normal_global_not_fires() {
        let d = lint_file(&*engine(), "function f()\n global g;\nend\n");
        assert!(!has_id(&d, "REDEFGI"), "got: {d:?}");
    }

    #[test]
    fn redefgg_twice_fires() {
        let d = lint_file(&*engine(), "function f()\n global g;\n global g;\nend\n");
        assert!(has_id(&d, "REDEFGG"), "got: {d:?}");
    }

    #[test]
    fn redefgg_once_not_fires() {
        let d = lint_file(&*engine(), "function f()\n global g;\nend\n");
        assert!(!has_id(&d, "REDEFGG"), "got: {d:?}");
    }

    #[test]
    fn redefgg_distinct_names_not_fires() {
        let d = lint_file(&*engine(), "function f()\n global g;\n global h;\nend\n");
        assert!(!has_id(&d, "REDEFGG"), "got: {d:?}");
    }

    #[test]
    fn nstimp_import_with_nested_fires() {
        let d = lint_file(
            &*engine(),
            "function p()\n import mypkg.x;\n function n()\n end\nend\n",
        );
        assert!(has_id(&d, "NSTIMP"), "got: {d:?}");
    }

    #[test]
    fn nstimp_no_import_not_fires() {
        let d = lint_file(&*engine(), "function p()\n function n()\n end\nend\n");
        assert!(!has_id(&d, "NSTIMP"), "got: {d:?}");
    }

    #[test]
    fn nstimp_import_no_nested_not_fires() {
        let d = lint_file(&*engine(), "function f()\n import mypkg.x;\nend\n");
        assert!(!has_id(&d, "NSTIMP"), "got: {d:?}");
    }

    #[test]
    fn redefgi_global_in_nested_function_fires() {
        // A nested function declaring its own input global.
        let d = lint_file(
            &*engine(),
            "function p(x)\n function n(a)\n  global a;\n end\nend\n",
        );
        assert!(has_id(&d, "REDEFGI"), "got: {d:?}");
    }
}
