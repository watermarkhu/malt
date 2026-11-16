//! INUSA/INUSD checks: input arguments that are never used.
//!
//! An input in the function signature that is declared in the function's
//! `arguments` block is INUSA's: it is reported at its declaration in the block.
//! An input that the block does not declare is INUSD's.

use super::*;
use crate::analysis::liveness::shared_dead_stores;
use malt_core::Fix;
use std::collections::HashMap;
use std::ops::Range;
use tree_sitter::Tree;

/// A name declared in an `arguments` block: where it is declared.
pub(crate) struct BlockDecl {
    pub(crate) name: String,
    byte_range: Range<usize>,
    line: usize,
    column: usize,
}

/// The `arguments` blocks of one file.
#[derive(Default)]
pub(crate) struct ArgumentsBlocks {
    /// Declarations keyed by the byte range of the function they belong to.
    pub(crate) by_function: HashMap<(usize, usize), Vec<BlockDecl>>,
    /// Byte range of every `arguments` block. A name that appears inside one
    /// (a validator or a default value) is not a use of the input.
    ranges: Vec<Range<usize>>,
}

impl UnusedEngine {
    /// Run INUSA/INUSD checks: input arguments that are never used.
    pub(crate) fn check_unused_inputs(
        &self,
        table: &SymbolTable,
        tree: &Tree,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let inusa_on = !self.is_check_disabled("INUSA");
        let inusd_on = !self.is_check_disabled("INUSD");
        if !inusa_on && !inusd_on {
            return;
        }
        let manu_enabled = !self.is_check_disabled("MANU");
        let blocks = collect_arguments_blocks(tree.root_node(), source);

        for (index, scope) in table.scopes.iter().enumerate() {
            if !matches!(
                scope.kind,
                ScopeKind::Function
                    | ScopeKind::LocalFunction
                    | ScopeKind::NestedFunction
                    | ScopeKind::Method
            ) {
                continue;
            }
            let declared = blocks
                .by_function
                .get(&(scope.byte_range.start, scope.byte_range.end));
            // The names read with their entry value. MATLAB reports an input whose
            // entry value is overwritten before any read, or read only after
            // `error(...)`, as unused. Where the analysis cannot follow the scope,
            // only a name with no use at all is unused.
            let entry_reads = shared_dead_stores(table, index, tree, source).map(|d| d.entry);

            for def in &scope.defs {
                if def.kind != DefKind::InputArg {
                    continue;
                }
                let name = &def.name;
                // VANUS owns varargin.
                if name == "varargin" || self.should_ignore_name(name) {
                    continue;
                }

                // MATLAB reports an input declared in the arguments block as
                // INUSA, at its declaration. Name-value arguments, which are
                // not in the signature, are not reported.
                if let Some(decl) = declared.and_then(|d| d.iter().find(|d| d.name == *name)) {
                    let unread = !is_read_in(table, index, name, &blocks)
                        || entry_reads
                            .as_ref()
                            .is_some_and(|reads| !reads.contains(name));
                    if inusa_on && unread {
                        diagnostics.push(Diagnostic {
                            rule_id: "INUSA",
                            message: "Input argument might be unused after the function arguments block(s)."
                                .to_string(),
                            severity: Severity::Warning,
                            byte_range: decl.byte_range.clone(),
                            line: decl.line,
                            column: decl.column,
                            fix: None,
                        });
                    }
                    continue;
                }

                if !inusd_on {
                    continue;
                }
                // MANU reports a method's only input when it is unused; with
                // more inputs MATLAB reports each unused one as INUSD.
                if scope.kind == ScopeKind::Method
                    && manu_enabled
                    && super::check_manu::manu_method(scope, tree.root_node(), source)
                {
                    continue;
                }
                // A read in a nested function counts (probed: a parent argument read by
                // an inner function is not INUSD), though the entry-value analysis of
                // this scope does not follow it.
                let read_in_nested = scope
                    .children
                    .iter()
                    .any(|&child| is_read_in(table, child, name, &blocks));
                let unread = !read_in_nested
                    && (!scope.is_used(name)
                        || entry_reads
                            .as_ref()
                            .is_some_and(|reads| !reads.contains(name)));
                if unread && !super::check_manu::returned_input(scope, name) {
                    diagnostics.push(Diagnostic {
                        rule_id: "INUSD",
                        message: "Input argument might be unused. Consider replacing the argument with ~ instead.".to_string(),
                        severity: Severity::Warning,
                        byte_range: def.byte_range.clone(),
                        line: def.line,
                        column: def.column,
                        // Unsafe: eval/evalin/inputname can still see the name.
                        fix: Some(Fix::new(def.byte_range.clone(), "~").unsafe_fix()),
                    });
                }
            }
        }
    }
}

