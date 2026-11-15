//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// MCPIN: a property cannot be initialized to an instance of the class itself.
    pub(crate) fn check_mcpin(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        if !self.is_check_enabled("MCPIN") {
            return;
        }
        for pb in &class.properties_blocks {
            // Probed: a Constant property whose default builds the class is silent.
            if Attributes(&pb.attributes).is_set("Constant") {
                continue;
            }
            for prop in &pb.properties {
                if let Some(ref default_value) = prop.default_value {
                    // Probed: a bare class name (`P = Foo;`) is a constructor call too.
                    let bare = default_value.trim() == class.name;
                    if bare || contains_self_constructor_call(default_value, &class.name) {
                        diagnostics.push(Diagnostic {
                            rule_id: "MCPIN",
                            message: "Unable to initialize class property to an instance of the class itself.".to_string(),
                            severity: Severity::Error,
                            byte_range: prop.byte_range.clone(),
                            line: prop.line,
                            column: 1,
                            fix: None,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_mcpin_fires_self_instance_default() {
        let source = "\
classdef Foo
    properties
        x Foo = Foo()
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCPIN");
        assert!(
            !hits.is_empty(),
            "MCPIN should fire when a property is initialized to an instance of the class itself"
        );
    }

    #[test]
    fn test_mcpin_no_fire() {
        let source = "\
classdef Foo
    properties
        x double = 5
    end
end
";
        let diags = check_source(source, "Foo.m");
        let hits = filter_by_id(&diags, "MCPIN");
        assert!(
            hits.is_empty(),
            "MCPIN should NOT fire for a plain default value"
        );
    }

    #[test]
    fn test_mcpin_fires_on_bare_class_name_default() {
        // Probed (P2): `P = P2;` with no parentheses is reported at the default's line.
        let source = "classdef P2\n    properties\n        P = P2;\n    end\nend\n";
        let diags = check_source(source, "P2.m");
        let hits = filter_by_id(&diags, "MCPIN");
        assert_eq!(hits.len(), 1, "got: {diags:?}");
        assert_eq!(hits[0].line, 3);
    }

    #[test]
    fn test_mcpin_silent_for_constant_property_and_empty_call() {
        // Probed (P3, P6): a Constant property that builds the class is silent, and so is
        // `P6.empty(0, 0)`, which does not construct an instance.
        for source in [
            "classdef P3\n    properties (Constant)\n        P = P3();\n    end\nend\n",
            "classdef P6\n    properties\n        x = P6.empty(0, 0);\n    end\nend\n",
        ] {
            let diags = check_source(source, "P.m");
            assert!(
                filter_by_id(&diags, "MCPIN").is_empty(),
                "{source}: got: {diags:?}"
            );
        }
    }

    #[test]
    fn test_mcpin_fires_on_non_constant_property_beside_constant() {
        // Probed (P5): the Constant property (line 3) is silent, the other one (line 6) is reported.
        let source = "classdef P5\n    properties (Constant)\n        A = P5();\n    end\n    properties\n        x P5 = P5();\n    end\nend\n";
        let diags = check_source(source, "P5.m");
        let lines: Vec<usize> = filter_by_id(&diags, "MCPIN")
            .iter()
            .map(|d| d.line)
            .collect();
        assert_eq!(lines, vec![6], "got: {diags:?}");
    }
}
