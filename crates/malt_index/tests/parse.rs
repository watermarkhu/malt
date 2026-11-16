//! The file parser: every entity kind, the arguments forms, attributes, placement of
//! docstrings, and malformed input. The samples are small MATLAB texts written here.

use std::path::Path;

use malt_index::{parse_source, Argument, ArgumentKind, Entity, Kind, ParserConfig};

fn parse(src: &str) -> Entity {
    parse_source(
        Path::new("/x/Sample.m"),
        src.as_bytes(),
        &ParserConfig::default(),
    )
    .expect("parse")
}

fn parse_with(src: &str, config: ParserConfig) -> Entity {
    parse_source(Path::new("/x/Sample.m"), src.as_bytes(), &config).expect("parse")
}

fn arg_names(args: &[Argument]) -> Vec<&str> {
    args.iter().map(|a| a.name.as_str()).collect()
}

fn arg<'a>(args: &'a [Argument], name: &str) -> &'a Argument {
    args.iter()
        .find(|a| a.name == name)
        .unwrap_or_else(|| panic!("no argument {name} in {:?}", arg_names(args)))
}

fn function(entity: &Entity) -> &malt_index::FunctionDetail {
    entity.function().expect("a function")
}

const SHAPES: &str = "function [area, varargout] = Shapes(kind, scale, varargin)
% SHAPES Area of a named shape.
%   Long description of the function.
arguments
    kind (1,1) string {mustBeNonempty}
    scale (1,:) double = 1
    varargin
    opts.Units char = 'm'
    opts.?Other.Prop
end
arguments (Output)
    area double
    varargout
end
area = scale;
end
";

#[test]
fn function_file_takes_the_file_name_and_its_span() {
    let e = parse("function r = Sample(a)\nr = a;\nend\n");
    assert_eq!(e.kind, Kind::Function);
    assert_eq!(e.name, "Sample");
    assert_eq!((e.lineno, e.endlineno), (Some(1), Some(3)));
    assert_eq!(e.path.as_deref(), Some(Path::new("/x/Sample.m")));
    assert!(e.attributes.is_empty());
}

#[test]
fn arguments_block_forms_are_read_as_maxx_reads_them() {
    let e = parse(SHAPES);
    let f = function(&e);
    assert_eq!(
        arg_names(&f.arguments),
        ["kind", "scale", "varargin", "Units", "opts"]
    );

    let kind = arg(&f.arguments, "kind");
    assert_eq!(kind.kind, ArgumentKind::PositionalOnly);
    assert_eq!(kind.type_.as_deref(), Some("string"));
    assert_eq!(
        kind.dimensions.as_deref(),
        Some(&["1".to_string(), "1".to_string()][..])
    );
    assert_eq!(kind.validators.as_deref(), Some("{mustBeNonempty}"));
    assert_eq!(kind.default, None);

    let scale = arg(&f.arguments, "scale");
    assert_eq!(scale.kind, ArgumentKind::Optional);
    assert_eq!(
        scale.dimensions.as_deref(),
        Some(&["1".to_string(), ":".to_string()][..])
    );
    assert_eq!(scale.default.as_deref(), Some("1"));
}

#[test]
fn varargin_is_its_own_kind_and_name_value_arguments_are_keyword_only() {
    let f = parse(SHAPES);
    let f = function(&f);
    assert_eq!(arg(&f.arguments, "varargin").kind, ArgumentKind::Varargin);

    let units = arg(&f.arguments, "Units");
    assert_eq!(units.kind, ArgumentKind::KeywordOnly);
    assert_eq!(units.type_.as_deref(), Some("char"));
    assert_eq!(units.default.as_deref(), Some("'m'"));

    // `opts.?Other.Prop`: the class path is the argument's type.
    let opts = arg(&f.arguments, "opts");
    assert_eq!(opts.kind, ArgumentKind::KeywordOnly);
    assert_eq!(opts.type_.as_deref(), Some("Other.Prop"));
}

#[test]
fn output_attribute_moves_arguments_to_the_returns() {
    let f = parse(SHAPES);
    let f = function(&f);
    assert_eq!(arg_names(&f.returns), ["area", "varargout"]);
    assert_eq!(arg(&f.returns, "area").type_.as_deref(), Some("double"));
    assert_eq!(
        arg(&f.returns, "varargout").kind,
        ArgumentKind::PositionalOnly
    );
    assert!(!arg_names(&f.arguments).contains(&"area"));
}

