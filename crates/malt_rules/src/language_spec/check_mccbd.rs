//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCCBD: Constructor must be defined in the class definition file.
    pub(crate) fn check_mccbd(
        &self,
        class: &ClassMeta,
        meta: &FileMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // If we're in a @directory class, the constructor may be in a separate file.
        // Check if there's a constructor in the methods blocks.
        let in_at_dir = ctx
            .file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .is_some_and(|dir| dir.starts_with('@'));

        if in_at_dir {
            // In @directory mode, skip this check — constructors can be in separate files.
            return;
        }

        // For single-file classes, check that a constructor is defined for non-abstract classes.
        // Actually, MCCBD checks that if a constructor IS defined, it should be in the class file.
        // This is mainly relevant for @folder classes where methods can be in separate files.
        // For single-file classes we verify the constructor is in a methods block (not local func).
        // A constructor prototype (`obj = Foo(a)`, a signature without a body) is
        // reported too, even when the class file also defines the constructor (probed).
        let mut stack = vec![ctx.tree.root_node()];
        while let Some(node) = stack.pop() {
            if node.kind() == "function_signature" {
                let text = &ctx.source[node.byte_range()];
                let head = text.split('(').next().unwrap_or("");
                let name = head.rsplit('=').next().unwrap_or("").trim();
                if name == class.name {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "MCCBD",
                        message: "Constructor must be fully defined in the class definition file."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: node.byte_range(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }

        for func in &meta.local_functions {
            if func.name == class.name {
                diagnostics.push(Diagnostic {
                    rule_id: "MCCBD",
                    message: "Constructor must be fully defined in the class definition file."
                        .to_string(),
                    severity: Severity::Error,
                    byte_range: func.byte_range.clone(),
                    line: func.line,
                    column: 1,
                    fix: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn constructor_prototype_is_mccbd_even_with_a_definition() {
        // Probed: `obj = Foo(a)` in a methods block gives MCCBD at its line (3),
        // with or without the definition below it.
        let proto = "classdef Foo\n    methods\n        obj = Foo(a)\n    end\n    properties\n        a\n    end\nend\n";
        let hits = filter_by_id(&check_source(proto, "Foo.m"), "MCCBD").len();
        assert_eq!(hits, 1);
        let both = "classdef Foo\n    methods\n        obj = Foo(a)\n        function obj = Foo(a)\n            obj.a = a;\n        end\n    end\n    properties\n        a\n    end\nend\n";
        let diags = check_source(both, "Foo.m");
        let hits = filter_by_id(&diags, "MCCBD");
        assert_eq!(hits.len(), 1, "got: {diags:?}");
        assert_eq!(hits[0].line, 3);
    }

    #[test]
    fn constructor_definition_alone_is_not_mccbd() {
        let source = "classdef Foo\n    methods\n        function obj = Foo(a)\n            obj.a = a;\n        end\n    end\n    properties\n        a\n    end\nend\n";
        assert!(filter_by_id(&check_source(source, "Foo.m"), "MCCBD").is_empty());
    }
}
