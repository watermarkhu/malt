use super::*;

impl GoodPracticesEngine {
    /// VALST: `varargin` / `varargout` that is not the last name in its list.
    pub(crate) fn check_valst(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("VALST") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if node.kind() == "function_definition" {
                let mut cursor = node.walk();
                for part in node.children(&mut cursor) {
                    match part.kind() {
                        "function_arguments" => {
                            report_not_last(part, "varargin", source, &mut diagnostics)
                        }
                        "function_output" => {
                            let list = part
                                .named_child(0)
                                .filter(|c| c.kind() == "multioutput_variable");
                            if let Some(list) = list {
                                report_not_last(list, "varargout", source, &mut diagnostics);
                            }
                        }
                        _ => {}
                    }
                }
            }
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }
        diagnostics.sort_by_key(|d| d.byte_range.start);
        diagnostics
    }
}

/// Report `name` when it is in `list` but followed by another name.
fn report_not_last(list: Node, name: &str, source: &str, out: &mut Vec<Diagnostic>) {
    let mut cursor = list.walk();
    let names: Vec<Node> = list
        .children(&mut cursor)
        .filter(|c| c.kind() == "identifier")
        .collect();
    for (i, n) in names.iter().enumerate() {
        if &source[n.byte_range()] == name && i + 1 < names.len() {
            let pos = n.start_position();
            out.push(Diagnostic {
                rule_id: "VALST",
                message: format!("{name} must be the last argument in the argument list."),
                severity: Severity::Warning,
                byte_range: n.byte_range(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn valst(src: &str) -> usize {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_file(&*e, src)
            .iter()
            .filter(|d| d.rule_id == "VALST")
            .count()
    }

    #[test]
    fn valst_fires_when_varargin_or_varargout_is_not_last() {
        assert_eq!(valst("function f(varargin, x)\nend\n"), 1);
        assert_eq!(valst("function [varargout, b] = f(x)\nend\n"), 1);
    }

    #[test]
    fn valst_ok_when_last_or_nargin_used() {
        assert_eq!(
            valst("function varargout = f(x, varargin)\n  if nargin < 1\n  end\nend\n"),
            0
        );
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        assert!(!has_id(&lint_file(&*e, "function f(x)\nend\n"), "VALST"));
    }
}
