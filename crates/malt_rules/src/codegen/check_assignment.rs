use super::*;

/// Whether `node` is a call to the function `name`.
fn is_call_to(node: tree_sitter::Node, source: &str, name: &str) -> bool {
    extract_func_name(node, source) == Some(name)
}

impl CodegenEngine {
    /// Check an `assignment` node for array growth (EMGRO), use of an
    /// undefined array in an indexed assignment (EMVDF), subscripted
    /// assignment of `load` output (EMLOAD) and direct reassignment of a
    /// fixed-point variable (FPASE).
    pub(crate) fn check_assignment<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let Some(rhs) = node.child_by_field_name("right").or_else(|| node.child(2)) else {
            return diags;
        };
        let Some(lhs) = node.child_by_field_name("left").or_else(|| node.child(0)) else {
            return diags;
        };

        // EMVDF: `x(k) = ...` where `x` was never mentioned earlier in the
        // enclosing function, so it is not fully defined before indexing.
        if self.is_enabled("EMVDF") && lhs.kind() == "function_call" {
            if let Some(name_node) = lhs.child_by_field_name("name") {
                if name_node.kind() == "identifier" {
                    let name = node_text(name_node, source);
                    let scope = enclosing_scope(node);
                    if !has_earlier_mention(scope, source, name, node.start_byte()) {
                        diags.push(make_diag("EMVDF", node));
                    }
                }
            }
        }

        // EMGRO: growth that is not `end + 1` indexing.
        if self.is_enabled("EMGRO") {
            if lhs.kind() == "function_call" && has_end_plus_other(node_text(lhs, source)) {
                diags.push(make_diag("EMGRO", node));
            } else if is_inside_loop(node) && lhs.kind() == "identifier" && rhs.kind() == "matrix" {
                // x = [x, val] inside a loop
                let var_name = node_text(lhs, source);
                if matrix_contains_var(rhs, source, var_name) {
                    diags.push(make_diag("EMGRO", node));
                }
            }
        }

        // EMLOAD: output of LOAD assigned with subscripting.
        if self.is_enabled("EMLOAD")
            && lhs.kind() == "function_call"
            && is_call_to(rhs, source, "load")
        {
            diags.push(make_diag("EMLOAD", node));
        }

        // FPASE: `y = ...` after `y = fi(...)` in an enclosing block.
        if self.is_enabled("FPASE")
            && lhs.kind() == "identifier"
            && !is_call_to(rhs, source, "fi")
            && Self::previously_assigned_fi(node, source, node_text(lhs, source))
        {
            diags.push(make_diag("FPASE", node));
        }

        diags
    }

    /// Whether an earlier statement in this or an enclosing block (up to the
    /// function boundary) assigned `name = fi(...)`.
    fn previously_assigned_fi(node: tree_sitter::Node, source: &str, name: &str) -> bool {
        let mut current = node;
        loop {
            let mut sib = current.prev_sibling();
            while let Some(s) = sib {
                if s.kind() == "assignment" {
                    let l = s.child_by_field_name("left").or_else(|| s.child(0));
                    let r = s.child_by_field_name("right").or_else(|| s.child(2));
                    if let (Some(l), Some(r)) = (l, r) {
                        if l.kind() == "identifier" && node_text(l, source) == name {
                            // The most recent assignment decides.
                            return is_call_to(r, source, "fi");
                        }
                    }
                }
                sib = s.prev_sibling();
            }
            match current.parent() {
                Some(p) if p.kind() != "function_definition" && p.kind() != "source_file" => {
                    current = p;
                }
                _ => return false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    // -- EMVDF ----------------------------------------------------------------

    #[test]
    fn emvdf_fires_on_indexing_undefined_variable() {
        let src = "function f()\n    x(3) = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMVDF"), "got: {diags:?}");
    }

    #[test]
    fn emvdf_not_fire_when_preallocated() {
        let src = "function f()\n    x = zeros(1, 3);\n    x(3) = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMVDF"), "got: {diags:?}");
    }

    #[test]
    fn emvdf_not_fire_on_parameter() {
        let src = "function f(x)\n    x(3) = 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMVDF"), "got: {diags:?}");
    }

    #[test]
    fn emvdf_not_fire_on_empty_init_in_loop() {
        let src = "for i = 1:10\n    x = [];\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMVDF"), "got: {diags:?}");
    }

    // -- EMGRO ----------------------------------------------------------------

    #[test]
    fn emgro_not_fire_on_end_plus_one() {
        let src = "x = zeros(1, 1);\nfor i = 1:10\n    x(end+1) = i;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMGRO"), "got: {diags:?}");
    }

    #[test]
    fn emgro_fires_on_end_plus_two() {
        let src = "x = zeros(1, 1);\nx(end+2) = 1;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMGRO"), "got: {diags:?}");
    }

    #[test]
    fn emgro_fires_on_concatenation_growth() {
        let src = "for i = 1:10\n    x = [x, i];\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMGRO"), "got: {diags:?}");
    }

    #[test]
    fn emgro_not_fire_on_indexed_assignment() {
        let src = "x = zeros(1, 10);\nfor i = 1:10\n    x(i) = i;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMGRO"), "got: {diags:?}");
    }

    // -- EMLOAD ---------------------------------------------------------------

    #[test]
    fn emload_fires_on_subscripted_load_assignment() {
        let src = "x(1) = load('f.mat');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMLOAD"), "got: {diags:?}");
    }

    #[test]
    fn emload_not_fire_on_plain_assignment() {
        let src = "s = load('f.mat');\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMLOAD"), "got: {diags:?}");
    }

    // -- FPASE ----------------------------------------------------------------

    #[test]
    fn fpase_fires_on_direct_reassignment_of_fi() {
        let src = "y = fi(0, 1, 16);\nfor n = 1:3\n    y = y + 1;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "FPASE"), "got: {diags:?}");
    }

    #[test]
    fn fpase_not_fire_on_fi_definition() {
        let src = "x = fi(y, 1, 16);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "FPASE"), "got: {diags:?}");
    }

    #[test]
    fn fpase_not_fire_on_subscripted_assignment() {
        let src = "y = fi(0, 1, 16);\ny(:) = y + 1;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "FPASE"), "got: {diags:?}");
    }

    #[test]
    fn fpase_not_fire_without_fi() {
        let src = "y = 0;\ny = y + 1;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "FPASE"), "got: {diags:?}");
    }
}
