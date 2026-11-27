//! FCNS-18: methods declared in a classdef file must be implemented.
//!
//! A signature in a `methods` block that is not `Abstract` needs an implementation. The
//! check looks for it in the file itself (a function of that name after the class, as the
//! standard's examples show) and in the class folder, when the file sits in an `@Class`
//! folder (`@Class/name.m`). It reports a declaration with no implementation in either
//! place. Nothing is reported when the file is not on disk, or when the folder holds a
//! `.p`, `.mlx` or mex file of that name. The reverse, an implementation without a
//! declaration, is not checked: a file in the class folder or a local function is not
//! necessarily meant as a method.

use malt_core::{Config, Diagnostic, FileContext};

use super::implementations::{declarations, Implementation};
use crate::monkeyproof::finding;

const ID: &str = "MP_FCNS_18";

/// Run the check over a file.
pub(crate) fn run(_config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    for declaration in declarations(ctx) {
        if matches!(declaration.implementation, Implementation::Missing) {
            out.push(finding(
                ctx.source,
                ID,
                declaration.range,
                format!(
                    "Method {} is declared but not implemented. Add its implementation or make it abstract.",
                    declaration.name
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::tests_util::{run_at, Layout};
    use super::ID;

    const DECLARED: &str = "classdef C\nmethods\nr = go(obj, a)\nend\nend\n";

    #[test]
    fn a_declaration_without_implementation_fires_at_the_name() {
        let d = run_at(ID, Layout::Plain, DECLARED, &[]);
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].line, d[0].column), (3, 5));
    }

    #[test]
    fn a_local_function_or_a_class_folder_file_implements_it() {
        let local = format!("{DECLARED}function r = go(obj, a)\nend\n");
        assert!(run_at(ID, Layout::Plain, &local, &[]).is_empty());
        let files = [("go.m", "function r = go(obj, a)\nend\n")];
        assert!(run_at(ID, Layout::ClassFolder, DECLARED, &files).is_empty());
        assert_eq!(run_at(ID, Layout::ClassFolder, DECLARED, &[]).len(), 1);
    }

    #[test]
    fn abstract_methods_and_full_definitions_are_fine() {
        let abstract_class = "classdef C\nmethods (Abstract)\nr = go(obj)\nend\nend\n";
        assert!(run_at(ID, Layout::Plain, abstract_class, &[]).is_empty());
        let full = "classdef C\nmethods\nfunction r = go(obj)\nend\nend\nend\n";
        assert!(run_at(ID, Layout::Plain, full, &[]).is_empty());
    }

    #[test]
    fn an_unknown_file_location_is_not_reported() {
        // The shared lint helper uses a path that does not exist on disk.
        let d = super::super::common::tests_util::run(ID, DECLARED, "");
        assert!(d.is_empty(), "{d:?}");
    }

    #[test]
    fn broken_code_does_not_panic() {
        run_at(ID, Layout::Plain, "classdef C\nmethods\nr = (\n", &[]);
    }
}