#[test]
fn outputs_and_ignored_inputs_are_all_kept() {
    let e = parse("function [a, b, ~] = Multi(x, ~, y)\na = x; b = y;\nend\n");
    let f = function(&e);
    assert_eq!(arg_names(&f.returns), ["a", "b", "~"]);
    assert_eq!(arg_names(&f.arguments), ["x", "~", "y"]);
}

#[test]
fn optional_input_with_default_in_the_signature_block() {
    let e = parse("function r = Opt(a, b)\narguments\n    a double = 2\nend\nr = a + b;\nend\n");
    let f = function(&e);
    assert_eq!(arg(&f.arguments, "a").kind, ArgumentKind::Optional);
    assert_eq!(arg(&f.arguments, "a").default.as_deref(), Some("2"));
    assert_eq!(arg(&f.arguments, "b").kind, ArgumentKind::PositionalOnly);
}

#[test]
fn docstring_of_a_function_is_the_comment_after_the_signature() {
    let e = parse(SHAPES);
    let doc = e.docstring.expect("docstring");
    assert_eq!(
        doc.value,
        "SHAPES Area of a named shape.\nLong description of the function."
    );
    assert_eq!((doc.lineno, doc.endlineno), (2, 3));
    assert!(doc.raw.starts_with("% SHAPES"));
}

#[test]
fn argument_comment_documents_the_previous_argument_by_default() {
    let e =
        parse("function r = F(a)\narguments\n    a double\n    % the a input\nend\nr = a;\nend\n");
    let f = function(&e);
    assert_eq!(
        arg(&f.arguments, "a")
            .docstring
            .as_ref()
            .map(|d| d.value.as_str()),
        Some("the a input")
    );
}

#[test]
fn getters_and_setters_attach_to_the_property_and_are_not_members() {
    let src = "classdef Sample\n properties\n  Size double\n end\n methods\n  function r = get.Size(obj)\n   r = 1;\n  end\n  function set.Size(obj, v)\n  end\n end\nend\n";
    let e = parse(src);
    assert_eq!(e.kind, Kind::Class);
    let prop = &e.members["Size"];
    let detail = prop.property().expect("property");
    assert!(detail
        .getter
        .as_ref()
        .is_some_and(|g| g.function().unwrap().is_getter));
    assert!(detail
        .setter
        .as_ref()
        .is_some_and(|s| s.function().unwrap().is_setter));
    assert_eq!(e.members.len(), 1);
}

#[test]
fn getter_without_a_property_is_a_member_named_after_the_property() {
    let src =
        "classdef Sample\n methods\n  function r = get.Lone(obj)\n   r = 1;\n  end\n end\nend\n";
    let e = parse(src);
    let lone = &e.members["Lone"];
    assert_eq!(lone.kind, Kind::Function);
    assert!(function(lone).is_getter);
}

#[test]
fn method_self_argument_is_removed_unless_static_or_constructor() {
    let src = "classdef Sample\n methods\n  function obj = Sample(v)\n  end\n  function show(obj, x, y)\n  end\n  end\n methods (Static, Access = protected)\n  function r = make(x)\n  end\n end\nend\n";
    let e = parse(src);
    assert_eq!(arg_names(&function(&e.members["Sample"]).arguments), ["v"]);
    assert_eq!(
        arg_names(&function(&e.members["show"]).arguments),
        ["x", "y"]
    );
    let make = &e.members["make"];
    assert_eq!(arg_names(&function(make).arguments), ["x"]);
    assert!(make.attributes.contains("Static"));
    assert!(make.attributes.contains("Access=protected"));
}

#[test]
fn class_attributes_bases_and_property_attributes() {
    let src = "classdef (Sealed) Widget < handle & matlab.mixin.Copyable\n properties (Access = private, Constant)\n  Count (1,1) double = 3\n end\n properties (Dependent)\n  Size\n end\nend\n";
    let e = parse(src);
    assert_eq!(e.bases(), ["handle", "matlab.mixin.Copyable"]);
    assert!(e.attributes.contains("Sealed"));
    let count = &e.members["Count"];
    assert_eq!(count.kind, Kind::Property);
    assert!(count.attributes.contains("Constant"));
    assert!(count.attributes.contains("Access=private"));
    assert!(e.members["Size"].attributes.contains("Dependent"));
    let detail = count.property().expect("property");
    assert_eq!(detail.type_.as_deref(), Some("double"));
    assert_eq!(
        detail.dimensions.as_deref(),
        Some(&["1".to_string(), "1".to_string()][..])
    );
    assert_eq!(detail.default.as_deref(), Some("3"));
}

