use super::*;
use crate::analysis::symbols::{DefKind, SymbolTable};
use std::collections::HashMap;

impl GoodPracticesEngine {
    /// STRNU: a structure variable is changed but the value might be unused.
    ///
    /// Flags assignments that modify a structure (`s.field = ...` or
    /// `s = struct(...)`) where the modified structure is never subsequently
    /// read. File-level because it needs whole-scope use information.
    ///
    /// # Limitations (heuristic)
    ///
    /// "Apparently a structure" is inferred from two syntactic signals only:
    /// a `field_expression` on the left-hand side, or a `struct(...)` call on
    /// the right-hand side. A variable that receives a structure through any
    /// other expression (e.g. `s = someStruct()` or a passed-in argument) is
    /// not recognized. Usage is resolved per enclosing scope via the symbol
    /// table, so a use in a nested function/lambda is not counted and may
    /// produce a false positive.
    pub(crate) fn check_strnu(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("STRNU") {
            return Vec::new();
        }

        // Collect structure-modifying assignments keyed by variable name.
        let mut assigns: HashMap<String, Vec<(std::ops::Range<usize>, usize)>> = HashMap::new();
        collect_structure_assignments(tree.root_node(), source, &mut assigns);

        if assigns.is_empty() {
            return Vec::new();
        }

        let sym = SymbolTable::build(tree, source);
        let mut diagnostics = Vec::new();

        for (name, ranges) in &assigns {
            for (range, line) in ranges {
                // Locate the scope that contains this assignment.
                let scope = sym
                    .scopes
                    .iter()
                    .find(|s| s.byte_range.start <= range.start && range.end <= s.byte_range.end);
                let Some(scope) = scope else { continue };

                // Output arguments are "used" externally — skip them.
                let is_output = scope
                    .defs
                    .iter()
                    .any(|d| d.kind == DefKind::OutputArg && d.name == *name);
                if is_output {
                    continue;
                }

                // The value is "used" only if read after this assignment.
                let subsequently_used = scope
                    .uses
                    .iter()
                    .any(|u| u.name == *name && u.byte_range.start >= range.end);

                if !subsequently_used {
                    diagnostics.push(Diagnostic {
                        rule_id: "STRNU",
                        message: "This variable, apparently a structure, is changed but the value might be unused."
                            .to_string(),
                        severity: Severity::Warning,
                        byte_range: range.clone(),
                        line: *line,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

/// Collect structure-modifying assignments: `s.field = ...` (field-expression
/// LHS) and `s = struct(...)` (RHS is a `struct(...)` call).
fn collect_structure_assignments(
    node: Node,
    source: &str,
    out: &mut HashMap<String, Vec<(std::ops::Range<usize>, usize)>>,
) {
    if node.kind() == "assignment" {
        if let Some(name) = structure_target_name(node, source) {
            let line = node.start_position().row + 1;
            out.entry(name)
                .or_default()
                .push((node.start_byte()..node.end_byte(), line));
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_structure_assignments(child, source, out);
    }
}

/// Return the structure variable name an assignment modifies, if any.
///
/// Recognizes `s.field = ...` (LHS is a field expression) and
/// `s = struct(...)` (RHS is a `struct(...)` call with an identifier LHS).
fn structure_target_name<'a>(node: Node<'a>, source: &'a str) -> Option<String> {
    let left = node.child_by_field_name("left").or_else(|| node.child(0));
    let right = node.child_by_field_name("right").or_else(|| node.child(2));

    // Case 1: LHS is a field expression `s.field`.
    if let Some(lhs) = left {
        if lhs.kind() == "field_expression" {
            if let Some(base) = lhs.child(0) {
                if base.kind() == "identifier" {
                    return Some(node_text(base, source).trim().to_string());
                }
            }
        }
    }

    // Case 2: RHS is a `struct(...)` call and LHS is a bare identifier.
    if let Some(rhs) = right {
        if rhs.kind() == "function_call" {
            if let Some(name_node) = rhs.child_by_field_name("name").or_else(|| rhs.child(0)) {
                if node_text(name_node, source).trim() == "struct" {
                    if let Some(lhs) = left {
                        if lhs.kind() == "identifier" {
                            return Some(node_text(lhs, source).trim().to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strnu_fires_on_unused_field_assignment() {
        let source = "function f()\ns = struct('a', 1);\ns.b = 2;\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_strnu(&tree, source);
        assert!(diags.iter().any(|d| d.rule_id == "STRNU"), "got: {diags:?}");
    }

    #[test]
    fn strnu_silent_when_structure_is_used() {
        let source = "function f()\ns = struct('a', 1);\ns.b = 2;\ny = s.b;\nend\n";
        let tree = parse(source);
        let eng = engine();
        let diags = eng.check_strnu(&tree, source);
        assert!(
            !diags.iter().any(|d| d.rule_id == "STRNU"),
            "got: {diags:?}"
        );
    }
}
