//! INUSA/INUSD checks: input arguments that are never used.
//!
//! An input in the function signature that is declared in the function's
//! `arguments` block is INUSA's: it is reported at its declaration in the block.
//! An input that the block does not declare is INUSD's.

use super::*;
use malt_core::Fix;
use std::collections::HashMap;
use std::ops::Range;

/// A name declared in an `arguments` block: where it is declared.
struct BlockDecl {
    name: String,
    byte_range: Range<usize>,
    line: usize,
    column: usize,
}

/// The `arguments` blocks of one file.
#[derive(Default)]
struct ArgumentsBlocks {
    /// Declarations keyed by the byte range of the function they belong to.
    by_function: HashMap<(usize, usize), Vec<BlockDecl>>,
    /// Byte range of every `arguments` block. A name that appears inside one
    /// (a validator or a default value) is not a use of the input.
    ranges: Vec<Range<usize>>,
}

impl UnusedEngine {
    /// Run INUSA/INUSD checks: input arguments that are never used.
    pub(crate) fn check_unused_inputs(
        &self,
        table: &SymbolTable,
        root: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let inusa_on = !self.is_check_disabled("INUSA");
        let inusd_on = !self.is_check_disabled("INUSD");
        if !inusa_on && !inusd_on {
            return;
        }
        let manu_enabled = !self.is_check_disabled("MANU");
        let blocks = collect_arguments_blocks(root, source);

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
                    if inusa_on && !is_read_in(table, index, name, &blocks) {
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
                // MANU reports unused method inputs, with advice that fits methods.
                if scope.kind == ScopeKind::Method && manu_enabled {
                    continue;
                }
                if !scope.is_used(name) {
                    diagnostics.push(Diagnostic {
                        rule_id: "INUSD",
                        message: format!(
                            "Input argument '{name}' might be unused. Consider replacing the argument with ~ instead."
                        ),
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
fn collect_arguments_blocks(root: Node, source: &str) -> ArgumentsBlocks {
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
            for child in node.children(&mut cursor) {
                if child.kind() != "property" {
                    continue;
                }
                let Some(name_node) = child.child_by_field_name("name") else {
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
}
