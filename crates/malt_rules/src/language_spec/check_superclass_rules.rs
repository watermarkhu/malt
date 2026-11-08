//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// Group E: constructor/superclass call validation.
    pub(crate) fn check_superclass_rules(
        &self,
        class: &ClassMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let functions = self.collect_c2_function_definitions(ctx.tree.root_node(), ctx.source);
        for (func_node, mname) in functions {
            let outputs = function_output_names(func_node, ctx.source);
            let first_output = outputs.first().cloned();
            let is_constructor = mname == class.name;
            let calls = self.collect_superclass_calls(func_node, ctx.source);
            let mut constructor_calls_seen = 0usize;
            for (call_node, caller, super_name) in calls {
                if is_constructor && Some(caller.as_str()) == first_output.as_deref() {
                    // Superclass constructor call.
                    if constructor_calls_seen == 0 {
                        constructor_calls_seen += 1;
                        if self.is_check_enabled("MCCBU") {
                            Self::check_super_after_object_use(
                                func_node,
                                call_node,
                                &caller,
                                ctx.source,
                                diagnostics,
                            );
                        }
                    } else if self.is_check_enabled("MCCMC") {
                        let pos = call_node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "MCCMC",
                            message: "Constructor for superclass can only be called once."
                                .to_string(),
                            severity: Severity::Error,
                            byte_range: call_node.start_byte()..call_node.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                    if self.is_check_enabled("MCCBS")
                        && !class.superclasses.iter().any(|s| s == &super_name)
                    {
                        let pos = call_node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "MCCBS",
                            message: format!(
                                "A superclass constructor is being called, but {super_name} is not a declared superclass name."
                            ),
                            severity: Severity::Error,
                            byte_range: call_node.start_byte()..call_node.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                    if self.is_check_enabled("MCSCT")
                        && Self::super_call_is_conditional_or_expression(call_node)
                    {
                        let pos = call_node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "MCSCT",
                            message:
                                "Superclass constructor call must not be conditionalized or be part of another expression."
                                    .to_string(),
                            severity: Severity::Error,
                            byte_range: call_node.start_byte()..call_node.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                    let lhs = call_node
                        .parent()
                        .filter(|p| p.kind() == "assignment")
                        .and_then(|p| p.child_by_field_name("left"));
                    match lhs {
                        Some(lhs) if lhs.kind() == "multioutput_variable" => {
                            if self.is_check_enabled("MCSMO") {
                                let pos = call_node.start_position();
                                diagnostics.push(Diagnostic {
                                    rule_id: "MCSMO",
                                    message: "Returning multiple outputs from a superclass object initialization is not supported."
                                        .to_string(),
                                    severity: Severity::Error,
                                    byte_range: call_node.start_byte()..call_node.end_byte(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                            }
                        }
                        Some(lhs) if lhs.kind() == "identifier" => {
                            let lhs_text = node_text(lhs, ctx.source);
                            if self.is_check_enabled("MCSCF")
                                && Some(lhs_text) != first_output.as_deref()
                            {
                                let pos = call_node.start_position();
                                diagnostics.push(Diagnostic {
                                    rule_id: "MCSCF",
                                    message:
                                        "A superclass constructor must be assigned to the first constructor output argument."
                                            .to_string(),
                                    severity: Severity::Error,
                                    byte_range: call_node.start_byte()..call_node.end_byte(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                            }
                        }
                        _ => {
                            if self.is_check_enabled("MCSCF") {
                                let pos = call_node.start_position();
                                diagnostics.push(Diagnostic {
                                    rule_id: "MCSCF",
                                    message:
                                        "A superclass constructor must be assigned to the first constructor output argument."
                                            .to_string(),
                                    severity: Severity::Error,
                                    byte_range: call_node.start_byte()..call_node.end_byte(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                            }
                        }
                    }
                } else if is_constructor {
                    // Inside the constructor but not using the first output argument.
                    if outputs.iter().any(|o| o == &caller) {
                        if self.is_check_enabled("MCSCO") {
                            let pos = call_node.start_position();
                            diagnostics.push(Diagnostic {
                                rule_id: "MCSCO",
                                message: "A superclass constructor must be called using the first constructor output argument."
                                    .to_string(),
                                severity: Severity::Error,
                                byte_range: call_node.start_byte()..call_node.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    } else if caller != mname && self.is_check_enabled("MCSCM") {
                        let pos = call_node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "MCSCM",
                            message: format!("To call a superclass method, the method name {} must match the name of the subclass method {}.", caller, mname),
                            severity: Severity::Error,
                            byte_range: call_node.start_byte()..call_node.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                } else if Some(caller.as_str()) == first_output.as_deref() {
                    // Superclass constructor call from a non-constructor method.
                    if self.is_check_enabled("MCSCC") {
                        let pos = call_node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "MCSCC",
                            message: format!("To call the superclass constructor, the name of the subclass constructor {} must match the name of the subclass {}.", caller, class.name),
                            severity: Severity::Error,
                            byte_range: call_node.start_byte()..call_node.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                } else if caller != mname && self.is_check_enabled("MCSCM") {
                    let pos = call_node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "MCSCM",
                        message: format!("To call a superclass method, the method name {} must match the name of the subclass method {}.", caller, mname),
                        severity: Severity::Error,
                        byte_range: call_node.start_byte()..call_node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }
    }

    /// MCCBU: report a superclass constructor call that follows a use of the object.
    pub(crate) fn check_super_after_object_use(
        func_node: tree_sitter::Node,
        call_node: tree_sitter::Node,
        caller: &str,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(block) = find_child_kind(func_node, "block") else {
            return;
        };
        let mut cur = call_node;
        let stmt = loop {
            match cur.parent() {
                Some(p) if p == block => break Some(cur),
                Some(p) => cur = p,
                None => break None,
            }
        };
        let Some(stmt) = stmt else {
            return;
        };
        let mut prior_use = false;
        let mut cursor = block.walk();
        for child in block.children(&mut cursor) {
            if child == stmt {
                break;
            }
            if child.is_named() && node_contains_identifier(child, caller, source) {
                prior_use = true;
                break;
            }
        }
        if prior_use {
            let pos = call_node.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "MCCBU",
                message:
                    "This superclass constructor is called after a use of the constructed object."
                        .to_string(),
                severity: Severity::Error,
                byte_range: call_node.start_byte()..call_node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
    }

    /// MCSCT: whether a super call is conditionalized or part of another expression.
    pub(crate) fn super_call_is_conditional_or_expression(call_node: tree_sitter::Node) -> bool {
        let parent = call_node.parent();
        let part_of_expression = match parent {
            Some(p) if p.kind() == "assignment" => p
                .child_by_field_name("right")
                .map(|r| r.id() != call_node.id())
                .unwrap_or(true),
            Some(p) => p.kind() != "assignment",
            None => true,
        };
        if part_of_expression {
            return true;
        }
        let mut cur = call_node;
        while let Some(p) = cur.parent() {
            if p.kind() == "function_definition" {
                break;
            }
            if matches!(
                p.kind(),
                "if_statement"
                    | "for_statement"
                    | "while_statement"
                    | "switch_statement"
                    | "try_statement"
                    | "spmd_statement"
            ) {
                return true;
            }
            cur = p;
        }
        false
    }

    /// Collect superclass calls (`X@Super(...)`) within a function node.
    pub(crate) fn collect_superclass_calls<'a>(
        &self,
        func_node: tree_sitter::Node<'a>,
        source: &str,
    ) -> Vec<(tree_sitter::Node<'a>, String, String)> {
        let mut out = Vec::new();
        let mut stack = vec![func_node];
        while let Some(n) = stack.pop() {
            if n.kind() == "function_call" && find_child_kind(n, "superclass").is_some() {
                let caller = {
                    let mut caller_cursor = n.walk();
                    let mut caller_name = String::new();
                    for child in n.children(&mut caller_cursor) {
                        if child.kind() == "identifier" {
                            caller_name = node_text(child, source).to_string();
                            break;
                        }
                    }
                    caller_name
                };
                let super_name = find_child_kind(n, "superclass")
                    .map(|s| node_text(s, source).trim().to_string())
                    .unwrap_or_default();
                out.push((n, caller, super_name));
            }
            let mut cursor = n.walk();
            for child in n.children(&mut cursor) {
                stack.push(child);
            }
        }
        out.reverse();
        out
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mccbs_fires_undeclared_superclass() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@NotSuper();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCCBS");
        assert!(
            !hits.is_empty(),
            "MCCBS should fire when the superclass constructor is not a declared superclass"
        );
    }

    #[test]
    fn test_mccbs_no_fire() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCCBS");
        assert!(
            hits.is_empty(),
            "MCCBS should NOT fire for a declared superclass"
        );
    }

    #[test]
    fn test_mccbu_fires_object_use_before_super() {
        let source = "\
classdef Foo < Bar
    properties
        y
    end
    methods
        function obj = Foo()
            obj.y = 1;
            obj = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCCBU");
        assert!(
            !hits.is_empty(),
            "MCCBU should fire when the object is used before the superclass constructor call"
        );
    }

    #[test]
    fn test_mccbu_no_fire() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCCBU");
        assert!(
            hits.is_empty(),
            "MCCBU should NOT fire when super call comes first"
        );
    }

    #[test]
    fn test_mccmc_fires_multiple_super_calls() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
            obj = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCCMC");
        assert!(
            !hits.is_empty(),
            "MCCMC should fire when the superclass constructor is called more than once"
        );
    }

    #[test]
    fn test_mccmc_no_fire() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCCMC");
        assert!(
            hits.is_empty(),
            "MCCMC should NOT fire for a single super call"
        );
    }

    #[test]
    fn test_mcscf_fires_not_assigned_to_first_output() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            x = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSCF");
        assert!(
            !hits.is_empty(),
            "MCSCF should fire when the superclass constructor is assigned to a non-first output"
        );
    }

    #[test]
    fn test_mcscf_no_fire() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSCF");
        assert!(
            hits.is_empty(),
            "MCSCF should NOT fire for a correct assignment"
        );
    }

    #[test]
    fn test_mcsco_fires_not_first_output_argument() {
        let source = "\
classdef Foo < Bar
    methods
        function [obj, other] = Foo()
            other = other@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSCO");
        assert!(
            !hits.is_empty(),
            "MCSCO should fire when the superclass constructor uses a non-first output argument"
        );
    }

    #[test]
    fn test_mcsco_no_fire() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSCO");
        assert!(
            hits.is_empty(),
            "MCSCO should NOT fire for the first output argument"
        );
    }

    #[test]
    fn test_mcsct_fires_conditional_super_call() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo(x)
            if x > 0
                obj = obj@Bar(1);
            end
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSCT");
        assert!(
            !hits.is_empty(),
            "MCSCT should fire when the superclass constructor call is conditionalized"
        );
    }

    #[test]
    fn test_mcsct_fires_super_in_expression() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = 1 + obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSCT");
        assert!(
            !hits.is_empty(),
            "MCSCT should fire when the superclass constructor call is part of another expression"
        );
    }

    #[test]
    fn test_mcsct_no_fire() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSCT");
        assert!(
            hits.is_empty(),
            "MCSCT should NOT fire for a top-level super call"
        );
    }

    #[test]
    fn test_mcsmo_fires_multiple_outputs() {
        let source = "\
classdef Foo < Bar
    methods
        function [obj, x] = Foo()
            [obj, x] = obj@Bar(1);
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSMO");
        assert!(
            !hits.is_empty(),
            "MCSMO should fire when a superclass object initialization has multiple outputs"
        );
    }

    #[test]
    fn test_mcsmo_no_fire() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSMO");
        assert!(hits.is_empty(), "MCSMO should NOT fire for a single output");
    }

    #[test]
    fn test_mscc_fires_super_call_in_non_constructor() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
        end
        function obj = helper()
            obj = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSCC");
        assert!(
            !hits.is_empty(),
            "MCSCC should fire when a superclass constructor is called from a non-constructor method"
        );
    }

    #[test]
    fn test_mscc_no_fire() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSCC");
        assert!(
            hits.is_empty(),
            "MCSCC should NOT fire for a valid constructor call"
        );
    }

    #[test]
    fn test_mscm_fires_wrong_superclass_method_name() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
        end
        function y = mymethod(obj)
            y = other@Bar(obj);
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSCM");
        assert!(
            !hits.is_empty(),
            "MCSCM should fire when a superclass method name does not match the enclosing method"
        );
    }

    #[test]
    fn test_mscm_no_fire() {
        let source = "\
classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
        end
        function y = mymethod(obj)
            y = mymethod@Bar(obj);
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCSCM");
        assert!(
            hits.is_empty(),
            "MCSCM should NOT fire for a matching method name"
        );
    }
}
