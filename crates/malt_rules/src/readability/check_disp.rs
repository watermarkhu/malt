//! # DSPSP: `disp(sprintf(...))` → `fprintf`
//!
//! `check_disp` rewrites `disp(sprintf(...))` into a direct `fprintf(...)`
//! call, skipping the intermediate string construction.

use super::*;

impl ReadabilityEngine {
    /// DSPSP: `disp(sprintf(...))` → `fprintf`
    pub(crate) fn check_disp(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("DSPSP") {
            return;
        }

        let args = match find_arguments(node) {
            Some(a) => a,
            None => return,
        };

        let named_children: Vec<_> = (0..args.named_child_count())
            .filter_map(|i| args.named_child(i))
            .collect();

        if named_children.len() != 1 {
            return;
        }

        let inner = named_children[0];
        if inner.kind() == "function_call" {
            if let Some(inner_name) = inner.child_by_field_name("name") {
                let inner_func = &source[inner_name.start_byte()..inner_name.end_byte()];
                if inner_func == "sprintf" {
                    // Extract sprintf arguments to construct fprintf replacement
                    if let Some(inner_args) = find_arguments(inner) {
                        let inner_args_text =
                            &source[inner_args.start_byte()..inner_args.end_byte()];
                        results.push(self.diag(
                            "DSPSP",
                            "'disp(sprintf(...))' can usually be replaced by 'fprintf(...\\n)'.",
                            node,
                            Some(Fix::new(
                                node.start_byte()..node.end_byte(),
                                format!("fprintf{inner_args_text}"),
                            )),
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};
    use malt_core::Config;

    fn engine() -> Box<dyn Rule> {
        ReadabilityEngine::from_config(&Config::default())
    }

    // -- DSPSP ---------------------------------------------------------------

    #[test]
    fn dspsp_disp_sprintf_fires() {
        let diags = lint_nodes(&*engine(), "disp(sprintf('%d', x));\n");
        assert!(has_id(&diags, "DSPSP"), "got: {diags:?}");
    }

    #[test]
    fn dspsp_disp_direct_does_not_fire() {
        let diags = lint_nodes(&*engine(), "disp(x);\n");
        assert!(!has_id(&diags, "DSPSP"), "got: {diags:?}");
    }
}
