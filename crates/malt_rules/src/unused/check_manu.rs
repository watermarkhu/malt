//! MANU check: method input argument that is never used.

use super::check_unused_inputs::collect_arguments_blocks;
use super::*;
use crate::analysis::liveness::shared_dead_stores;
use crate::analysis::symbols::Scope;
use tree_sitter::Node;

/// Whether a method has exactly one input argument, the case MANU reports.
pub(crate) fn sole_input(scope: &Scope, source: &str) -> bool {
    // Probed: a `~` placeholder is an input too, so `k(obj, ~)` is INUSD, not MANU.
    let header = source
        .get(scope.byte_range.start..)
        .and_then(|s| s.lines().next())
        .unwrap_or("");
    scope
        .defs
        .iter()
        .filter(|d| d.kind == DefKind::InputArg)
        .count()
        == 1
        && !header.contains('~')
}

/// Whether a methods block's attributes make it `Static`: a bare `Static`,
/// `Static = true` or `Static = 1`. MATLAB reads other values (`0`, `FALSE`,
/// `TRUE`) as not static, and reports their inputs as MANU (probed).
fn methods_block_is_static(block: tree_sitter::Node, source: &str) -> bool {
    let attrs = crate::analysis::metadata::attributes_of(
        block,
        source,
        crate::analysis::metadata::AttrForms::Named,
    );
    crate::analysis::metadata::Attributes(&attrs).is_true("Static")
}

/// Whether MANU, rather than INUSD, reports the sole input of a method. A `Static`
/// method and a constructor (named after its class) are not MANU: MATLAB reports
/// their unused input as INUSD (probed).
pub(crate) fn manu_method(scope: &Scope, root: Node, source: &str) -> bool {
    if !sole_input(scope, source) {
        return false;
    }
    let Some(mut func) =
        root.descendant_for_byte_range(scope.byte_range.start, scope.byte_range.end)
    else {
        return true;
    };
    while func.kind() != "function_definition" {
        match func.parent() {
            Some(parent) => func = parent,
            None => return true,
        }
    }
    let name = func
        .child_by_field_name("name")
        .map(|n| &source[n.byte_range()]);
    let mut current = func.parent();
    while let Some(node) = current {
        match node.kind() {
            "methods" => {
                if methods_block_is_static(node, source) {
                    return false;
                }
            }
            "class_definition" => {
                let class = node
                    .child_by_field_name("name")
                    .map(|n| &source[n.byte_range()]);
                if class.is_some() && class == name {
                    return false;
                }
            }
            _ => {}
        }
        current = node.parent();
    }
    true
}

/// Whether an input is also an output (`function obj = set(obj)`): the value
/// is returned, so MATLAB does not report it (probed).
pub(crate) fn returned_input(scope: &Scope, name: &str) -> bool {
    scope
        .defs
        .iter()
        .any(|d| d.kind == DefKind::OutputArg && d.name == name)
}

