//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// Check the parfor header: loop variable name (PFANSLP) and range (PFRNG).
    pub(crate) fn check_parfor_header(
        &self,
        node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // PFANSLP: 'ans' as the parfor loop variable
        if let Some(var) = self.extract_for_variable(node, source) {
            if var == "ans" {
                self.push_diag(
                    node,
                    "PFANSLP",
                    "'ans' is not supported as a parfor loop variable.",
                    diagnostics,
                );
            }
        }

        // PFRNG: the parfor range must be increasing consecutive integers
        if let Some(range_node) = self.parfor_range_node(node) {
            let mut named = 0usize;
            let mut step_is_one = true;
            let mut cursor = range_node.walk();
            for child in range_node.children(&mut cursor) {
                if child.is_named() {
                    named += 1;
                    if named == 2 {
                        step_is_one = node_text(child, source).trim() == "1";
                    }
                }
            }
            if named >= 3 && !step_is_one {
                self.push_diag(
                    range_node,
                    "PFRNG",
                    "The range of a PARFOR statement must be consecutive integers.",
                    diagnostics,
                );
            }
        }
    }

    /// Find the `range` node of a for/parfor statement's iterator, if any.
    pub(crate) fn parfor_range_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
    ) -> Option<tree_sitter::Node<'a>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "iterator" {
                let mut inner = child.walk();
                for c in child.children(&mut inner) {
                    if c.kind() == "range" {
                        return Some(c);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn test_pfanslp_fires_ans_parfor_variable() {
        let source = "\
function f()
    parfor ans = 1:10
        y = ans;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFANSLP").is_empty(),
            "PFANSLP should fire for 'ans' as parfor variable"
        );
    }

    #[test]
    fn test_pfanslp_no_fire_normal_parfor_variable() {
        let source = "\
function f()
    parfor i = 1:10
        y = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFANSLP").is_empty(),
            "PFANSLP should NOT fire for a normal parfor variable"
        );
    }

    #[test]
    fn test_pfrng_fires_non_unit_step() {
        let source = "\
function f()
    parfor i = 1:2:10
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFRNG").is_empty(),
            "PFRNG should fire for a non-unit step"
        );
    }

    #[test]
    fn test_pfrng_fires_negative_step() {
        let source = "\
function f()
    parfor i = 10:-1:1
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFRNG").is_empty(),
            "PFRNG should fire for a negative step"
        );
    }

    #[test]
    fn test_pfrng_no_fire_unit_step() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFRNG").is_empty(),
            "PFRNG should NOT fire for a unit step"
        );
    }
}
