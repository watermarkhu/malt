//! FCNS-19: input and output declarations and implementations of methods must match.
//!
//! For a method declared by signature, the implementation is looked up as described for
//! FCNS-18: a function of that name in the same file, or `name.m` in the `@Class` folder.
//! The two lists of inputs and the two lists of outputs must have the same names in the
//! same order; a `~` on either side matches any name. The finding points at the
//! declaration. A method with no implementation is FCNS-18's concern, and an implementation
//! without a declaration is not looked for.

use malt_core::{Config, Diagnostic, FileContext};

use super::implementations::{declarations, Implementation};
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_19";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    for declaration in declarations(ctx) {
        if let Implementation::Found(found) = &declaration.implementation {
            if !declaration.declared.matches(found) {
                out.push(finding(
                    ctx.source,
                    ID,
                    declaration.range,
                    format!(
                        "Method {} is declared as {} but implemented as {}. Make them match.",
                        declaration.name,
                        declaration.declared.describe(),
                        found.describe()
                    ),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::{run_at, Layout};
    use super::ID;

    const DECLARED: &str = "classdef C\nmethods (Static)\nout1 = getFolder(in2, in1)\nend\nend\n";

    #[test]
    fn mismatched_names_fire_at_the_declaration() {
        let src = format!("{DECLARED}function [folder, ok] = getFolder(in1, in2)\nend\n");
        let d = run_at(ID, Layout::Plain, &src, &[]);
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (3, 8));
    }

    #[test]
    fn matching_signatures_are_fine_and_tilde_matches_anything() {
        let src = "classdef C\nmethods\nr = go(obj, a)\nend\nend\nfunction r = go(~, a)\nend\n";
        assert!(run_at(ID, Layout::Plain, src, &[]).is_empty());
    }

    #[test]
    fn the_class_folder_file_is_compared() {
        let src = "classdef C\nmethods\nr = go(obj, a)\nend\nend\n";
        let bad = [("go.m", "function r = go(obj, b)\nend\n")];
        assert_eq!(run_at(ID, Layout::ClassFolder, src, &bad).len(), 1);
        let good = [("go.m", "function r = go(obj, a)\nend\n")];
        assert!(run_at(ID, Layout::ClassFolder, src, &good).is_empty());
    }

    #[test]
    fn a_missing_implementation_is_not_this_checks_concern() {
        let src = "classdef C\nmethods\nr = go(obj, a)\nend\nend\n";
        assert!(run_at(ID, Layout::Plain, src, &[]).is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run_at(
            ID,
            Layout::Plain,
            "classdef C\nmethods\nr = (\nend\nfunction go(\n",
            &[],
        );
    }
}
