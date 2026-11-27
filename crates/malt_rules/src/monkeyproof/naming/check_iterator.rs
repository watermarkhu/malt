//! NAMING-2: iterator variables of for loops.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;
use tree_sitter::Node;

use super::decls::text;
use crate::monkeyproof::finding;

const ID: &str = "MP_NAMING_2";

/// Parameters of `MP_NAMING_2`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    /// The shortest iterator name.
    min_length: usize,
    /// The first letter of the outermost loop's iterator; each nested loop takes the
    /// next letter of the alphabet.
    first_letter: String,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            min_length: 3,
            first_letter: "i".to_string(),
        }
    }
}

/// Report iterators that are too short or do not follow the letter sequence.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    let first = params
        .first_letter
        .chars()
        .next()
        .filter(char::is_ascii_alphabetic)
        .map(|c| c.to_ascii_lowercase())
        .unwrap_or('i');
    visit(ctx.tree.root_node(), ctx.source, 0, first, &params, out);
}

fn visit(
    node: Node,
    source: &str,
    depth: u32,
    first: char,
    params: &Params,
    out: &mut Vec<Diagnostic>,
) {
    let mut depth = depth;
    match node.kind() {
        "function_definition" => depth = 0,
        "for_statement" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "iterator" {
                    if let Some(variable) = child.named_child(0) {
                        if variable.kind() == "identifier" {
                            check(variable, source, depth, first, params, out);
                        }
                    }
                }
            }
            depth += 1;
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, source, depth, first, params, out);
    }
}

fn check(
    variable: Node,
    source: &str,
    depth: u32,
    first: char,
    params: &Params,
    out: &mut Vec<Diagnostic>,
) {
    let name = text(variable, source);
    if name.chars().count() < params.min_length {
        out.push(finding(
            source,
            ID,
            variable.byte_range(),
            format!(
                "Loop variable '{name}' is shorter than {} characters; use a descriptive name such as 'iItem'.",
                params.min_length
            ),
        ));
    }
    let expected = char::from_u32(first as u32 + depth).filter(char::is_ascii_lowercase);
    if let (Some(expected), Some(actual)) = (expected, name.chars().next()) {
        if actual.to_ascii_lowercase() != expected {
            out.push(finding(
                source,
                ID,
                variable.byte_range(),
                format!(
                    "Loop variable '{name}' should start with '{expected}' at this nesting depth."
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::monkeyproof::lint;
    use crate::monkeyproof::naming::from_config;

    fn run(src: &str, toml: &str) -> Vec<(usize, usize, String)> {
        lint(from_config, src, toml)
            .into_iter()
            .filter(|d| d.rule_id == super::ID)
            .map(|d| (d.line, d.column, d.message))
            .collect()
    }

    #[test]
    fn nested_loops_follow_the_alphabet() {
        let src = "for iNode = 1:3\n    for jPoint = 1:2\n        for kStep = 1:2\n        end\n    end\nend\n";
        assert!(run(src, "").is_empty());
    }

    #[test]
    fn short_name_and_wrong_letter_are_reported_at_the_iterator() {
        let src = "for nodeIdx = 1:3\n    for j = 1:2\n    end\nend\n";
        let found = run(src, "");
        // nodeIdx: wrong letter. j: too short.
        assert_eq!(found.len(), 2);
        assert_eq!((found[0].0, found[0].1), (1, 5));
        assert!(found[0].2.contains("'i'"));
        assert_eq!((found[1].0, found[1].1), (2, 9));
        assert!(found[1].2.contains("shorter"));
    }

    #[test]
    fn inner_loop_must_take_the_next_letter() {
        let src = "for iNode = 1:3\n    for iPoint = 1:2\n    end\nend\n";
        let found = run(src, "");
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].0, found[0].1), (2, 9));
        assert!(found[0].2.contains("'j'"));
    }

    #[test]
    fn sibling_loops_both_start_at_the_first_letter() {
        let src = "for iAaa = 1:3\nend\nfor iBbb = 1:3\nend\n";
        assert!(run(src, "").is_empty());
    }

    #[test]
    fn depth_restarts_in_a_function() {
        let src = "function f\n    for iAaa = 1:3\n    end\nend\nfunction g\n    for iBbb = 1:3\n    end\nend\n";
        assert!(run(src, "").is_empty());
    }

    #[test]
    fn loop_inside_an_if_counts_as_nested() {
        let src =
            "for iAaa = 1:3\n    if true\n        for iBbb = 1:2\n        end\n    end\nend\n";
        assert_eq!(run(src, "").len(), 1);
    }

    #[test]
    fn parameters_are_respected() {
        let src = "for k = 1:3\nend\n";
        assert_eq!(run(src, "").len(), 2);
        let toml = "[lint.rules.MP_NAMING_2]\nmin_length = 1\nfirst_letter = \"k\"\n";
        assert!(run(src, toml).is_empty());
    }

    #[test]
    fn uppercase_first_letter_is_accepted() {
        assert!(run("for INode = 1:3\nend\n", "").is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run("for = 1:3\n  for x\n", "");
        run("for iAaa = \n", "");
    }
}
