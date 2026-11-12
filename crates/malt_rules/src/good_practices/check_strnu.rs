use super::*;
use crate::analysis::liveness::{dead_stores, is_field_store};
use crate::analysis::symbols::{DefKind, SymbolTable};

impl GoodPracticesEngine {
    /// STRNU: a structure variable is changed but the value might be unused.
    ///
    /// Reports a field store (`s.field = ...`, `s(2).f = ...`, `s.f(3) = ...`)
    /// after which the variable is never read: a later whole assignment or
    /// `clear` replaces it, or the function ends. A partial store reads the old
    /// value, so `s.a = 1; s.a = 2; disp(s)` reports nothing. The analysis is
    /// [`crate::analysis::liveness`], as for NASGU.
    ///
    /// Input arguments, globals and persistents are not reported, since a modified
    /// input may be a handle object (probed: MATLAB is silent for `function f(s)`
    /// with `s.a = 1`). Scopes that call `eval`, `load` and the like are not
    /// analysed. A struct built by `struct(...)` that is never read is NASGU, not
    /// STRNU (probed).
    pub(crate) fn check_strnu(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("STRNU") {
            return Vec::new();
        }

        let table = SymbolTable::build(tree, source);
        let mut diagnostics = Vec::new();

        for index in 0..table.scopes.len() {
            let Some(dead) = dead_stores(&table, index, tree, source) else {
                continue;
            };
            let scope = &table.scopes[index];
            for def_index in dead.partial {
                let def = &scope.defs[def_index];
                let is_argument_or_shared = scope.defs.iter().any(|d| {
                    matches!(
                        d.kind,
                        DefKind::InputArg | DefKind::Global | DefKind::Persistent
                    ) && d.name == def.name
                });
                if is_argument_or_shared
                    || !is_field_store(tree, def.byte_range.clone())
                    || stores_through_brace(tree, def.byte_range.clone(), source)
                {
                    continue;
                }
                // Probed: when the latest whole-variable definition before the store
                // is not `struct(...)` (a function call, `[]`, `containers.Map`, a
                // number, `get_handle`), MATLAB is silent: it may be a handle object.
                // With no earlier definition, or one built by `struct(...)`, it reports.
                let defined_otherwise = last_plain_assignment_before(
                    tree.root_node(),
                    &def.name,
                    scope.byte_range.start,
                    def.byte_range.start,
                    source,
                )
                .and_then(|assign| assign.child_by_field_name("right"))
                .is_some_and(|rhs| {
                    rhs.kind() != "function_call"
                        || get_function_call_name(rhs, source) != Some("struct")
                });
                if defined_otherwise {
                    continue;
                }
                diagnostics.push(Diagnostic {
                    rule_id: "STRNU",
                    message: "This variable, apparently a structure, is changed but the value might be unused."
                        .to_string(),
                    severity: Severity::Warning,
                    byte_range: def.byte_range.clone(),
                    line: def.line,
                    column: def.column,
                    fix: None,
                });
            }

            // Probed: in a loop, a field store is unused when the variable is read
            // nowhere else, although the next iteration reads the old value through
            // the store. Only stores of one shape (`x(i).f`) qualify: `x(i).f` with
            // `x(i).g` is silent.
            for def in &scope.defs {
                if diagnostics
                    .iter()
                    .any(|d: &Diagnostic| d.rule_id == "STRNU" && d.byte_range == def.byte_range)
                    || !in_loop(tree, def.byte_range.clone())
                    || scope.defs.iter().any(|d| {
                        matches!(
                            d.kind,
                            DefKind::InputArg | DefKind::Global | DefKind::Persistent
                        ) && d.name == def.name
                    })
                    || !is_field_store(tree, def.byte_range.clone())
                    || stores_through_brace(tree, def.byte_range.clone(), source)
                    || !only_same_stores(tree, source, scope, def)
                {
                    continue;
                }
                let defined_otherwise = last_plain_assignment_before(
                    tree.root_node(),
                    &def.name,
                    scope.byte_range.start,
                    def.byte_range.start,
                    source,
                )
                .and_then(|assign| assign.child_by_field_name("right"))
                .is_some_and(|rhs| {
                    rhs.kind() != "function_call"
                        || get_function_call_name(rhs, source) != Some("struct")
                });
                if defined_otherwise {
                    continue;
                }
                diagnostics.push(Diagnostic {
                    rule_id: "STRNU",
                    message: "This variable, apparently a structure, is changed but the value might be unused."
                        .to_string(),
                    severity: Severity::Warning,
                    byte_range: def.byte_range.clone(),
                    line: def.line,
                    column: def.column,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

/// Whether the store at `range` is inside a `for` or `while` loop of its function.
fn in_loop(tree: &tree_sitter::Tree, range: std::ops::Range<usize>) -> bool {
    let Some(mut node) = tree
        .root_node()
        .descendant_for_byte_range(range.start, range.end)
    else {
        return false;
    };
    while let Some(parent) = node.parent() {
        if matches!(parent.kind(), "for_statement" | "while_statement") {
            return true;
        }
        if parent.kind() == "function_definition" {
            return false;
        }
        node = parent;
    }
    false
}

/// Whether the store at `range` writes through braces (`c{2}.f = v`). Probed:
/// MATLAB is silent on those.
fn stores_through_brace(
    tree: &tree_sitter::Tree,
    range: std::ops::Range<usize>,
    source: &str,
) -> bool {
    enclosing_lhs(tree, range).is_some_and(|lhs| source[lhs.byte_range()].contains('{'))
}

/// The left-hand side of the assignment that contains the target at `range`,
/// when the target is part of that left-hand side.
fn enclosing_lhs<'t>(
    tree: &'t tree_sitter::Tree,
    range: std::ops::Range<usize>,
) -> Option<Node<'t>> {
    let mut node = tree
        .root_node()
        .descendant_for_byte_range(range.start, range.end)?;
    loop {
        if node.kind() == "assignment" {
            let left = node.child_by_field_name("left")?;
            return (left.start_byte() <= range.start && range.end <= left.end_byte())
                .then_some(left);
        }
        node = node.parent()?;
    }
}

/// Whether every reference to `def`'s variable in its scope is a store with the
/// same left-hand side as `def` (`x(i).f = ...` each time).
fn only_same_stores(
    tree: &tree_sitter::Tree,
    source: &str,
    scope: &crate::analysis::symbols::Scope,
    def: &crate::analysis::symbols::VarDef,
) -> bool {
    let Some(lhs) = enclosing_lhs(tree, def.byte_range.clone()) else {
        return false;
    };
    let shape = &source[lhs.byte_range()];
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        let inside =
            node.start_byte() >= scope.byte_range.start && node.end_byte() <= scope.byte_range.end;
        if inside && node.kind() == "identifier" && source[node.byte_range()] == def.name {
            match enclosing_lhs(tree, node.byte_range()) {
                Some(other) if &source[other.byte_range()] == shape => {}
                _ => return false,
            }
        }
        let mut cursor = node.walk();
        stack.extend(node.children(&mut cursor));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // Probe c4_B t1-t5, t7, t9 (l_registry.m:82): MATLAB is silent for a variable
    // from a function call, `[]`, `containers.Map`, a number or an overwrite by a
    // call; only a `struct(...)` value is reported (probe t6).
    #[test]
    fn strnu_silent_unless_built_by_struct() {
        assert!(strnu("function f()\nreg = l_reg_probe();\nreg.Count = 0;\nend\n").is_empty());
        assert!(strnu("function f()\ns = [];\ns.a = 1;\nend\n").is_empty());
        assert!(strnu("function f()\ns = containers.Map();\ns.Count = 0;\nend\n").is_empty());
        assert!(strnu("function f()\ns = 5;\ns.a = 1;\nend\n").is_empty());
        assert!(
            strnu("function f()\ns = struct('a', 1);\ns = my_fun();\ns.a = 1;\nend\n").is_empty()
        );
    }

    /// The STRNU diagnostics for `source`, as (line, column).
    fn strnu(source: &str) -> Vec<(usize, usize)> {
        let tree = parse(source);
        let diags = engine().check_strnu(&tree, source);
        diags.iter().map(|d| (d.line, d.column)).collect()
    }

    // -- Probes: MATLAB R2026b checkcode -------------------------------------

    #[test]
    fn strnu_fires_on_unused_field_assignment() {
        let source = "function f()\ns = struct('a', 1);\ns.b = 2;\nend\n";
        assert_eq!(strnu(source), vec![(3, 1)]);
    }

    #[test]
    fn strnu_silent_when_structure_is_used() {
        let source = "function f()\ns = struct('a', 1);\ns.b = 2;\ny = s.b;\nend\n";
        assert!(strnu(source).is_empty());
    }

    #[test]
    fn strnu_fires_on_dead_field_store_at_function_end() {
        assert_eq!(strnu("function f()\n    s.a = 1;\nend\n"), vec![(2, 5)]);
    }

    #[test]
    fn strnu_fires_when_overwritten_as_a_whole() {
        // `s.a = 1` is dead: `s = 2` replaces the structure before any read.
        assert_eq!(
            strnu("function f()\n    s.a = 1;\n    s = 2;\n    disp(s)\nend\n"),
            vec![(2, 5)]
        );
        assert_eq!(strnu("s.a = 1;\ns = 2;\ndisp(s)\n"), vec![(1, 1)]);
    }

    #[test]
    fn strnu_not_for_partial_store_that_is_read() {
        // The second store reads `s` (partial), so the first is live; `disp(s)` reads both.
        assert!(strnu("function f()\n    s.a = 1;\n    s.a = 2;\n    disp(s)\nend\n").is_empty());
    }

    #[test]
    fn strnu_fires_on_the_last_of_two_unread_stores() {
        assert_eq!(
            strnu("function f()\n    s.a = 1;\n    s.a = 2;\nend\n"),
            vec![(3, 5)]
        );
    }

    #[test]
    fn strnu_fires_on_indexed_field_store() {
        assert_eq!(strnu("function f()\n    s.a(3) = 1;\nend\n"), vec![(2, 5)]);
    }

    #[test]
    fn strnu_fires_on_multi_output_field_store() {
        // Probed: the field element of `[s.a, b] = deal(...)` is STRNU, `b` is ASGLU.
        assert_eq!(
            strnu("function f()\n    [s.a, b] = deal(1, 2);\nend\n"),
            vec![(2, 6)]
        );
    }

    #[test]
    fn strnu_silent_on_struct_constructor_never_read() {
        // Probed: MATLAB reports this one as NASGU.
        assert!(strnu("function f()\n    s = struct('a', 1);\nend\n").is_empty());
    }

    #[test]
    fn strnu_silent_for_input_argument() {
        // Probed: MATLAB is silent (a modified input may be a handle object).
        assert!(strnu("function f(s)\n    s.a = 1;\nend\n").is_empty());
    }

    #[test]
    fn strnu_silent_for_script_final_value() {
        assert!(strnu("t.x = 1;\n").is_empty());
        assert!(strnu("t.x = 1; t.y = 2; disp(t.x)\n").is_empty());
    }

    #[test]
    fn strnu_silent_for_output_argument() {
        assert!(strnu("function s = f()\n    s.a = 1;\nend\n").is_empty());
    }

    #[test]
    fn strnu_silent_when_loop_reads_the_store_again() {
        // The store is read by the next iteration (a partial store reads `s`).
        assert!(strnu(
            "function f(n)\n    s = struct();\n    for k = 1:n\n        s.a = k;\n    end\nend\n"
        )
        .is_empty());
    }
}
