//! # Phase 7: Unset-variable and shared-variable compatibility checks
//!
//! Three behavior-change checks that MATLAB now enforces more strictly than in
//! previous releases. All three are file-level: they build a [`SymbolTable`]
//! over the whole tree and compare definition/use offsets within scopes.
//!
//! - **IDISVARLOW** — a name is indexed (`x(i)`, `x{i}`, `x.field`) before it
//!   is defined in the enclosing scope, so it could be mistaken for a function
//!   on the path. Fires on the indexing use. Only fires when the name is a
//!   genuine variable in that scope (defined somewhere in it): a name used
//!   only as a callee (`plot(x, y)`) is treated as a function and skipped, as
//!   are dotted/qualified calls (`pkg.func(x)`), file-defined functions, and
//!   builtin names.
//! - **IDISVARHIGH** — a plain read of a name with no prior definition in
//!   scope. Fires on the first such read. Callee/indexing uses are excluded
//!   (they belong to IDISVARLOW).
//! - **SHVAI** — a nested function reads/writes a name that the parent function
//!   also references, but the parent only defines it *after* the nested
//!   function's definition, so the shared variable is not initialized before
//!   the nested function can be called. Fires on the first offending parent
//!   definition.
//!
//! The checks are deliberately narrow: only clear use-before-definition
//! patterns fire. Branch-partial cases are left to the `unset_variables`
//! engine (NODEF/USENS/PSET).

use super::super::CompatibilityEngine;
use malt_core::{Diagnostic, Severity};
use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Tree};

use crate::analysis::symbols::{DefKind, Scope, ScopeKind, SymbolTable, VarDef, VarUse};

/// IDISVARLOW: indexing an undefined variable may resolve to a path function.
const IDISVARLOW_MSG: &str = "To avoid a potential conflict with functions on the path, explicitly define the variable before indexing into it.";

/// IDISVARHIGH: a variable must be explicitly defined before its first use.
const IDISVARHIGH_MSG: &str = "Variable must be explicitly defined before first use. In some cases, the definition was not required in previous releases, but it is now required.";

/// SHVAI: shared variables must be initialized before the nested function runs.
const SHVAI_MSG: &str = "Explicitly define shared variables in the parent function before calling the nested function. MATLAB does not share uninitialized variables between a nested function and the parent function.";

/// Names that are always defined by MATLAB and never need a user definition.
const BUILTINS: &[&str] = &[
    "true",
    "false",
    "pi",
    "inf",
    "Inf",
    "nan",
    "NaN",
    "eps",
    "i",
    "j",
    "end",
    "nargin",
    "nargout",
    "varargin",
    "varargout",
    "ans",
];

/// How an identifier use is classified for the unset-variable checks.
///
/// The symbol table records every identifier it treats as a usage, but the
/// checks only fire on some of those positions:
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UseClass {
    /// Callee of a plain `function_call` (`x(i)`, `x{i}`) — ambiguous
    /// indexing/call context (IDISVARLOW's domain).
    CallName,
    /// Object of a plain `field_expression` (`x.field`) — indexing context.
    FieldObject,
    /// Callee name of a dotted call (`pkg.func(x)`): definitely a function or
    /// method call, never variable indexing. Skipped by both checks.
    QualifiedCallee,
    /// Object/qualifier of a dotted call (`dsp` in `dsp.FIRFilter(x)`):
    /// a package or namespace qualifier, not a variable read.
    QualifiedObject,
    /// Any other read (RHS, call argument, operator operand, ...).
    Plain,
}

/// File-level entry point: run all three unset/shared-variable checks.
pub(crate) fn collect_file_checks(
    engine: &CompatibilityEngine,
    tree: &Tree,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let symbol_table = SymbolTable::build(tree, source);
    let use_classes = classify_identifier_uses(tree.root_node());
    let known_functions = collect_known_functions(tree.root_node(), source);

    engine.check_idisvar_low(&symbol_table, &use_classes, &known_functions, diagnostics);
    engine.check_idisvar_high(&symbol_table, &use_classes, &known_functions, diagnostics);
    engine.check_shvai(&symbol_table, diagnostics);
}

