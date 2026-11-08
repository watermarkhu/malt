//! # ISCHR, ISSTR, ISLOG, ISCEL, ISMAT: `isa(x, 'type')` → `istype(x)`
//!
//! `check_isa` rewrites `isa(x, 'char')` / `isa(x, 'string')` /
//! `isa(x, 'logical')` / `isa(x, 'cell')` / `isa(x, 'double')` to the
//! dedicated `ischar` / `isstring` / `islogical` / `iscell` / `isnumeric`
//! predicates.

use super::*;

impl ReadabilityEngine {
    /// ISCHR, ISSTR, ISLOG, ISCEL, ISMAT: `isa(x, 'type')` → `istype(x)`
    pub(crate) fn check_isa(
        &self,
        node: tree_sitter::Node,
        source: &str,
        results: &mut Vec<Diagnostic>,
    ) {
        let args = match find_arguments(node) {
            Some(a) => a,
            None => return,
        };

        // isa requires exactly 2 arguments
        let named_children: Vec<_> = (0..args.named_child_count())
            .filter_map(|i| args.named_child(i))
            .collect();

        if named_children.len() != 2 {
            return;
        }

        let first_arg = named_children[0];
        let second_arg = named_children[1];
        let first_text = &source[first_arg.start_byte()..first_arg.end_byte()];
        let second_text = &source[second_arg.start_byte()..second_arg.end_byte()];

        // Strip quotes from second argument
        let type_name = second_text
            .trim_start_matches('\'')
            .trim_end_matches('\'')
            .trim_start_matches('"')
            .trim_end_matches('"');

        for &(isa_type, check_id, replacement_fn, message) in ISA_REPLACEMENTS {
            if type_name == isa_type && self.is_check_enabled(check_id) {
                let fix_text = format!("{replacement_fn}({first_text})");
                results.push(self.diag(
                    check_id,
                    message,
                    node,
                    Some(Fix::new(node.start_byte()..node.end_byte(), fix_text)),
                ));
                return;
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

    // -- ISCHR ---------------------------------------------------------------

    #[test]
    fn ischr_isa_char_fires() {
        let diags = lint_nodes(&*engine(), "x = isa(a, 'char');\n");
        assert!(has_id(&diags, "ISCHR"), "got: {diags:?}");
    }

    #[test]
    fn ischr_isa_other_type_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = isa(a, 'numeric');\n");
        assert!(!has_id(&diags, "ISCHR"), "got: {diags:?}");
    }

    // -- ISSTR ---------------------------------------------------------------

    #[test]
    fn isstr_isa_string_fires() {
        let diags = lint_nodes(&*engine(), "x = isa(a, 'string');\n");
        assert!(has_id(&diags, "ISSTR"), "got: {diags:?}");
    }

    #[test]
    fn isstr_isa_other_type_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = isa(a, 'numeric');\n");
        assert!(!has_id(&diags, "ISSTR"), "got: {diags:?}");
    }

    // -- ISLOG ---------------------------------------------------------------

    #[test]
    fn islog_isa_logical_fires() {
        let diags = lint_nodes(&*engine(), "x = isa(a, 'logical');\n");
        assert!(has_id(&diags, "ISLOG"), "got: {diags:?}");
    }

    #[test]
    fn islog_isa_other_type_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = isa(a, 'numeric');\n");
        assert!(!has_id(&diags, "ISLOG"), "got: {diags:?}");
    }

    // -- ISCEL ---------------------------------------------------------------

    #[test]
    fn iscel_isa_cell_fires() {
        let diags = lint_nodes(&*engine(), "x = isa(a, 'cell');\n");
        assert!(has_id(&diags, "ISCEL"), "got: {diags:?}");
    }

    #[test]
    fn iscel_isa_other_type_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = isa(a, 'numeric');\n");
        assert!(!has_id(&diags, "ISCEL"), "got: {diags:?}");
    }

    // -- ISMAT ---------------------------------------------------------------

    #[test]
    fn ismat_isa_double_fires() {
        let diags = lint_nodes(&*engine(), "x = isa(a, 'double');\n");
        assert!(has_id(&diags, "ISMAT"), "got: {diags:?}");
    }

    #[test]
    fn ismat_isa_other_type_does_not_fire() {
        let diags = lint_nodes(&*engine(), "x = isa(a, 'numeric');\n");
        assert!(!has_id(&diags, "ISMAT"), "got: {diags:?}");
    }
}
