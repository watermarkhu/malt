//! SONUMOUT: `stepImpl` uses `varargout` but the class lacks `getNumOutputsImpl`.

use super::*;

impl SystemObjectsEngine {
    /// Check SONUMOUT: `stepImpl` declares `varargout` and the class does not define
    /// a `getNumOutputsImpl` method.
    pub(crate) fn check_sonumout(
        &self,
        class_node: tree_sitter::Node,
        source: &str,
        diags: &mut Vec<Diagnostic>,
    ) {
        if !self.is_enabled("SONUMOUT") {
            return;
        }
        let Some(header) = method_header(class_node, "stepImpl", source) else {
            return;
        };
        let Some(i) = header.find("stepImpl") else {
            return;
        };
        let outputs = &header[..i];
        let uses_varargs = outputs.contains("varargout");
        if uses_varargs && !class_has_method(class_node, "getNumOutputsImpl", source) {
            diags.push(make_diag("SONUMOUT", class_node));
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
    fn sonumout_varargs_without_getter_fires() {
        let source = system_class(&format!("{STEP}    end"));
        let diags = lint_file(&*engine(), &source);
        assert!(has_id(&diags, "SONUMOUT"), "got: {diags:?}");
    }

    #[test]
    fn sonumout_with_getter_does_not_fire() {
        let source = system_class(&format!(
            "{STEP}        function n = getNumOutputsImpl(obj)\n            n = 1;\n        end\n    end"
        ));
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SONUMOUT"), "got: {diags:?}");
    }

    #[test]
    fn sonumout_fixed_arity_does_not_fire() {
        let source = system_class(
            "    methods\n        function y = stepImpl(obj, u)\n            y = u;\n        end\n    end",
        );
        let diags = lint_file(&*engine(), &source);
        assert!(!has_id(&diags, "SONUMOUT"), "got: {diags:?}");
    }
}