#[test]
fn property_type_joins_every_identifier_as_maxx_does() {
    let e = parse("classdef S\n properties\n  idx@uint8 scalar = 0;\n end\nend\n");
    assert_eq!(
        e.members["idx"].property().unwrap().type_.as_deref(),
        Some("uint8scalar")
    );
}

#[test]
fn enumeration_members_take_the_comment_after_them_by_default() {
    let src =
        "classdef Colours\n enumeration\n  Red (1)\n  % the red one\n  Green (2)\n end\nend\n";
    let e = parse(src);
    assert_eq!(e.members["Red"].kind, Kind::Enumeration);
    assert_eq!(
        e.members["Red"].docstring.as_ref().unwrap().value,
        "the red one"
    );
    assert!(e.members["Green"].docstring.is_none());
    assert_eq!(
        e.members["Red"]
            .enumeration_value()
            .map(|v| v.text.as_str()),
        Some("1")
    );
}

#[test]
fn docstring_placement_before_is_configurable() {
    let src = "classdef Colours\n enumeration\n  Red (1)\n  % the green one\n  Green (2)\n end\n properties\n  % count\n  Count\n  Name\n end\nend\n";
    let before = parse_with(
        src,
        ParserConfig {
            docstring_before_enumerations: true,
            docstring_before_properties: true,
            ..ParserConfig::default()
        },
    );
    assert_eq!(
        before.members["Green"].docstring.as_ref().unwrap().value,
        "the green one"
    );
    assert!(before.members["Red"].docstring.is_none());
    assert_eq!(
        before.members["Count"].docstring.as_ref().unwrap().value,
        "count"
    );
    assert!(before.members["Name"].docstring.is_none());
}

#[test]
fn block_comment_and_cell_title_are_docstrings() {
    let e = parse("function r = F(a)\n%{\nFirst line.\n  Indented.\n%}\n%% Title\nr = a;\nend\n");
    let value = e.docstring.expect("docstring").value;
    assert_eq!(value, "First line.\n  Indented.\nTitle");
}

#[test]
fn local_and_nested_functions_are_members() {
    let e = parse("function out = outer(a)\nout = inner(a);\nfunction y = inner(b)\ny = b;\nend\nend\nfunction z = local(c)\nz = c;\nend\n");
    assert_eq!(e.kind, Kind::Function);
    assert_eq!(e.name, "Sample");
    assert!(e.members.contains_key("local"));
    let nested = &e.members["inner"];
    assert_eq!(arg_names(&function(nested).arguments), ["b"]);
}

#[test]
fn nested_function_inside_a_function_body_is_a_member() {
    let src = "function r = Nested(c)\nr = inner();\n    function v = inner()\n        v = 1;\n    end\nend\n";
    let e = parse(src);
    assert_eq!(arg_names(&function(&e).arguments), ["c"]);
    assert_eq!(arg_names(&function(&e).returns), ["r"]);
    assert_eq!(function(&e.members["inner"]).returns[0].name, "v");
}

#[test]
fn events_block_is_not_an_entity() {
    let src = "classdef Sample\n events\n  Changed\n end\n properties\n  X\n end\nend\n";
    let e = parse(src);
    assert_eq!(
        e.members.keys().map(String::as_str).collect::<Vec<_>>(),
        ["X"]
    );
}

#[test]
fn a_file_without_declaration_is_a_script_with_the_header_as_docstring() {
    let e = parse("% SCRIPT Does the thing.\nx = 1;\ndisp(x)\n");
    assert_eq!(e.kind, Kind::Script);
    assert_eq!(e.lineno, None);
    assert_eq!(e.docstring.unwrap().value, "SCRIPT Does the thing.");
}

#[test]
fn header_comment_documents_a_function_without_its_own_comment() {
    let e = parse("% Licence text.\nfunction r = F(a)\nr = a;\nend\n");
    assert_eq!(e.docstring.unwrap().value, "Licence text.");
}

