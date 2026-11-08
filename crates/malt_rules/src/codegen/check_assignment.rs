use super::*;

impl CodegenEngine {
    /// Check an `assignment` node for growth patterns (EMGRO) and
    /// variable-size data (EMVDF).
    pub(crate) fn check_assignment<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let rhs = match node.child_by_field_name("right").or_else(|| node.child(2)) {
            Some(n) => n,
            None => return diags,
        };

        let rhs_text = node_text(rhs, source);

        // EMVDF: dynamic allocation patterns like `zeros(1, n)` where n is not constant
        // Simplified heuristic: detect `[]` as empty matrix initialization
        if self.is_enabled("EMVDF") && rhs_text == "[]" && is_inside_loop(node) {
            diags.push(make_diag("EMVDF", node));
        }

        // EMGRO: growing arrays inside loops (same pattern as AGROW)
        if self.is_enabled("EMGRO") && is_inside_loop(node) {
            let lhs = match node.child_by_field_name("left").or_else(|| node.child(0)) {
                Some(n) => n,
                None => return diags,
            };

            // Pattern: x(end+1) = ... or x = [x, val]
            if lhs.kind() == "function_call" && contains_end_plus_pattern(lhs, source) {
                diags.push(make_diag("EMGRO", node));
            } else if lhs.kind() == "identifier" && rhs.kind() == "matrix" {
                let var_name = node_text(lhs, source);
                if matrix_contains_var(rhs, source, var_name) {
                    diags.push(make_diag("EMGRO", node));
                }
            }
        }

        // FPASE: assignment to scaled expression (heuristic: detect fi() on RHS)
        if self.is_enabled("FPASE") && rhs_text.contains("fi(") {
            diags.push(make_diag("FPASE", node));
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_nodes};

    // -- EMVDF ----------------------------------------------------------------

    #[test]
    fn emvdf_fires_on_empty_init_in_loop() {
        let src = "for i = 1:10\n    x = [];\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "EMVDF"), "got: {diags:?}");
    }

    #[test]
    fn emvdf_not_fire_outside_loop() {
        let src = "x = [];\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMVDF"), "got: {diags:?}");
    }

    // -- EMGRO ----------------------------------------------------------------

    #[test]
    fn emgro_fires_on_end_plus_one_growth() {
        let src = "for i = 1:10\n    x(end+1) = i;\nend\n";
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
        let src = "for i = 1:10\n    x(i) = i;\nend\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "EMGRO"), "got: {diags:?}");
    }

    // -- FPASE ----------------------------------------------------------------

    #[test]
    fn fpase_fires_on_fi_rhs() {
        let src = "x = fi(y, 1, 16);\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(has_id(&diags, "FPASE"), "got: {diags:?}");
    }

    #[test]
    fn fpase_not_fire_on_plain_rhs() {
        let src = "x = y + 1;\n";
        let diags = lint_nodes(&*engine(), src);
        assert!(!has_id(&diags, "FPASE"), "got: {diags:?}");
    }
}
