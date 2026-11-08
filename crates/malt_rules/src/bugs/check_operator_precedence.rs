use super::*;
use std::collections::HashSet;

impl BugsEngine {
    /// MOCUP: a variable is cleared (via `clear`/`clearvars`) but an `onCleanup`
    /// object whose cleanup function references that variable is still live, so
    /// the cleanup function will hit an undefined variable when it runs.
    ///
    /// # Limitations (heuristic)
    ///
    /// This is a whole-file analysis without lexical scope resolution: cleanup
    /// references and `clear` commands are matched by variable NAME alone, not
    /// by scope. A variable name that appears in both an `onCleanup` callback
    /// and an unrelated `clear` in a different function/scope can therefore
    /// produce a false positive. There is no static type or data-flow analysis,
    /// so "is still live" is inferred from a name match rather than proved.
    pub(crate) fn check_mocup(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        // First pass: collect variable names referenced by onCleanup cleanup
        // functions.
        let mut cleanup_refs: HashSet<String> = HashSet::new();
        Self::collect_cleanup_refs(root, source, &mut cleanup_refs);

        // Second pass: flag `clear`/`clearvars` of a cleanup-referenced variable.
        let mut diagnostics = Vec::new();
        Self::walk_clear_commands(root, source, &cleanup_refs, &mut diagnostics);
        diagnostics
    }

    fn collect_cleanup_refs(node: Node, source: &str, cleanup_refs: &mut HashSet<String>) {
        if is_oncleanup_call(node, source) {
            let mut identifiers = HashSet::new();
            let mut call_names = HashSet::new();
            collect_identifiers(node, source, &mut identifiers);
            collect_function_call_names(node, source, &mut call_names);
            for id in identifiers.difference(&call_names) {
                cleanup_refs.insert(id.clone());
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_cleanup_refs(child, source, cleanup_refs);
        }
    }

    fn walk_clear_commands(
        node: Node,
        source: &str,
        cleanup_refs: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if let Some(vars) = clear_command_vars(node, source) {
            for v in vars {
                if cleanup_refs.contains(&v) {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "MOCUP",
                        message: "Variable VAR_NAME may be cleared before the cleanup function that references VAR_NAME executes, resulting in an undefined variable error.".to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_clear_commands(child, source, cleanup_refs, diagnostics);
        }
    }
}

/// If `node` clears variables (via `clear`/`clearvars` in command or function
/// form), return the cleared variable names.
fn clear_command_vars(node: Node, source: &str) -> Option<Vec<String>> {
    match node.kind() {
        "command" => {
            let text = node_text(node, source).trim();
            let mut words = text.split_whitespace();
            if words.next() != Some("clear") {
                return None;
            }
            Some(
                words
                    .filter(|w| !w.starts_with('-'))
                    .map(|w| w.to_string())
                    .collect(),
            )
        }
        "function_call" => {
            let name = extract_call_name(node, source)?;
            if name != "clear" && name != "clearvars" {
                return None;
            }
            let vars = collect_call_args(node, source)
                .iter()
                .map(|a| a.trim().trim_matches(|c| c == '\'' || c == '"').to_string())
                .filter(|a| !a.is_empty() && !a.starts_with('-'))
                .collect();
            Some(vars)
        }
        _ => None,
    }
}

/// Whether `node` is an `onCleanup(...)` call.
fn is_oncleanup_call(node: Node, source: &str) -> bool {
    node.kind() == "function_call" && extract_call_name(node, source) == Some("onCleanup")
}

/// Collect all identifier names in the subtree.
fn collect_identifiers(node: Node, source: &str, out: &mut HashSet<String>) {
    if node.kind() == "identifier" {
        out.insert(node_text(node, source).trim().to_string());
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_identifiers(child, source, out);
    }
}

/// Collect all function-call name identifiers in the subtree.
fn collect_function_call_names(node: Node, source: &str, out: &mut HashSet<String>) {
    if node.kind() == "function_call" {
        if let Some(name) = node.child_by_field_name("name") {
            out.insert(node_text(name, source).trim().to_string());
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_function_call_names(child, source, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    // -- MOCUP ---------------------------------------------------------------

    #[test]
    fn mocup_fires_on_clear_of_cleanup_referenced_var() {
        let src = "\
clear X
c = onCleanup(@() disp(X));
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "MOCUP"), "got: {diags:?}");
    }

    #[test]
    fn mocup_fires_on_clear_function_form() {
        let src = "\
clear('x')
c = onCleanup(@() cleanupFn(x));
";
        let diags = file_diags(src);
        assert!(has_id(&diags, "MOCUP"), "got: {diags:?}");
    }

    #[test]
    fn mocup_no_fire_when_cleanup_var_not_cleared() {
        let src = "\
c = onCleanup(@() disp(x));
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "MOCUP"), "got: {diags:?}");
    }

    #[test]
    fn mocup_no_fire_when_cleared_var_not_referenced() {
        let src = "\
clear x
x = 1;
";
        let diags = file_diags(src);
        assert!(!has_id(&diags, "MOCUP"), "got: {diags:?}");
    }
}