impl UnusedEngine {
    /// Run MANU check: an input argument of a method that is never used in the
    /// method body. MATLAB suggests replacing it with `~` or making the method
    /// `Static`.
    ///
    /// # Limitations
    ///
    /// MANU is reported only for a method whose sole input is unused (probes M1,
    /// M5 and ls-c__V4). A method with several inputs reports each unused one
    /// as INUSD instead (probes M2, M3, M4 and M7), and so does a `Static`
    /// method or a constructor (cl03, cl05).
    pub(crate) fn check_manu(
        &self,
        table: &SymbolTable,
        tree: &tree_sitter::Tree,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let root = tree.root_node();
        if self.is_check_disabled("MANU") {
            return;
        }
        let blocks = collect_arguments_blocks(root, source);

        for (index, scope) in table.scopes.iter().enumerate() {
            if scope.kind != ScopeKind::Method || !manu_method(scope, root, source) {
                continue;
            }
            // For an input that is also an output: the entry value must not reach
            // the exit (probed, plan 6.2: `function obj = f(obj)` whose body only
            // calls `error(...)` is MANU).
            let entry_reads = shared_dead_stores(table, index, tree, source).map(|d| d.entry);

            for def in &scope.defs {
                if def.kind != DefKind::InputArg {
                    continue;
                }
                let name = &def.name;

                if self.should_ignore_name(name) {
                    continue;
                }

                // An input declared in the arguments block is INUSA's.
                let declared = blocks
                    .by_function
                    .get(&(scope.byte_range.start, scope.byte_range.end))
                    .is_some_and(|d| d.iter().any(|d| d.name == *name));
                if declared {
                    continue;
                }

                let unread = if returned_input(scope, name) {
                    entry_reads
                        .as_ref()
                        .is_some_and(|reads| !reads.contains(name))
                } else {
                    !scope.is_used(name)
                };
                if unread {
                    diagnostics.push(Diagnostic {
                        rule_id: "MANU",
                        message: "Input argument might be unused. Consider replacing the argument with ~, or make this method Static instead.".to_string(),
                        severity: Severity::Warning,
                        byte_range: def.byte_range.clone(),
                        line: def.line,
                        column: def.column,
                        fix: None,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        UnusedEngine::from_config(&Config::default())
    }

    // -- MANU: unused method input argument ----------------------------------

    #[test]
    fn manu_on_returned_input_when_the_method_only_errors() {
        // Probed (plan 6.2): the value never reaches the exit, so `obj` is unused.
        let errors = "classdef M\n    methods\n        function obj = f(obj)\n            error('X:y', 'no');\n        end\n    end\nend\n";
        assert!(has_id(&lint_file(&*engine(), errors), "MANU"));
        let returns =
            "classdef M\n    methods\n        function obj = f(obj)\n        end\n    end\nend\n";
        assert!(!has_id(&lint_file(&*engine(), returns), "MANU"));
    }

    #[test]
    fn manu_fires_on_unused_sole_method_input() {
        // Probes M1 and M5: the only input of a method, unused.
        let diags = lint_file(
            &*engine(),
            "classdef M1\n    methods\n        function r = m(obj)\n            r = 1;\n        end\n    end\nend\n",
        );
        assert!(has_id(&diags, "MANU"), "got: {diags:?}");
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn manu_silent_with_several_method_inputs_reports_inusd() {
        // Probes M2, M3 and M7: with more than one input, MATLAB reports
        // each unused input as INUSD and no MANU.
        let diags = lint_file(
            &*engine(),
            "classdef M3\n    methods\n        function r = m(obj, a)\n            r = 1;\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "MANU"), "got: {diags:?}");
        let inusd = diags.iter().filter(|d| d.rule_id == "INUSD").count();
        assert_eq!(inusd, 2, "got: {diags:?}");
    }

    #[test]
    fn manu_silent_on_sole_input_declared_in_arguments_block() {
        // Probe t__Cls constructor: a block-declared input is INUSA's, not MANU's.
        let diags = lint_file(
            &*engine(),
            "classdef Cls\n    methods\n        function r = Cls(x)\n            arguments\n                x\n            end\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "MANU"), "got: {diags:?}");
    }

    #[test]
    fn manu_ok_when_method_input_used() {
        let diags = lint_file(
            &*engine(),
            "classdef Foo\n    methods\n        function y = bar(self, x)\n            y = self.v + x;\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "MANU"), "got: {diags:?}");
    }

    #[test]
    fn manu_fires_on_startup_fcn_callback_method() {
        // Probes p52 c04 and c05: an App Designer-style private method named
        // `startupFcn` is not exempt; its unused sole input is MANU like any
        // method's (c05's `helper2`), with or without the
        // `% Callbacks that handle component events` header.
        let src = "classdef Cls\n    methods (Access = private)\n        % Callbacks that handle component events\n        function startupFcn(app)\n        end\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "MANU"), "got: {diags:?}");
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn manu_silent_when_tilde_placeholder_follows_unused_input() {
        // Probe U_e: with a `~` placeholder, the unused `obj` is INUSD (line 3), not MANU.
        let diags = lint_file(
            &*engine(),
            "classdef U_e\n    methods\n        function r = k(obj, ~)\n            r = 1;\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "MANU"), "got: {diags:?}");
        let inusd: Vec<usize> = diags
            .iter()
            .filter(|d| d.rule_id == "INUSD")
            .map(|d| d.line)
            .collect();
        assert_eq!(inusd, vec![3], "got: {diags:?}");
    }

    #[test]
    fn manu_silent_when_input_is_also_an_output() {
        // Probe U_e: `function obj = set(obj)` that writes a field of its input is not unused.
        let diags = lint_file(
            &*engine(),
            "classdef U_e\n    methods\n        function obj = set(obj)\n            obj.x = 1;\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "MANU"), "got: {diags:?}");
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn static_method_and_constructor_inputs_are_inusd() {
        // Probed (cl03, cl05): an unused input of a Static method or of a
        // constructor is INUSD, not MANU.
        let diags = lint_file(
            &*engine(),
            "classdef S_e\n    methods (Static)\n        function r = m(a)\n            r = 1;\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "MANU"), "got: {diags:?}");
        assert!(has_id(&diags, "INUSD"), "got: {diags:?}");
        let diags = lint_file(
            &*engine(),
            "classdef C_e\n    methods\n        function obj = C_e(a)\n            obj.x = 1;\n        end\n    end\nend\n",
        );
        assert!(!has_id(&diags, "MANU"), "got: {diags:?}");
        assert!(has_id(&diags, "INUSD"), "got: {diags:?}");
    }

    #[test]
    fn static_zero_is_not_static_so_the_input_is_manu() {
        // Probed: `Static = 0` is not evaluated as static: MANU, not INUSD.
        let diags = lint_file(
            &*engine(),
            "classdef S_z\n    methods (Static = 0)\n        function r = m(a)\n            r = 1;\n        end\n    end\nend\n",
        );
        assert!(has_id(&diags, "MANU"), "got: {diags:?}");
        assert!(!has_id(&diags, "INUSD"), "got: {diags:?}");
        let diags = lint_file(
            &*engine(),
            "classdef S_t\n    methods (Static = true)\n        function r = m(a)\n            r = 1;\n        end\n    end\nend\n",
        );
        assert!(has_id(&diags, "INUSD"), "got: {diags:?}");
        assert!(!has_id(&diags, "MANU"), "got: {diags:?}");
    }
}
