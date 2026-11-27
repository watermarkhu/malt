//! `check_dupnamearg` is one check of the Language Specification engine.

use super::*;

impl LanguageSpecEngine {
    /// DUPNAMEARG: a `Name=value` argument repeats a name already given in the
    /// same call. Probed against R2026b: the names compare case-insensitively,
    /// the callee need not be defined (local, builtin or unknown function), each
    /// repeat after the first is reported, and the `'Name', value` form is not.
    pub(crate) fn check_dupnamearg(
        &self,
        root: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled("DUPNAMEARG") {
            return;
        }
        let mut seen = HashSet::new();
        Self::walk_dupnamearg(root, source, &mut seen, diagnostics);
    }

    fn walk_dupnamearg(
        node: tree_sitter::Node,
        source: &str,
        seen: &mut HashSet<(usize, String)>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.kind() == "=" {
            if let (Some(args), Some(name)) = (node.parent(), node.prev_named_sibling()) {
                let in_call = args.kind() == "function_call"
                    || args.parent().is_some_and(|p| p.kind() == "function_call");
                if in_call && name.kind() == "identifier" {
                    let text = source[name.byte_range()].to_lowercase();
                    if !seen.insert((args.id(), text)) {
                        let pos = name.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "DUPNAMEARG",
                            message: "This named argument was already provided. If this function does not allow or ignores repeated named arguments, remove one of the duplicated named arguments.".to_string(),
                            severity: Severity::Warning,
                            byte_range: name.byte_range(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_dupnamearg(child, source, seen, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn dupnamearg_fires_on_repeat_in_local_call() {
        let source = "function d1()\nf1(Name=1, Name=2)\nend\nfunction f1(opts)\narguments\n  opts.Name (1,1) double = 0\nend\nend\n";
        let diags = check_source(source, "d1.m");
        let hits = filter_by_id(&diags, "DUPNAMEARG");
        assert_eq!(hits.len(), 1, "got: {diags:?}");
        assert_eq!((hits[0].line, hits[0].column), (2, 12));
    }

    #[test]
    fn dupnamearg_fires_for_unknown_callee_and_each_repeat() {
        let source = "function d()\nunknownfn(Name=1, Name=2, Name=3)\nend\n";
        let diags = check_source(source, "d.m");
        assert_eq!(
            filter_by_id(&diags, "DUPNAMEARG").len(),
            2,
            "got: {diags:?}"
        );
    }

    #[test]
    fn dupnamearg_compares_names_case_insensitively() {
        let source = "function d()\nh(name=1, Name=2)\nend\n";
        let diags = check_source(source, "d.m");
        assert_eq!(
            filter_by_id(&diags, "DUPNAMEARG").len(),
            1,
            "got: {diags:?}"
        );
    }

    #[test]
    fn dupnamearg_ok_on_distinct_names_and_string_form() {
        let source = "function d()\nh(Name=1, Other=2)\nh2('Name', 1, 'Name', 2)\nend\n";
        let diags = check_source(source, "d.m");
        assert!(
            filter_by_id(&diags, "DUPNAMEARG").is_empty(),
            "got: {diags:?}"
        );
    }
}
