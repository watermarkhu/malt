//! Node-level `function_call`, `attribute` and `field_expression` dispatch
//! for the unsupported features engine.
//!
//! * Unsupported functions (`awtinvoke`, `axescheck`, `feature`, `findpackage`,
//!   `hgconvertunits`, `ismembc`, `setptr`, `system_dependent`, `uirestore`,
//!   `uisuspend`) map to their own check IDs.
//! * `IMPKG` / `MIPKG`: references to `internal.matlab.*` and
//!   `matlab.<...>.internal.*` namespaces.
//! * `MCADE`: a `Description` class-member attribute.

use super::*;

impl UnsupportedEngine {
    /// Check a `function_call` node for unsupported function usage.
    pub(crate) fn check_function_call<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // `obj.feature(...)` is a method call, not the function.
        if node
            .parent()
            .is_some_and(|p| p.kind() == "field_expression")
        {
            return diags;
        }
        let Some(func_name) = extract_func_name(node, source) else {
            return diags;
        };

        let found = UNSUPPORTED_FUNCTIONS.iter().find(|(n, _)| *n == func_name);
        if let Some((_, id)) = found.filter(|(_, id)| self.is_enabled(id)) {
            diags.push(make_diag(id, node));
        }
        diags
    }

    /// Check an outermost `field_expression` for internal namespace usage.
    pub(crate) fn check_field_expression<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        if node
            .parent()
            .is_some_and(|p| p.kind() == "field_expression")
        {
            return Vec::new();
        }
        // Build the dotted prefix of identifier segments (stop at calls/indexing).
        let mut segs: Vec<&str> = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "identifier" => segs.push(node_text(child, source)),
                "function_call" => {
                    if let Some(n) = extract_func_name(child, source) {
                        segs.push(n);
                    }
                    break;
                }
                _ => break,
            }
        }
        match namespace_check(&segs.join(".")) {
            Some(id) if self.is_enabled(id) => vec![make_diag(id, node)],
            _ => Vec::new(),
        }
    }

    /// Check a class-member `attribute` for the unsupported `Description`.
    pub(crate) fn check_attribute<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let Some(first) = node.named_child(0) else {
            return Vec::new();
        };
        if self.is_enabled("MCADE")
            && first.kind() == "identifier"
            && node_text(first, source) == "Description"
        {
            vec![make_diag("MCADE", node)]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    fn ids(src: &str) -> Vec<&'static str> {
        lint_nodes(&*engine(), src)
            .iter()
            .map(|d| d.rule_id)
            .collect()
    }

    #[test]
    fn unsupported_functions_fire_with_own_id() {
        for (f, id) in UNSUPPORTED_FUNCTIONS {
            assert_eq!(ids(&format!("x = {f}(1);\n")), vec![*id], "{f}");
        }
    }

    #[test]
    fn ordinary_calls_do_not_fire() {
        for src in [
            "disp('hi');\n",
            "tf = ismember(x, y);\n",
            "s = serialport('COM1', 9600);\n",
            "v = version;\n",
        ] {
            assert!(ids(src).is_empty(), "{src}");
        }
    }

    #[test]
    fn method_call_with_same_name_does_not_fire() {
        assert!(ids("obj.feature(1);\n").is_empty());
    }

    #[test]
    fn old_wrong_detections_removed() {
        for src in [
            "deploytool();\n",
            "ex = actxserver('x');\n",
            "p = findprop(o, 'N');\n",
            "h = uicontainer();\n",
            "tf = isMember(x, y);\n",
            "s = serial('COM1');\n",
            "guidata(h, d);\n",
            "uiresume(gcf);\n",
        ] {
            assert!(ids(src).is_empty(), "{src}");
        }
    }

    #[test]
    fn impkg_fires_on_internal_matlab_call() {
        assert_eq!(ids("x = internal.matlab.foo(1);\n"), vec!["IMPKG"]);
    }

    #[test]
    fn mipkg_fires_on_matlab_internal_reference() {
        assert_eq!(ids("y = matlab.internal.a.b;\n"), vec!["MIPKG"]);
        assert_eq!(
            ids("y = matlab.graphics.internal.draw(1);\n"),
            vec!["MIPKG"]
        );
    }

    #[test]
    fn regular_namespaces_do_not_fire() {
        assert!(ids("y = matlab.lang.makeValidName('a');\n").is_empty());
        assert!(ids("y = s.internal.matlab.x;\n").is_empty());
    }

    #[test]
    fn mcade_fires_on_description_attribute() {
        let src = "classdef A\n properties (Description = 'x')\n a\n end\nend\n";
        assert!(has_id(&lint_nodes(&*engine(), src), "MCADE"));
    }

    #[test]
    fn mcade_ignores_other_attributes() {
        let src = "classdef A\n properties (Access = 'private')\n a\n end\nend\n";
        assert!(ids(src).is_empty());
    }
}
