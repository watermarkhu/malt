//! FCNS-4: do not use scripts.
//!
//! A file is a script when its first piece of code is neither a `function` nor a
//! `classdef`. The finding points at that first statement. Files with only comments, and
//! files whose first statement does not parse, are left alone. The check does not know
//! whether a script is a deliberate project initialisation.

use malt_core::{Config, Diagnostic, FileContext};

use super::common::head_range;
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_4";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let root = ctx.tree.root_node();
    let mut cursor = root.walk();
    let first = root
        .named_children(&mut cursor)
        .find(|c| c.kind() != "comment");
    let Some(first) = first else { return };
    if matches!(
        first.kind(),
        "function_definition" | "class_definition" | "ERROR"
    ) {
        return;
    }
    out.push(finding(
        ctx.source,
        ID,
        head_range(first, ctx.source),
        "This file is a script. Write it as a function instead.",
    ));
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::run;
    use super::ID;

    #[test]
    fn a_script_fires_at_its_first_statement() {
        let d = run(ID, "% header\n\nx = 1;\ny = 2;\n", "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (3, 1));
    }

    #[test]
    fn function_and_class_files_are_fine() {
        assert!(run(ID, "% c\nfunction f()\nend\n", "").is_empty());
        assert!(run(ID, "classdef C\nend\n", "").is_empty());
    }

    #[test]
    fn a_script_with_local_functions_is_still_a_script() {
        assert_eq!(run(ID, "f();\nfunction f()\nend\n", "").len(), 1);
    }

    #[test]
    fn empty_and_comment_only_files_are_fine() {
        assert!(run(ID, "", "").is_empty());
        assert!(run(ID, "% nothing\n", "").is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(ID, "x = (\n", "");
    }
}