impl CompatibilityEngine {
    /// IDISVARLOW: a variable is indexed before it is defined in its scope.
    fn check_idisvar_low(
        &self,
        symbol_table: &SymbolTable,
        use_classes: &HashMap<usize, UseClass>,
        known_functions: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for scope in &symbol_table.scopes {
            for name in scope.used_names() {
                if is_builtin(name) || known_functions.contains(name) {
                    continue;
                }
                // Only a genuine variable can conflict with a path function. A
                // name with no definition in this scope (e.g. `plot(x, y)`) is
                // treated as a function call, not an indexing conflict.
                if !has_variable_def(scope, name) {
                    continue;
                }
                for use_ in uses_in_order(scope, name) {
                    if !matches!(
                        use_classes.get(&use_.byte_range.start),
                        Some(UseClass::CallName | UseClass::FieldObject)
                    ) {
                        continue;
                    }
                    if !has_def_before_or_at(scope, name, use_.byte_range.start) {
                        push_use_diag(
                            "IDISVARLOW",
                            IDISVARLOW_MSG,
                            use_,
                            Severity::Warning,
                            diagnostics,
                        );
                        break;
                    }
                }
            }
        }
    }

    /// IDISVARHIGH: a plain read with no prior definition in scope.
    fn check_idisvar_high(
        &self,
        symbol_table: &SymbolTable,
        use_classes: &HashMap<usize, UseClass>,
        known_functions: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for (scope_idx, scope) in symbol_table.scopes.iter().enumerate() {
            for name in scope.used_names() {
                if is_builtin(name) || known_functions.contains(name) {
                    continue;
                }
                for use_ in uses_in_order(scope, name) {
                    // Indexing/call positions are IDISVARLOW's domain.
                    if !matches!(
                        use_classes.get(&use_.byte_range.start),
                        None | Some(UseClass::Plain)
                    ) {
                        continue;
                    }
                    if !has_def_before_or_at(scope, name, use_.byte_range.start)
                        && !has_def_in_ancestors(symbol_table, scope_idx, name)
                    {
                        push_use_diag(
                            "IDISVARHIGH",
                            IDISVARHIGH_MSG,
                            use_,
                            Severity::Warning,
                            diagnostics,
                        );
                        break;
                    }
                }
            }
        }
    }

