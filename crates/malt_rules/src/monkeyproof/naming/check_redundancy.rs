//! NAMING-4: names of fields and properties that contain their parent's name.

use malt_core::{Config, Diagnostic, FileContext};
use serde::Deserialize;

use super::decls::{declarations, Kind};
use crate::monkeyproof::finding;

const ID: &str = "MP_NAMING_4";

/// Parameters of `MP_NAMING_4`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Params {
    /// Parent names shorter than this are not compared, since a short name is
    /// contained in many unrelated words.
    min_parent_length: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            min_parent_length: 3,
        }
    }
}

/// Report fields written by an assignment and properties whose name contains the
/// parent's name, ignoring case.
pub(crate) fn run(config: &Config, ctx: &FileContext, out: &mut Vec<Diagnostic>) {
    let params: Params = config.rule_params(ID);
    for decl in declarations(ctx.tree.root_node(), ctx.source) {
        let (Kind::Field | Kind::Property, Some(parent)) = (decl.kind, decl.parent) else {
            continue;
        };
        if parent.chars().count() < params.min_parent_length {
            continue;
        }
        if decl.name.to_lowercase().contains(&parent.to_lowercase()) {
            let what = if decl.kind == Kind::Field {
                "field"
            } else {
                "property"
            };
            out.push(finding(
                ctx.source,
                ID,
                decl.range,
                format!(
                    "The {what} '{}' repeats the name '{parent}' of its parent; drop that part.",
                    decl.name
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
    fn field_that_repeats_the_struct_name() {
        let found = run("Beam.beamThickness = 1;\nchair.weightOfChair = 2;\n", "");
        assert_eq!(found.len(), 2);
        assert_eq!((found[0].0, found[0].1), (1, 6));
        assert_eq!((found[1].0, found[1].1), (2, 7));
    }

    #[test]
    fn unrelated_fields_are_silent() {
        assert!(run("beam.thickness = 1;\nchair.weight = 2;\n", "").is_empty());
    }

    #[test]
    fn the_immediate_parent_is_compared() {
        let found = run("a.shape.shapeId = 1;\n", "");
        assert_eq!(found.len(), 1);
        assert!(run("a.shape.sizeOfA = 1;\n", "").is_empty());
    }

    #[test]
    fn indexed_parent_counts() {
        assert_eq!(run("beam(2).beamLength = 1;\n", "").len(), 1);
    }

    #[test]
    fn reading_a_field_is_not_reported() {
        assert!(run("x = Beam.beamThickness;\n", "").is_empty());
    }

    #[test]
    fn property_that_repeats_the_class_name() {
        let src =
            "classdef Beam\n    properties\n        beamWidth\n        height\n    end\nend\n";
        let found = run(src, "");
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].0, found[0].1), (3, 9));
    }

    #[test]
    fn short_parent_names_are_skipped() {
        assert!(run("a.data = 1;\n", "").is_empty());
        let toml = "[lint.rules.MP_NAMING_4]\nmin_parent_length = 1\n";
        assert_eq!(run("a.data = 1;\n", toml).len(), 1);
    }

    #[test]
    fn dynamic_fields_are_skipped() {
        assert!(run("beam.('beamX') = 1;\n", "").is_empty());
    }

    #[test]
    fn broken_code_does_not_panic() {
        run("beam.beamX = \ns.\n", "");
        run("classdef\n properties\n", "");
    }
}
