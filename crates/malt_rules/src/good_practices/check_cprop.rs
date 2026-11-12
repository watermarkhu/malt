use super::check_prop_validation::{is_app_class, property_name_uses, NameUse};
use super::*;

impl GoodPracticesEngine {
    /// CPROP: a bare read or call of a property name in a method that does not use the
    /// name as a variable, so the name refers to the property (probed: `y = Color(1);`
    /// and `y = Color;` in a method of a class with a property `Color`). Static methods
    /// and App Designer classes are silent.
    pub(crate) fn check_cprop(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("CPROP") || is_app_class(tree, source) {
            return Vec::new();
        }
        property_name_uses(tree, source)
            .iter()
            .filter(|n| !n.is_variable)
            .flat_map(|n| n.uses.iter().map(|u| cprop_diagnostic(&n.name, u)))
            .collect()
    }
}

/// The CPROP diagnostic for one bare use of a property name.
fn cprop_diagnostic(name: &str, usage: &NameUse) -> Diagnostic {
    Diagnostic {
        rule_id: "CPROP",
        message: format!(
            "'{name}' is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename the property to improve readability."
        ),
        severity: Severity::Warning,
        byte_range: usage.byte_range.clone(),
        line: usage.line,
        column: usage.column,
        fix: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::lint_file;
    use malt_core::Config;

    /// Lines of the CPROP diagnostics for `src`.
    fn cprop_lines(src: &str) -> Vec<usize> {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_file(&*e, src)
            .iter()
            .filter(|d| d.rule_id == "CPROP")
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn cprop_fires_on_a_call_or_read_of_a_property_name() {
        // Probed: `y = Color(1);` (7) and `y = Color;` (7) in plain classes.
        let call = "classdef C\n properties\n  Color\n end\n methods\n  function y = go(obj)\n   y = Color(1);\n  end\n end\nend\n";
        let read = "classdef C\n properties\n  Color\n end\n methods\n  function y = go(obj)\n   y = Color;\n  end\n end\nend\n";
        assert_eq!(cprop_lines(call), vec![7]);
        assert_eq!(cprop_lines(read), vec![7]);
    }

    #[test]
    fn cprop_fires_on_a_constant_property_read_and_in_a_handle_class() {
        // Probed: `y = K.a;` (7) on a constant; `y = Color(1) + 1;` (8) in a handle class.
        let constant = "classdef C\n properties (Constant)\n  K = 1\n end\n methods\n  function y = go(obj)\n   y = K.a;\n  end\n end\nend\n";
        let handle = "classdef C < handle\n properties\n  Color\n end\n methods\n  function y = go(obj)\n   if true\n    y = Color(1) + 1;\n   end\n  end\n end\nend\n";
        assert_eq!(cprop_lines(constant), vec![7]);
        assert_eq!(cprop_lines(handle), vec![8]);
    }

    #[test]
    fn cprop_is_silent_when_the_method_uses_the_name_as_a_variable() {
        // Probed: `Color = ...` before the use makes it PROP, not CPROP.
        let src = "classdef C\n properties\n  Color\n end\n methods\n  function y = go(obj)\n   Color = [1 2 3];\n   y = Color(1);\n  end\n end\nend\n";
        assert!(cprop_lines(src).is_empty());
    }

    #[test]
    fn cprop_is_silent_for_a_method_call_event_static_method_and_field() {
        // Probed: an event call (17), a static method (20) and `obj.Color(1)` (16) are silent.
        let event = "classdef C\n events\n  Changed\n end\n methods\n  function go(obj)\n   Changed(1);\n  end\n end\nend\n";
        let stat = "classdef C\n properties\n  Color\n end\n methods (Static)\n  function y = go()\n   y = Color(1);\n  end\n end\nend\n";
        let field = "classdef C\n properties\n  Color\n end\n methods\n  function y = go(obj)\n   y = obj.Color(1);\n  end\n end\nend\n";
        assert!(cprop_lines(event).is_empty());
        assert!(cprop_lines(stat).is_empty());
        assert!(cprop_lines(field).is_empty());
    }

    #[test]
    fn cprop_is_silent_in_app_designer_classes() {
        // Probed: App Designer reports ADPROP for a read (158), not CPROP.
        let src = "classdef C < matlab.apps.AppBase\n properties\n  Count = 0\n end\n methods\n  function r = go(app)\n   r = Count;\n  end\n end\nend\n";
        assert!(cprop_lines(src).is_empty());
    }
}
