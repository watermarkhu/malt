//! Turns the data files `src/data/compatibility.toml` and
//! `src/data/suggested_improvements.toml` into static Rust tables at build
//! time, so the linter does not parse half a megabyte of TOML at every start.
//!
//! Each `[[checks]]` entry becomes a struct literal of `CompatEntry` (in
//! `src/compatibility/mod.rs`) or `SuggestedEntry` (in
//! `src/suggested_improvements.rs`). The field lists below mirror those
//! structs: a field marked required must be present, any other one defaults to
//! `""`, `0`, `None`, `&[]` or `false` when absent, as `#[serde(default)]` did.

use std::fmt::Write as _;
use std::path::Path;

use toml::Value;

/// The Rust type of a field.
#[derive(Clone, Copy)]
enum Kind {
    /// `&'static str`
    Str,
    /// `usize`
    Usize,
    /// `Option<usize>`
    OptUsize,
    /// `&'static [&'static str]`
    StrList,
    /// `bool`, `false` when absent
    Bool,
    /// `bool`, `true` when absent
    BoolDefaultTrue,
}

use Kind::*;

/// (field name, type, required)
type Field = (&'static str, Kind, bool);

const COMPAT_FIELDS: &[Field] = &[
    ("id", Str, true),
    ("function_name", Str, true),
    ("severity", Str, true),
    ("message", Str, true),
    ("category", Str, true),
    ("pattern", Str, false),
    ("low_reliability", Bool, false),
    ("min_args", OptUsize, false),
    ("max_args", OptUsize, false),
    ("args_any", StrList, false),
    ("args_all", StrList, false),
    ("unless_args", StrList, false),
    ("second_arg", Str, false),
    ("second_arg_any", StrList, false),
    ("script_only", Bool, false),
    ("first_arg_text", Bool, false),
    ("first_arg_not_text", Bool, false),
    ("min_outputs", OptUsize, false),
    ("first_arg_any", StrList, false),
    ("unless_sole_arg", StrList, false),
    ("unless_all_args", StrList, false),
    ("handle", Bool, false),
    ("number_arg_at", Usize, false),
    ("pairs_from", Usize, false),
    ("function_prefix", Bool, false),
];

const SUGGESTED_FIELDS: &[Field] = &[
    ("id", Str, true),
    ("function_name", Str, true),
    ("message", Str, true),
    ("replacement", Str, true),
    ("pattern", Str, false),
    ("min_args", Usize, false),
    ("max_args", OptUsize, false),
    ("literal_args", StrList, false),
    ("literal_has_value", Bool, false),
    ("literal_at", Usize, false),
    ("literal_exact", Bool, false),
    ("function_only", Bool, false),
    ("not_assigned", Bool, false),
    ("statement_only", Bool, false),
    ("min_outputs", Usize, false),
    ("arg_at", Usize, false),
    ("arg_kinds", StrList, false),
    ("not_arg_kinds", StrList, false),
    ("arg_text", Str, false),
    ("not_with_call", Str, false),
    ("arg_through_variables", Bool, false),
    ("arg_sole_value", Bool, false),
    ("version_at_least", Str, false),
    ("within_parfor", Bool, false),
    ("not_in_call", Str, false),
    ("not_in_call_literal", Str, false),
    ("handle", BoolDefaultTrue, false),
    ("renames_call", Bool, false),
    ("rewrite", Str, false),
    ("rewrite_text", Str, false),
    ("rewrite_unsafe", Bool, false),
    ("shadowed_command", Bool, false),
    ("not_all_arg_kinds", StrList, false),
];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    generate(
        "src/data/compatibility.toml",
        "compat_data.rs",
        "COMPAT_ENTRIES",
        "CompatEntry",
        COMPAT_FIELDS,
    );
    generate(
        "src/data/suggested_improvements.toml",
        "suggested_data.rs",
        "SUGGESTED_ENTRIES",
        "SuggestedEntry",
        SUGGESTED_FIELDS,
    );
}

/// Write `static <name>: &[<ty>] = &[...];` for the `[[checks]]` of `input`
/// to `OUT_DIR/<output>`.
fn generate(input: &str, output: &str, name: &str, ty: &str, fields: &[Field]) {
    println!("cargo:rerun-if-changed={input}");
    let text = std::fs::read_to_string(input).unwrap_or_else(|e| panic!("read {input}: {e}"));
    let data: Value = toml::from_str(&text).unwrap_or_else(|e| panic!("parse {input}: {e}"));
    let checks = data
        .get("checks")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{input}: no [[checks]]"));
    let mut out = format!("static {name}: &[{ty}] = &[\n");
    for (i, check) in checks.iter().enumerate() {
        let table = check
            .as_table()
            .unwrap_or_else(|| panic!("{input}: check {i} is not a table"));
        write!(out, "    {ty} {{").unwrap();
        for &(field, kind, required) in fields {
            let value = table.get(field);
            if required && value.is_none() {
                panic!("{input}: check {i} has no `{field}`");
            }
            let bad = format!("{input}: check {i}: `{field}` has the wrong type");
            // `{:?}` of a string is a valid Rust string literal.
            let literal = match (kind, value) {
                (Str, None) => "\"\"".to_string(),
                (Str, Some(v)) => format!("{:?}", v.as_str().expect(&bad)),
                (Usize, None) => "0".to_string(),
                (Usize, Some(v)) => usize_of(v).expect(&bad).to_string(),
                (OptUsize, None) => "None".to_string(),
                (OptUsize, Some(v)) => format!("Some({})", usize_of(v).expect(&bad)),
                (StrList, None) => "&[]".to_string(),
                (StrList, Some(v)) => {
                    let items: Vec<&str> = v
                        .as_array()
                        .expect(&bad)
                        .iter()
                        .map(|item| item.as_str().expect(&bad))
                        .collect();
                    format!("&{items:?}")
                }
                (BoolDefaultTrue, None) => "true".to_string(),
                (BoolDefaultTrue, Some(v)) => v.as_bool().expect(&bad).to_string(),
                (Bool, None) => "false".to_string(),
                (Bool, Some(v)) => v.as_bool().expect(&bad).to_string(),
            };
            write!(out, " {field}: {literal},").unwrap();
        }
        out.push_str(" },\n");
    }
    out.push_str("];\n");
    let dest = Path::new(&std::env::var("OUT_DIR").unwrap()).join(output);
    std::fs::write(dest, out).unwrap_or_else(|e| panic!("write {output}: {e}"));
}

/// A non-negative TOML integer.
fn usize_of(value: &Value) -> Option<usize> {
    value.as_integer().and_then(|n| usize::try_from(n).ok())
}
