//! `check_validation_order` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_validation_order(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        if !self.is_check_enabled("FVSYN") {
            return diagnostics;
        }
        self.walk_validation_order(root, source, &mut diagnostics);
        diagnostics
    }

    pub(crate) fn walk_validation_order(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "arguments_statement" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "property" {
                    self.check_property_validation_order(child, source, diagnostics);
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_validation_order(child, source, diagnostics);
        }
    }

    /// A class name after the validation functions (`x {mustBeReal} double`) is a
    /// syntax error in MATLAB, reported as FVSYN at the class name. A class name
    /// inside the braces (`x {mustBeReal, double}`) is accepted.
    pub(crate) fn check_property_validation_order(
        &self,
        property: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let name = property.child_by_field_name("name");
        let Some(vf) = Self::find_validation_functions(property) else {
            return;
        };
        let class_id = (0..property.child_count())
            .filter_map(|i| property.child(i))
            .find(|c| c.kind() == "identifier" && Some(*c) != name);
        if let Some(class_id) = class_id {
            if class_id.start_byte() > vf.start_byte() {
                self.push_fvsyn_class(class_id, source, diagnostics);
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
        SyntaxErrorsEngine::from_config(&Config::default())
    }

    fn engine_with_disabled(checks: &[&str]) -> Box<dyn Rule> {
        let disabled = checks
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let config = Config::from_toml(&format!(
            "[lint.rules.SYNTAX_ERRORS_ENGINE]\ndisabled_checks = [{disabled}]\n"
        ))
        .unwrap();
        SyntaxErrorsEngine::from_config(&config)
    }

    // -- FVSYN: a class name after the validation functions ---

    #[test]
    fn fvsyn_fires_on_class_after_validation_functions() {
        let src = "function f(x)\n    arguments\n        x {mustBeReal} double\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "FVSYN"), "got: {diags:?}");
        assert!(!has_id(&diags, "VTPOD"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_fires_on_class_after_validation_functions_with_dims() {
        let src =
            "function f(x)\n    arguments\n        x (1,1) {mustBeReal} double\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_ok_on_class_inside_braces() {
        // MATLAB accepts a class name among the validators (probe: no finding).
        let src =
            "function f(x)\n    arguments\n        x (1,1) {mustBeReal, double}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_ok_on_sole_class_validator() {
        let src = "function f(x)\n    arguments\n        x {double}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_ok_on_correct_order() {
        let src = "function f(x)\n    arguments\n        x (1,1) double {mustBeReal}\n        y (1,1) double\n        z {mustBePositive}\n        w (1,1) double {mustBeReal} = 1\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_ok_on_class_without_dims() {
        let src = "function f(x)\n    arguments\n        x double {mustBePositive}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_ok_on_class_properties_block() {
        let src = "classdef Foo\n    properties\n        x {mustBeReal} double\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }

    #[test]
    fn fvsyn_disabled_via_config() {
        let engine = engine_with_disabled(&["FVSYN"]);
        let src = "function f(x)\n    arguments\n        x {mustBeReal} double\n    end\nend\n";
        let diags = lint_file(&*engine, src);
        assert!(!has_id(&diags, "FVSYN"), "got: {diags:?}");
    }
}
