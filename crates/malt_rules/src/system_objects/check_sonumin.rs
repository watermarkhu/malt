//! SONUMIN: `stepImpl` uses `varargin` but the class lacks `getNumInputsImpl`.

use super::*;

impl SystemObjectsEngine {
    /// Check SONUMIN: `stepImpl` declares `varargin` and the class does not define
    /// a `getNumInputsImpl` method.
    pub(crate) fn check_sonumin(
        &self,
        class_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_enabled("SONUMIN") {
            return;
        }
        let Some(header) = method_header(class_node, "stepImpl", source) else {
            return;
        };
        let Some(i) = header.find("stepImpl") else {
            return;
        };
        let params = &header[i..];
        let uses_varargs = params.contains("varargin");
        if uses_varargs && !class_has_method(class_node, "getNumInputsImpl", source) {
            diags.push(make_diag("SONUMIN", class_node));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{engine, system_class};
    use crate::test_util::{has_id, lint_file};

    const STEP: &str =
        "    methods\n        function varargout = stepImpl(obj, varargin)\n        end\n";

    #[test]
    fn sonumin_varargs_without_getter_fires() {
        let source = system_class(&format!("{STEP}    end"));
        let diags = lint_file(&*engine(), &source);
        assert!(has_id(&diags, "SONUMIN"), "got: {diags:?}");
    }

    #[test]
    fn sonumin_with_getter_does_not_fire() {
        let source = system_class(&format!(
            "{STEP}        function n = getNumInputsImpl(obj)\n            n = 1;\n        end\n    end"
        ));
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SONUMIN"), "got: {diags:?}");
    }

    #[test]
    fn sonumin_fixed_arity_does_not_fire() {
        let source = system_class(
            "    methods\n        function y = stepImpl(obj, u)\n            y = u;\n        end\n    end",
        );
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SONUMIN"), "got: {diags:?}");
    }
}