/// Collect the names declared in every `arguments` block of the file.
pub(crate) fn collect_arguments_blocks(root: Node, source: &str) -> ArgumentsBlocks {
    let mut blocks = ArgumentsBlocks::default();
    collect_arguments_dfs(root, source, &mut blocks);
    blocks
}

fn collect_arguments_dfs(node: Node, source: &str, blocks: &mut ArgumentsBlocks) {
    if node.kind() == "arguments_statement" {
        blocks.ranges.push(node.start_byte()..node.end_byte());
        if let Some(function) = enclosing_function(node) {
            let decls = blocks
                .by_function
                .entry((function.start_byte(), function.end_byte()))
                .or_default();
            let mut cursor = node.walk();
            let mut declarations: Vec<Node> = Vec::new();
            for child in node.children(&mut cursor) {
                // `opts.?x = 1` does not parse: the declaration is inside an error.
                if child.kind() == "ERROR" {
                    let mut inner = child.walk();
                    declarations.extend(
                        child
                            .children(&mut inner)
                            .filter(|c| c.kind() == "class_property"),
                    );
                } else {
                    declarations.push(child);
                }
            }
            for child in declarations {
                // `opts.?double` and `opts.?matlab.graphics...` parse as
                // `class_property`, whose first identifier is the declared name.
                let name_node = match child.kind() {
                    "property" => child.child_by_field_name("name"),
                    "class_property" => child.named_child(0),
                    _ => None,
                };
                let Some(name_node) = name_node else {
                    continue;
                };
                // `opts.Mode` declares `opts`.
                let text = &source[name_node.start_byte()..name_node.end_byte()];
                let name = text.split('.').next().unwrap_or("");
                // Keep the first declaration of each name; skip `~`.
                if name.is_empty() || name.starts_with('~') || decls.iter().any(|d| d.name == name)
                {
                    continue;
                }
                let pos = child.start_position();
                decls.push(BlockDecl {
                    name: name.to_string(),
                    byte_range: child.start_byte()..child.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                });
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_arguments_dfs(child, source, blocks);
    }
}

/// The nearest enclosing `function_definition` of `node`.
fn enclosing_function(node: Node) -> Option<Node> {
    let mut current = node.parent();
    while let Some(n) = current {
        if n.kind() == "function_definition" {
            return Some(n);
        }
        current = n.parent();
    }
    None
}

/// `true` if `name` is read in the scope at `index` or in a scope nested in it,
/// outside every `arguments` block. Assignments are not reads.
fn is_read_in(table: &SymbolTable, index: usize, name: &str, blocks: &ArgumentsBlocks) -> bool {
    let scope = &table.scopes[index];
    scope.uses.iter().any(|u| {
        u.name == name
            && !blocks
                .ranges
                .iter()
                .any(|r| r.contains(&u.byte_range.start))
    }) || scope
        .children
        .iter()
        .any(|&child| is_read_in(table, child, name, blocks))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    fn lines_of(diags: &[Diagnostic], id: &str) -> Vec<usize> {
        diags
            .iter()
            .filter(|d| d.rule_id == id)
            .map(|d| d.line)
            .collect()
    }

    // -- INUSA / INUSD: unused input arguments ----------------------

    #[test]
    fn inusd_silent_when_a_nested_function_reads_the_parent_argument() {
        // Probed (zn_2, zn_3, zn_5, zr_nf7): MATLAB reports nothing for `a`; with no
        // read anywhere, `a` is INUSD (zn_7).
        let read =
            "function outer(a)\n    function inner()\n        disp(a)\n    end\ninner();\nend\n";
        assert!(!lines_of(&lint_file(&*engine(), read), "INUSD").contains(&1));
        let unread =
            "function outer(a)\n    function inner()\n        disp(1)\n    end\ninner();\nend\n";
        assert!(lines_of(&lint_file(&*engine(), unread), "INUSD").contains(&1));
    }

    #[test]
    fn signature_inputs_are_inusd_not_inusa() {
        let diags = lint_file(&*engine(), "function foo(x)\nend\n");
        assert!(!has_id(&diags, "INUSA"), "got: {diags:?}");
        assert!(has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn inusa_ok_when_input_used() {
        let diags = lint_file(&*engine(), "function foo(x)\n    disp(x);\nend\n");
        assert!(!has_id(&diags, "INUSA"), "got: {diags:?}");
    }

    #[test]
    fn inusd_fires_on_unused_input() {
        let diags = lint_file(&*engine(), "function foo(x)\nend\n");
        assert!(has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn inusd_ok_when_input_used() {
        let diags = lint_file(&*engine(), "function foo(x)\n    disp(x);\nend\n");
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn inusa_fires_on_unused_block_input() {
        let diags = lint_file(
            &*engine(),
            "function r = foo(a, b)\n    arguments\n        a (1,1) double\n        b (1,1) double\n    end\n    r = a;\nend\n",
        );
        assert_eq!(lines_of(&diags, "INUSA"), vec![4], "got: {diags:?}");
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn inusa_fires_when_input_only_in_validator() {
        // Probe: a name inside another validator is not a use.
        let diags = lint_file(
            &*engine(),
            "function r = foo(a, b)\n    arguments\n        a (1,1) double\n        b (1,1) double {mustBeLessThan(b, 10)}\n    end\n    r = a;\nend\n",
        );
        assert_eq!(lines_of(&diags, "INUSA"), vec![4], "got: {diags:?}");
    }

    #[test]
    fn inusa_fires_when_input_only_assigned() {
        // Probe: assigning to an input is not a read of it.
        let diags = lint_file(
            &*engine(),
            "function r = foo(a, b)\n    arguments\n        a (1,1) double\n        b (1,1) double\n    end\n    b = 2;\n    r = a;\nend\n",
        );
        assert_eq!(lines_of(&diags, "INUSA"), vec![4], "got: {diags:?}");
    }

    #[test]
    fn inusa_fires_when_input_only_used_in_default() {
        // Probe: a default value of another argument is not a use of the input.
        let diags = lint_file(
            &*engine(),
            "function r = foo(a, b)\n    arguments\n        a (1,1) double = 1\n        b (1,1) double = a\n    end\n    r = b;\nend\n",
        );
        assert_eq!(lines_of(&diags, "INUSA"), vec![3], "got: {diags:?}");
    }

    #[test]
    fn inusa_fires_when_input_only_passed_to_eval() {
        // Probe: MATLAB does not look inside eval strings.
        let diags = lint_file(
            &*engine(),
            "function r = foo(a)\n    arguments\n        a (1,1) double\n    end\n    eval('disp(a)');\n    r = 1;\nend\n",
        );
        assert_eq!(lines_of(&diags, "INUSA"), vec![3], "got: {diags:?}");
    }

    #[test]
    fn inusa_fires_on_unused_struct_input_in_signature() {
        // Probe: `opts` is a positional input, so its block declaration is reported.
        let diags = lint_file(
            &*engine(),
            "function r = foo(a, opts)\n    arguments\n        a (1,1) double\n        opts.Mode (1,1) string = \"x\"\n    end\n    r = a;\nend\n",
        );
        assert_eq!(lines_of(&diags, "INUSA"), vec![4], "got: {diags:?}");
    }

    #[test]
    fn inusa_ignores_name_value_arguments() {
        // Probe: name-value arguments (not in the signature) are not reported.
        let diags = lint_file(
            &*engine(),
            "function r = foo(a)\n    arguments\n        a (1,1) double\n        opts.A (1,1) double\n        opts.B (1,1) double\n    end\n    r = a;\nend\n",
        );
        assert!(!has_id(&diags, "INUSA"), "got: {diags:?}");
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn inusa_ok_when_block_input_used() {
        let diags = lint_file(
            &*engine(),
            "function r = foo(a)\n    arguments\n        a (1,1) double\n    end\n    r = a;\nend\n",
        );
        assert!(!has_id(&diags, "INUSA"), "got: {diags:?}");
    }

    #[test]
    fn inusa_ignores_repeating_varargin() {
        let diags = lint_file(
            &*engine(),
            "function r = foo(a, varargin)\n    arguments\n        a (1,1) double\n    end\n    arguments (Repeating)\n        varargin\n    end\n    r = a;\nend\n",
        );
        assert!(!has_id(&diags, "INUSA"), "got: {diags:?}");
    }

    #[test]
    fn inusa_fires_on_method_block_inputs() {
        // Probe: MATLAB reports the object argument of a method as well.
        let diags = lint_file(
            &*engine(),
            "classdef foo\n    methods\n        function r = m(obj, a)\n            arguments\n                obj\n                a (1,1) double\n            end\n            r = 1;\n        end\n    end\nend\n",
        );
        assert_eq!(lines_of(&diags, "INUSA"), vec![5, 6], "got: {diags:?}");
    }

    #[test]
    fn inusd_fires_for_signature_input_outside_block() {
        // Probe: `b` is in the signature but not in the arguments block: INUSD.
        let diags = lint_file(
            &*engine(),
            "function r = foo(a, b)\n    arguments\n        a (1,1) double\n    end\n    r = a;\nend\n",
        );
        assert!(has_id(&diags, "INUSD"), "got: {diags:?}");
        assert!(!has_id(&diags, "INUSA"), "got: {diags:?}");
    }

    #[test]
    fn inusd_not_reported_for_block_input() {
        // The block declares `a`, so an unused `a` is INUSA's, not INUSD's.
        let diags = lint_file(
            &*engine(),
            "function r = foo(a)\n    arguments\n        a (1,1) double\n    end\nend\n",
        );
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
        assert_eq!(lines_of(&diags, "INUSA"), vec![3], "got: {diags:?}");
    }
    #[test]
    fn inusa_fires_on_unused_dotted_metaclass_declaration() {
        // Probe (t__fv3): `opts.?matlab.graphics...` parses as a class property.
        let diags = lint_file(
            &*engine(),
            "function r = foo(n, opts)\narguments (Input)\n    n\n    opts.?matlab.graphics.chart.primitive.Line\nend\nr = n;\nend\n",
        );
        assert!(has_id(&diags, "INUSA"), "got: {diags:?}");
    }

    #[test]
    fn inusa_fires_on_unused_metaclass_declaration_with_default() {
        // Probe (t__fv2): a default value makes the declaration an error node.
        let diags = lint_file(
            &*engine(),
            "function r = foo(n, opts)\narguments (Input)\n    n\n    opts.?matlab.graphics.chart.primitive.Line = 1\nend\nr = n;\nend\n",
        );
        assert!(has_id(&diags, "INUSA"), "got: {diags:?}");
    }

    #[test]
    fn inusa_fires_when_block_input_is_overwritten_before_read() {
        // Probed (ia03): the body overwrites the declared input before any read.
        let diags = lint_file(
            &*engine(),
            "function r = foo(a)\narguments\n    a double\nend\na = 2;\nr = a;\nend\n",
        );
        assert_eq!(lines_of(&diags, "INUSA"), vec![3], "got: {diags:?}");
        // Probed (ib02): `a = a + 1` reads the entry value first.
        let diags = lint_file(
            &*engine(),
            "function r = foo(a)\narguments\n    a double\nend\na = a + 1;\nr = a;\nend\n",
        );
        assert!(!has_id(&diags, "INUSA"), "got: {diags:?}");
    }

    #[test]
    fn inusd_fires_when_entry_value_is_overwritten_before_read() {
        // Probe (gp-b__t1): `x = 1` overwrites the input before any read of it.
        let diags = lint_file(
            &*engine(),
            "function r = foo(x)\n    x = 1;\n    r = x;\nend\n",
        );
        assert_eq!(lines_of(&diags, "INUSD"), vec![1], "got: {diags:?}");
    }

    #[test]
    fn inusd_fires_when_input_read_only_after_error() {
        // Probe (s5): a read after `error(...)` is unreachable, so the input is unused.
        let diags = lint_file(
            &*engine(),
            "function foo(x)\n    error('a');\n    disp(x);\nend\n",
        );
        assert_eq!(lines_of(&diags, "INUSD"), vec![1], "got: {diags:?}");
    }

    #[test]
    fn inusd_ok_when_entry_value_read_before_overwrite() {
        // Probe: `x = x + 1` reads the entry value before it is overwritten.
        let diags = lint_file(
            &*engine(),
            "function r = foo(x)\n    x = x + 1;\n    r = x;\nend\n",
        );
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn inusd_counts_tilde_placeholder_and_skips_input_that_is_also_output() {
        // Probes U_a and U_b: `k(obj, ~)` leaves `obj` unused (INUSD on line 1), while
        // `function obj = setv(obj)` writes a field of its input, so it is used.
        let diags = lint_file(&*engine(), "function r = k(obj, ~)\nr = 1;\nend\n");
        assert_eq!(lines_of(&diags, "INUSD"), vec![1], "got: {diags:?}");
        let diags = lint_file(&*engine(), "function obj = setv(obj)\nobj.x = 1;\nend\n");
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
    }
}
