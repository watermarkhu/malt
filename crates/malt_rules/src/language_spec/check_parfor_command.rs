//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// Handle a command-syntax statement inside a parfor body (PFLD, PFSV, PFNAIO).
    pub(crate) fn check_parfor_command(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "command_name" {
                continue;
            }
            let name = node_text(child, source);
            match name {
                "load" => {
                    self.push_diag(
                        node,
                        "PFLD",
                        "'load' must assign to an output variable in parfor loops.",
                        diagnostics,
                    );
                }
                "save" => {
                    // `save foo -fromstruct s` is allowed; the option is in the command text.
                    if !node_text(node, source).contains("-fromstruct") {
                        self.push_diag(
                            node,
                            "PFSV",
                            "SAVE cannot be called in a PARFOR loop without the '-fromstruct' \
                             option.",
                            diagnostics,
                        );
                    }
                }
                "nargin" | "nargout" => {
                    self.push_diag(
                        node,
                        "PFNAIO",
                        "'nargin' and 'nargout' require a function argument in parfor loops.",
                        diagnostics,
                    );
                }
                _ => {}
            }
        }
    }

    /// PFNAIO for a bare `nargin`/`nargout` identifier used as a value in a parfor
    /// body, such as `y = nargin;`. Calls with arguments and field names are skipped.
    pub(crate) fn check_parfor_bare_nargin(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !matches!(node_text(node, source), "nargin" | "nargout") {
            return;
        }
        if node
            .parent()
            .is_some_and(|p| matches!(p.kind(), "function_call" | "field_expression"))
        {
            return;
        }
        self.push_diag(
            node,
            "PFNAIO",
            "'nargin' and 'nargout' require a function argument in parfor loops.",
            diagnostics,
        );
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_pfnaio_fires_bare_nargin_value_in_script_parfor() {
        let source = "\
parfor i=1:3
    y = nargin;
end
";
        let diags = check_source(source, "s.m");
        assert!(
            !filter_by_id(&diags, "PFNAIO").is_empty(),
            "PFNAIO should fire for `y = nargin;` in parfor"
        );
    }

    #[test]
    fn test_pfnaio_no_fire_nargin_with_argument_value() {
        let source = "\
parfor i=1:3
    y = nargin('f');
end
";
        let diags = check_source(source, "s.m");
        assert!(filter_by_id(&diags, "PFNAIO").is_empty());
    }

    #[test]
    fn test_pfsv_no_fire_command_save_with_fromstruct() {
        let source = "\
parfor i=1:3
    save foo -fromstruct s
    save foo
end
";
        let diags = check_source(source, "s.m");
        let pfsv = filter_by_id(&diags, "PFSV");
        assert_eq!(pfsv.len(), 1, "only the plain save should fire: {diags:?}");
        assert_eq!(pfsv[0].line, 3);
    }

    #[test]
    fn test_pfld_fires_load_command_form() {
        let source = "\
function f()
    parfor i = 1:10
        load data.mat
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFLD").is_empty(),
            "PFLD should fire for command-form load in parfor"
        );
    }
}
