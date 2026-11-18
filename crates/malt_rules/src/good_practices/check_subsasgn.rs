//! SUBSASGN: a `subsasgn` call used as a statement, which discards its updated
//! value. Probed: `subsasgn(s, ...);` fires at the call. Silent: an assigned
//! result, and `subsref`.

use super::*;

impl GoodPracticesEngine {
    /// SUBSASGN: `subsasgn` called as a statement, so its output is never assigned.
    pub(crate) fn check_subsasgn(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("SUBSASGN")
            || node.kind() != "function_call"
            || get_function_call_name(node, source) != Some("subsasgn")
        {
            return Vec::new();
        }
        let is_statement = node
            .parent()
            .is_some_and(|p| matches!(p.kind(), "source_file" | "block" | "lambda"));
        if !is_statement {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: "SUBSASGN",
            message: "Output of 'subsasgn' should be assigned to a variable.".to_string(),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::has_id;

    /// Run SUBSASGN on every node of `src`.
    fn subsasgn_diags(src: &str) -> Vec<Diagnostic> {
        let tree = parse(src);
        let eng = engine();
        let mut out = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            out.extend(eng.check_subsasgn(n, src));
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        out
    }

    #[test]
    fn subsasgn_fires_on_unassigned_statement() {
        let diags = subsasgn_diags("s = struct('a', 1);\nsubsasgn(s, substruct('.', 'a'), 2);\n");
        assert!(has_id(&diags, "SUBSASGN"), "got: {diags:?}");
        assert_eq!((diags[0].line, diags[0].column), (2, 1));
    }

    #[test]
    fn subsasgn_silent_when_assigned_or_subsref() {
        for src in [
            "s = subsasgn(s, substruct('.', 'a'), 2);\n",
            "subsref(s, substruct('.', 'a'));\n",
        ] {
            assert!(!has_id(&subsasgn_diags(src), "SUBSASGN"), "{src}");
        }
    }
}

#[cfg(test)]
mod probe_p2a_tests {
    use crate::test_util::lint_nodes;
    use malt_core::Config;

    /// Lines of the `id` diagnostics that the default (mathworks) engine reports for `src`.
    fn lines(src: &str, id: &str) -> Vec<usize> {
        let e = crate::good_practices::GoodPracticesEngine::from_config(&Config::default());
        lint_nodes(&*e, src)
            .iter()
            .filter(|d| d.rule_id == id)
            .map(|d| d.line)
            .collect()
    }

    #[test]
    fn subsasgn_is_reported_as_an_anonymous_function_body_too() {
        // Probed: `f = @() subsasgn(...)` (167) is reported; an assigned result is not.
        let lambda = "s = struct();\nf = @() subsasgn(s, substruct('.', 'a'), 1);\n";
        let assigned = "s = struct();\ns = subsasgn(s, substruct('.', 'a'), 1);\n";
        assert_eq!(lines(lambda, "SUBSASGN"), vec![2]);
        assert!(lines(assigned, "SUBSASGN").is_empty());
    }
}