#[test]
fn byte_order_mark_crlf_and_latin1_are_read() {
    let crlf = "\u{feff}function r = bom(a)\r\n% doc line\r\nr = a;\r\nend\r\n";
    let e = parse(crlf);
    assert_eq!(e.kind, Kind::Function);
    assert_eq!(e.docstring.unwrap().value, "doc line");

    let latin1: &[u8] = b"function r = lat(a)\n% R\xe9sum\xe9 du fichier\nr = a;\nend\n";
    let e =
        parse_source(Path::new("/x/Sample.m"), latin1, &ParserConfig::default()).expect("parse");
    assert_eq!(e.docstring.unwrap().value, "R\u{e9}sum\u{e9} du fichier");
}

#[test]
fn malformed_file_gives_an_entity_and_does_not_panic() {
    for src in [
        "function r = Sample(a, b\n% missing paren\nr = a + ;\nif (b\n",
        "",
        "classdef Sample < \n properties\n  X = [1 2\n end\n",
        "%{\nunterminated block comment",
        "function\n",
        "arguments\n x double\nend\n",
    ] {
        let e = parse(src);
        assert_eq!(e.name, "Sample");
    }
}

/// An abstract method is a signature with no body, not a `function_definition`: it is
/// still a method of the class (MATLAB lists it, found by the cross-check against MATLAB).
#[test]
fn abstract_method_signatures_are_methods() {
    let src = "classdef (Abstract) Base < handle\n\
               methods (Abstract)\n\
               out = doit(obj, x)\n\
               end\n\
               methods (Abstract, Static)\n\
               r = make(x)\n\
               end\n\
               end\n";
    let class = parse(src);
    assert_eq!(class.kind, Kind::Class);
    let doit = &class.members["doit"];
    assert_eq!(doit.kind, Kind::Function);
    assert!(doit.attributes.contains("Abstract"));
    let detail = function(&class.members["doit"]);
    assert_eq!(
        arg_names(&detail.arguments),
        vec!["x"],
        "the object is dropped"
    );
    assert_eq!(arg_names(&detail.returns), vec!["out"]);
    let make = &class.members["make"];
    assert!(make.attributes.contains("Static"));
    assert_eq!(arg_names(&function(make).arguments), vec!["x"]);
}

/// `Access` with a class list is `restricted` (MATLAB's metadata says so), not `private`.
#[test]
fn access_restricted_to_classes_is_restricted() {
    let src = "classdef Msg < handle\n\
               methods (Access = {?Logger, ?Other})\n\
               function out = build(obj)\n\
               out = 1;\n\
               end\n\
               end\n\
               methods (Access = ?Logger)\n\
               function out = single(obj)\n\
               out = 2;\n\
               end\n\
               end\n\
               methods (Access = protected)\n\
               function out = guarded(obj)\n\
               out = 3;\n\
               end\n\
               end\n\
               end\n";
    let class = parse(src);
    assert!(class.members["build"]
        .attributes
        .contains("Access=restricted"));
    assert!(class.members["single"]
        .attributes
        .contains("Access=restricted"));
    assert!(class.members["guarded"]
        .attributes
        .contains("Access=protected"));
}

#[test]
fn events_are_named_in_source_order_with_or_without_attributes() {
    let class = parse(
        "classdef Sample < handle
    events
        Changed
        Moved
    end
    events (ListenAccess = private)
        Internal
    end
end
",
    );
    assert_eq!(class.kind, Kind::Class);
    assert_eq!(class.events(), ["Changed", "Moved", "Internal"]);
    assert!(parse("function f()\nend\n").events().is_empty());
    assert!(parse("classdef Plain\nend\n").events().is_empty());
}

#[test]
fn declares_constructor_is_a_method_named_as_the_class() {
    let with = parse(
        "classdef Sample
    methods
        function obj = Sample(x)
        end
        function y = bump(obj)
        end
    end
end
",
    );
    assert!(with.declares_constructor());
    let without = parse(
        "classdef Sample
    methods
        function y = Sampler(obj)
        end
    end
end
",
    );
    assert!(!without.declares_constructor());
    assert!(!parse("classdef Sample\nend\n").declares_constructor());
    assert!(!parse("function Sample()\nend\n").declares_constructor());
}