    /// SHVAI: a shared variable is only defined after the nested function.
    fn check_shvai(&self, symbol_table: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
        for scope in &symbol_table.scopes {
            if scope.kind != ScopeKind::NestedFunction {
                continue;
            }
            let Some(parent_idx) = scope.parent else {
                continue;
            };
            let parent = &symbol_table.scopes[parent_idx];
            let nested_start = scope.byte_range.start;

            let mut names: HashSet<&str> = HashSet::new();
            names.extend(scope.uses.iter().map(|u| u.name.as_str()));
            names.extend(scope.defs.iter().map(|d| d.name.as_str()));

            for name in names {
                if is_builtin(name) {
                    continue;
                }
                // Only "variable" definitions count — a sibling nested
                // function name is not a shared variable.
                let parent_defs = parent.defs_of(name);
                if parent_defs.is_empty() {
                    continue;
                }
                let variable_defs: Vec<&VarDef> = parent_defs
                    .into_iter()
                    .filter(|d| d.kind != DefKind::NestedFunction)
                    .collect();
                if variable_defs.is_empty() {
                    continue;
                }
                // All definitions must occur after the nested function starts.
                if variable_defs
                    .iter()
                    .all(|d| d.byte_range.start >= nested_start)
                {
                    push_def_diag(
                        "SHVAI",
                        SHVAI_MSG,
                        variable_defs[0],
                        Severity::Warning,
                        diagnostics,
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// True when the name is a MATLAB builtin that needs no user definition.
fn is_builtin(name: &str) -> bool {
    BUILTINS.contains(&name)
}

/// The uses of `name` in `scope`, ordered by byte offset.
fn uses_in_order<'a>(scope: &'a Scope, name: &str) -> Vec<&'a VarUse> {
    let mut uses = scope.uses_of(name);
    uses.sort_by_key(|u| u.byte_range.start);
    uses
}

/// True when `scope` defines `name` as a variable (any definition kind other
/// than a nested-function name).
fn has_variable_def(scope: &Scope, name: &str) -> bool {
    scope
        .defs_of(name)
        .iter()
        .any(|d| d.kind != DefKind::NestedFunction)
}

/// True when `scope` defines `name` at or before the given use offset.
fn has_def_before_or_at(scope: &Scope, name: &str, use_start: usize) -> bool {
    scope
        .defs_of(name)
        .iter()
        .any(|d| d.byte_range.start <= use_start)
}

/// True when any ancestor scope (shared-workspace scopes such as nested
/// functions and lambdas) defines `name`. A nested function may legitimately
/// read a parent's variable, so an ancestor definition suppresses IDISVARHIGH
/// (the shared-variable case is handled by SHVAI).
fn has_def_in_ancestors(symbol_table: &SymbolTable, mut scope_idx: usize, name: &str) -> bool {
    while let Some(parent_idx) = symbol_table.scopes[scope_idx].parent {
        if symbol_table.scopes[parent_idx].is_defined(name) {
            return true;
        }
        scope_idx = parent_idx;
    }
    false
}

/// Classify every identifier in the tree by its use position.
fn classify_identifier_uses(root: Node) -> HashMap<usize, UseClass> {
    let mut classes = HashMap::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "identifier" {
            if let Some(class) = classify_identifier(node) {
                classes.insert(node.start_byte(), class);
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }
    classes
}

/// Classify one identifier based on its parent context.
fn classify_identifier(node: Node) -> Option<UseClass> {
    let parent = node.parent()?;
    match parent.kind() {
        "function_call" => {
            let is_name = parent
                .child_by_field_name("name")
                .map(|n| n.id() == node.id())
                .unwrap_or(false);
            if !is_name {
                return Some(UseClass::Plain);
            }
            // A dotted call (`pkg.func(x)`) has the function_call as the field
            // child of a field_expression: the callee is definitely a function.
            match parent.parent().map(|g| g.kind()) {
                Some("field_expression") => Some(UseClass::QualifiedCallee),
                _ => Some(UseClass::CallName),
            }
        }
        "field_expression" => {
            let is_field = parent
                .child_by_field_name("field")
                .map(|f| f.id() == node.id())
                .unwrap_or(false);
            if is_field {
                // The field name of `x.field` is not a variable read.
                return None;
            }
            // The object/qualifier position. A field whose value is a call
            // (`dsp.FIRFilter(x)`, `x.field(1)`) is a qualified call, so the
            // object identifiers are namespace qualifiers, not variable reads.
            let has_call_field = named_children(parent)
                .iter()
                .any(|c| c.kind() == "function_call");
            if has_call_field {
                Some(UseClass::QualifiedObject)
            } else {
                Some(UseClass::FieldObject)
            }
        }
        _ => Some(UseClass::Plain),
    }
}

/// The named children of a node.
fn named_children(node: Node) -> Vec<Node> {
    let mut children = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        children.push(child);
    }
    children
}

/// Names of every function and class defined in the file.
fn collect_known_functions(root: Node, source: &str) -> HashSet<String> {
    let mut known = HashSet::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if matches!(node.kind(), "function_definition" | "class_definition") {
            if let Some(name) = node.child_by_field_name("name") {
                known.insert(source[name.start_byte()..name.end_byte()].to_string());
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }
    known
}

/// Push a diagnostic located at a variable use.
fn push_use_diag(
    id: &'static str,
    message: &str,
    use_: &VarUse,
    severity: Severity,
    diagnostics: &mut Vec<Diagnostic>,
) {
    diagnostics.push(Diagnostic {
        rule_id: id,
        message: message.to_string(),
        severity,
        byte_range: use_.byte_range.clone(),
        line: use_.line,
        column: use_.column,
        fix: None,
    });
}

/// Push a diagnostic located at a variable definition.
fn push_def_diag(
    id: &'static str,
    message: &str,
    def: &VarDef,
    severity: Severity,
    diagnostics: &mut Vec<Diagnostic>,
) {
    diagnostics.push(Diagnostic {
        rule_id: id,
        message: message.to_string(),
        severity,
        byte_range: def.byte_range.clone(),
        line: def.line,
        column: def.column,
        fix: None,
    });
}

#[cfg(test)]
mod tests {
    use crate::compatibility::CompatibilityEngine;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn malt_core::Rule> {
        CompatibilityEngine::from_config(&Config::default())
    }

    // -- IDISVARLOW: indexing before definition -----------------------------

    #[test]
    fn idisvarlow_indexed_before_later_assignment_fires() {
        let d = lint_file(&*engine(), "function f()\n y = A(1);\n A = [1 2 3];\nend\n");
        assert!(has_id(&d, "IDISVARLOW"), "got: {d:?}");
    }

    #[test]
    fn idisvarlow_cell_indexing_before_assignment_fires() {
        let d = lint_file(&*engine(), "function f()\n y = A{1};\n A = {1, 2};\nend\n");
        assert!(has_id(&d, "IDISVARLOW"), "got: {d:?}");
    }

    #[test]
    fn idisvarlow_field_access_before_assignment_fires() {
        let d = lint_file(
            &*engine(),
            "function f()\n y = A.field;\n A = struct('field', 1);\nend\n",
        );
        assert!(has_id(&d, "IDISVARLOW"), "got: {d:?}");
    }

    #[test]
    fn idisvarlow_never_defined_not_fires() {
        // A name used only as a callee with no definition anywhere is treated
        // as a function call (like `plot(x, y)`), not an indexing conflict.
        let d = lint_file(&*engine(), "function f()\n y = A(1);\nend\n");
        assert!(!has_id(&d, "IDISVARLOW"), "got: {d:?}");
    }

    #[test]
    fn idisvarlow_input_arg_not_fires() {
        let d = lint_file(&*engine(), "function f(A)\n y = A(1);\nend\n");
        assert!(!has_id(&d, "IDISVARLOW"), "got: {d:?}");
    }

    #[test]
    fn idisvarlow_assigned_before_not_fires() {
        let d = lint_file(&*engine(), "function f()\n A = [1 2 3];\n y = A(1);\nend\n");
        assert!(!has_id(&d, "IDISVARLOW"), "got: {d:?}");
    }

    #[test]
    fn idisvarlow_builtin_not_fires() {
        let d = lint_file(&*engine(), "function f()\n y = ans(1);\nend\n");
        assert!(!has_id(&d, "IDISVARLOW"), "got: {d:?}");
    }

    #[test]
    fn idisvarlow_local_function_call_not_fires() {
        let d = lint_file(
            &*engine(),
            "function f()\n y = helper(1);\nend\nfunction z = helper(a)\n z = a;\nend\n",
        );
        assert!(!has_id(&d, "IDISVARLOW"), "got: {d:?}");
    }

    #[test]
    fn idisvarlow_nested_function_call_not_fires() {
        let d = lint_file(
            &*engine(),
            "function p()\n y = nf(1);\n function z = nf(a)\n  z = a;\n end\nend\n",
        );
        assert!(!has_id(&d, "IDISVARLOW"), "got: {d:?}");
    }

    #[test]
    fn idisvarlow_dotted_call_not_fires() {
        let d = lint_file(&*engine(), "function f()\n y = dsp.FIRFilter(1);\nend\n");
        assert!(!has_id(&d, "IDISVARLOW"), "got: {d:?}");
    }

    // -- IDISVARHIGH: plain read before definition --------------------------

    #[test]
    fn idisvarhigh_plain_read_undefined_fires() {
        let d = lint_file(&*engine(), "function f()\n y = x + 1;\nend\n");
        assert!(has_id(&d, "IDISVARHIGH"), "got: {d:?}");
    }

    #[test]
    fn idisvarhigh_read_before_later_assignment_fires() {
        let d = lint_file(&*engine(), "function f()\n y = x + 1;\n x = 5;\nend\n");
        assert!(has_id(&d, "IDISVARHIGH"), "got: {d:?}");
    }

    #[test]
    fn idisvarhigh_call_argument_undefined_fires() {
        let d = lint_file(&*engine(), "function f()\n disp(x);\nend\n");
        assert!(has_id(&d, "IDISVARHIGH"), "got: {d:?}");
    }

    #[test]
    fn idisvarhigh_defined_before_not_fires() {
        let d = lint_file(&*engine(), "function f()\n x = 5;\n y = x + 1;\nend\n");
        assert!(!has_id(&d, "IDISVARHIGH"), "got: {d:?}");
    }

    #[test]
    fn idisvarhigh_callee_not_fires() {
        let d = lint_file(&*engine(), "function f()\n y = A(1);\nend\n");
        assert!(!has_id(&d, "IDISVARHIGH"), "got: {d:?}");
    }

    #[test]
    fn idisvarhigh_builtin_not_fires() {
        let d = lint_file(&*engine(), "function f()\n y = pi + 1;\nend\n");
        assert!(!has_id(&d, "IDISVARHIGH"), "got: {d:?}");
    }

    #[test]
    fn idisvarhigh_input_arg_not_fires() {
        let d = lint_file(&*engine(), "function f(x)\n y = x + 1;\nend\n");
        assert!(!has_id(&d, "IDISVARHIGH"), "got: {d:?}");
    }

    #[test]
    fn idisvarhigh_loop_iterator_not_fires() {
        let d = lint_file(
            &*engine(),
            "function f()\n for i = 1:10\n  y = i + 1;\n end\nend\n",
        );
        assert!(!has_id(&d, "IDISVARHIGH"), "got: {d:?}");
    }

    #[test]
    fn idisvarhigh_shared_with_parent_not_fires() {
        // `x` is defined in the parent function, so the nested read is legal.
        let d = lint_file(
            &*engine(),
            "function p()\n x = 5;\n function n()\n  y = x + 1;\n end\n n();\nend\n",
        );
        assert!(!has_id(&d, "IDISVARHIGH"), "got: {d:?}");
    }

    // -- SHVAI: shared variable defined after the nested function -----------

    #[test]
    fn shvai_shared_var_defined_after_nested_fires() {
        let d = lint_file(
            &*engine(),
            "function p()\n function n()\n  y = x + 1;\n end\n x = 5;\n n();\nend\n",
        );
        assert!(has_id(&d, "SHVAI"), "got: {d:?}");
    }

    #[test]
    fn shvai_shared_var_defined_before_nested_not_fires() {
        let d = lint_file(
            &*engine(),
            "function p()\n x = 5;\n function n()\n  y = x + 1;\n end\n n();\nend\n",
        );
        assert!(!has_id(&d, "SHVAI"), "got: {d:?}");
    }

    #[test]
    fn shvai_no_shared_var_not_fires() {
        let d = lint_file(
            &*engine(),
            "function p()\n function n()\n  y = 1;\n end\n x = 5;\n n();\nend\n",
        );
        assert!(!has_id(&d, "SHVAI"), "got: {d:?}");
    }

    #[test]
    fn shvai_local_only_nested_not_fires() {
        // The nested function defines its own variable; the parent never
        // references it, so there is nothing shared.
        let d = lint_file(
            &*engine(),
            "function p()\n x = 5;\n function n()\n  x = 1;\n  y = x + 1;\n end\n n();\nend\n",
        );
        assert!(!has_id(&d, "SHVAI"), "got: {d:?}");
    }

    #[test]
    fn shvai_sibling_function_call_not_fires() {
        // `other` is a sibling nested function, not a shared variable.
        let d = lint_file(
            &*engine(),
            "function p()\n function n()\n  other();\n end\n function other()\n end\n n();\nend\n",
        );
        assert!(!has_id(&d, "SHVAI"), "got: {d:?}");
    }
}
