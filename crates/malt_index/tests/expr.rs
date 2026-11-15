//! Expressions: the text of a type, default, validator, base or enumeration value, with
//! the span it was read from.

use std::path::Path;

use malt_index::{parse_source, ArgumentKind, Expr, ParserConfig};

const SRC: &str = "\
classdef Shape < handle & matlab.mixin.Copyable
    properties
        Units (1,:) char {mustBeMember(Units,{'m','cm'})} = 'm'
        idx@uint8 scalar
    end
    enumeration
        Red (1)
        Blue
    end
    methods
        function out = area(obj, opts)
            arguments
                obj
                opts.?Other.Prop
            end
            out = 1;
        end
    end
end
";

fn parse(src: &str) -> malt_index::Entity {
    parse_source(
        Path::new("Shape.m"),
        src.as_bytes(),
        &ParserConfig::default(),
    )
    .unwrap()
}

#[test]
fn property_type_default_and_validators_keep_their_span() {
    let e = parse(SRC);
    let units = e.members["Units"].property().unwrap();
    let ty: &Expr = units.type_.as_ref().unwrap();
    assert_eq!(ty.text, "char");
    assert_eq!(
        (ty.lineno, ty.column, ty.endlineno, ty.endcolumn),
        (3, 20, 3, 24)
    );

    let validators = units.validators.as_ref().unwrap();
    assert_eq!(validators.text, "{mustBeMember(Units,{'m','cm'})}");
    assert_eq!((validators.lineno, validators.column), (3, 25));

    let default = units.default.as_ref().unwrap();
    assert_eq!(default.text, "'m'");
    assert_eq!(default.lineno, 3);
    assert_eq!(default.column, validators.endcolumn + 3);
}

#[test]
fn an_expression_derefs_to_its_text_and_compares_with_strings() {
    let e = parse(SRC);
    let units = e.members["Units"].property().unwrap();
    assert_eq!(units.type_.as_deref(), Some("char"));
    assert_eq!(units.type_.as_ref().unwrap(), "char");
    assert!(units.type_.as_ref().unwrap().starts_with("ch"));
}

#[test]
fn a_type_of_several_identifiers_is_joined_as_maxx_does() {
    let e = parse(SRC);
    let ty = e.members["idx"].property().unwrap().type_.as_ref().unwrap();
    assert_eq!(ty.text, "uint8scalar");
    assert_eq!(ty.lineno, 4);
    assert_eq!(ty.column, 12);
    assert_eq!(ty.endcolumn, 12 + "uint8 scalar".len());
}

#[test]
fn bases_keep_their_span() {
    let e = parse(SRC);
    let bases = e.bases();
    assert_eq!(bases.len(), 2);
    assert_eq!(bases[0].text, "handle");
    assert_eq!((bases[0].lineno, bases[0].column), (1, 17));
    assert_eq!(bases[1].text, "matlab.mixin.Copyable");
    assert_eq!((bases[1].lineno, bases[1].column), (1, 26));
    assert_eq!(bases[1].endcolumn, 26 + "matlab.mixin.Copyable".len());
}

#[test]
fn enumeration_values_are_expressions_when_written() {
    let e = parse(SRC);
    assert_eq!(
        e.members["Red"]
            .enumeration_value()
            .map(|v| v.text.as_str()),
        Some("1")
    );
    assert!(e.members["Blue"].enumeration_value().is_none());
}

#[test]
fn name_value_argument_type_spans_the_class_path() {
    let e = parse(SRC);
    let area = e.members["area"].function().unwrap();
    let opts = area
        .arguments
        .iter()
        .find(|a| a.name == "opts")
        .expect("opts argument");
    assert_eq!(opts.kind, ArgumentKind::KeywordOnly);
    assert_eq!(opts.type_.as_ref().unwrap().text, "Other.Prop");
}

#[test]
fn an_expression_is_written_as_its_text_in_json() {
    let e = parse(SRC);
    let units = e.members["Units"].property().unwrap();
    assert_eq!(serde_json::to_value(&units.type_).unwrap(), "char");
}
