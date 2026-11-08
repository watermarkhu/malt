//! `check_validation_order` — one check of the Syntax Errors engine.

use super::*;

impl SyntaxErrorsEngine {
    pub(crate) fn check_validation_order(&self, root: Node, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        if !self.is_check_enabled("VTPOD") {
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

    pub(crate) fn check_property_validation_order(
        &self,
        property: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let name = property.child_by_field_name("name");
        let vf = Self::find_validation_functions(property);
        let mut class_id: Option<Node> = None;

        for i in 0..property.child_count() {
            let Some(child) = property.child(i) else {
                continue;
            };
            if child.kind() == "identifier" && Some(child) != name && class_id.is_none() {
                class_id = Some(child);
            }
        }

        // Pattern A: a class identifier appears after the validation functions.
        if let (Some(class_id), Some(vf)) = (class_id, vf) {
            if class_id.start_byte() > vf.start_byte() {
                self.push_vtpod(class_id, diagnostics);
                return;
            }
        }

        // Pattern B: a class name appears inside the validation functions.
        if class_id.is_none() {
            if let Some(vf) = vf {
                let mut cursor = vf.walk();
                for child in vf.children(&mut cursor) {
                    if child.kind() != "identifier" {
                        continue;
                    }
                    let text = &source[child.start_byte()..child.end_byte()];
                    if KNOWN_CLASS_NAMES.contains(&text) {
                        self.push_vtpod(child, diagnostics);
                    }
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

    // -- VTPOD: validation order must be size, then class, then functions ---

    #[test]
    fn vtpod_fires_on_class_inside_braces_with_dims() {
        let src =
            "function f(x)\n    arguments\n        x (1,1) {mustBeReal, double}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "VTPOD"), "got: {diags:?}");
    }

    #[test]
    fn vtpod_fires_on_class_inside_braces_no_dims() {
        let src = "function f(x)\n    arguments\n        x {mustBeReal, double}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "VTPOD"), "got: {diags:?}");
    }

    #[test]
    fn vtpod_fires_on_class_as_sole_validator() {
        let src = "function f(x)\n    arguments\n        x {double}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "VTPOD"), "got: {diags:?}");
    }

    #[test]
    fn vtpod_fires_on_class_after_validation_functions() {
        let src = "function f(x)\n    arguments\n        x {mustBeReal} double\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(has_id(&diags, "VTPOD"), "got: {diags:?}");
    }

    #[test]
    fn vtpod_ok_on_correct_order() {
        let src = "function f(x)\n    arguments\n        x (1,1) double {mustBeReal}\n        y (1,1) double\n        z {mustBePositive}\n        w (1,1) double {mustBeReal} = 1\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "VTPOD"), "got: {diags:?}");
    }

    #[test]
    fn vtpod_ok_on_class_without_dims() {
        let src = "function f(x)\n    arguments\n        x double {mustBePositive}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "VTPOD"), "got: {diags:?}");
    }

    #[test]
    fn vtpod_ok_on_class_properties_block() {
        let src = "classdef Foo\n    properties\n        x {mustBeReal, double}\n    end\nend\n";
        let diags = lint_file(&*engine(), src);
        assert!(!has_id(&diags, "VTPOD"), "got: {diags:?}");
    }

    #[test]
    fn vtpod_disabled_via_config() {
        let engine = engine_with_disabled(&["VTPOD"]);
        let src =
            "function f(x)\n    arguments\n        x (1,1) {mustBeReal, double}\n    end\nend\n";
        let diags = lint_file(&*engine, src);
        assert!(!has_id(&diags, "VTPOD"), "got: {diags:?}");
    }
}
