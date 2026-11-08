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
                    self.push_diag(
                        node,
                        "PFSV",
                        "SAVE cannot be called in a PARFOR loop without the '-fromstruct' \
                         option.",
                        diagnostics,
                    );
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
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

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
