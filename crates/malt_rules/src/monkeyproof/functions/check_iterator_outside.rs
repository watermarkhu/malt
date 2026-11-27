//! FCNS-21: do not use iterator variables outside the for loop.
//!
//! Reports a read of a loop variable after its `for` or `parfor` loop has ended, when no
//! assignment (or other loop over the same name) comes between the loop and the read in
//! the source. Each read is reported once. An assignment in a branch counts as a
//! reassignment, so the check errs on the silent side. Reads inside the loop, including
//! its own header, are fine.

use std::collections::HashSet;

use malt_core::{Config, Diagnostic, FileContext};

use crate::analysis::symbols::{DefKind, SymbolTable};
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_21";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let root = ctx.tree.root_node();
    let table = SymbolTable::shared(|| root, ctx.source);
    let mut reported = HashSet::new();
    for scope in &table.scopes {
        for def in scope.defs.iter().filter(|d| d.kind == DefKind::ForIterator) {
            let Some(loop_end) = root
                .descendant_for_byte_range(def.byte_range.start, def.byte_range.end)
                .and_then(|n| {
                    let mut node = n;
                    while node.kind() != "for_statement" {
                        node = node.parent()?;
                    }
                    Some(node.end_byte())
                })
            else {
                continue;
            };
            for read in scope
                .uses_of(&def.name)
                .into_iter()
                .filter(|u| u.byte_range.start >= loop_end)
            {
                let reassigned = scope.defs_of(&def.name).iter().any(|d| {
                    !d.partial
                        && d.effect_start >= loop_end
                        && d.effect_start <= read.byte_range.start
                });
                if reassigned || !reported.insert(read.byte_range.start) {
                    continue;
                }
                out.push(finding(
                    ctx.source,
                    ID,
                    read.byte_range.clone(),
                    format!(
                        "The loop variable {} is read after its loop. Store the value you need in another variable inside the loop.",
                        def.name
                    ),
                ));
            }
        }
    }
    out.sort_by_key(|d| d.byte_range.start);
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::run;
    use super::ID;

    #[test]
    fn a_read_after_the_loop_fires_at_the_read() {
        let src = "function f(n)\nfor i = 1:n\nif i > 2\nbreak\nend\nend\nv = i + 10;\nend\n";
        let d = run(ID, src, "");
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (7, 5));
    }

    #[test]
    fn reads_inside_the_loop_are_fine() {
        let src = "function f(n)\nfor i = 1:n\nx = i;\nend\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn a_reassignment_after_the_loop_is_fine() {
        let src = "function f(n)\nfor i = 1:n\nend\ni = 0;\nv = i;\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn a_later_loop_over_the_same_name_is_fine() {
        let src = "function f(n)\nfor i = 1:n\nend\nfor i = 1:n\nx = i;\nend\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn another_function_and_other_names_are_fine() {
        let src = "function f(n)\nfor i = 1:n\nend\nv = j;\nend\nfunction g(i)\nv = i;\nend\n";
        assert!(run(ID, src, "").is_empty());
    }

    #[test]
    fn each_read_is_reported_and_scripts_work() {
        let src = "for k = 1:3\nend\na = k;\nb = k;\n";
        assert_eq!(run(ID, src, "").len(), 2);
    }

    #[test]
    fn broken_code_does_not_panic() {
        run(ID, "for i = 1:\nx = i\n", "");
    }
}
