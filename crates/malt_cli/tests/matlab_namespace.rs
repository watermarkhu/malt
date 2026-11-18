//! The path index cross-checked against MATLAB, on the same folders.
//!
//! Two phases. Generation: `tests/matlab/generate_namespace_facts.m` runs `matlab_facts.m`
//! in MATLAB and writes a JSON file with what MATLAB says about every name of some folders:
//! the file it resolves to (and `which -all`), the signature (`nargin`, `nargout`), and for
//! classes the superclasses, properties, methods, events and enumeration members. No project
//! code runs. Comparison (this test): the facts file (`MALT_MATLAB_FACTS`), or MATLAB started
//! here to generate it the same way. CI generates the file with `matlab-actions/run-command`
//! because MATLAB is only licensed there through that action. The same folders are read by
//! `malt_index`, and the two are compared in both directions:
//!
//! - every name the index lists is a name MATLAB lists, and resolves to the same file;
//! - every name MATLAB's `what` lists is in the index;
//! - the facts both sides have agree (the index's docstrings and spans are not compared).
//!
//! The folders are `tests/matlab/project` (the fixture: shadowing, packages, class folders,
//! private functions, classdef and old-style classes, arguments blocks). `MALT_MATLAB_EXTRA_PATHS`
//! (an OS path list) adds local folders, each with its plain subfolders, in front of the path,
//! as a user's own code would be; for example a toolkit clone. They are read, never copied.
//! Differences that are accepted are listed, each with a reason, in
//! `tests/matlab/namespace_allowlist.json`. Any other difference fails the test.
//!
//! Without MATLAB the test is skipped (see `common::matlab`); `MALT_MATLAB_REQUIRE=1` fails
//! instead. See `docs/VERIFY.md`, 'Cross-check against MATLAB'.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use malt_index::{ArgumentKind, Entity, Index, Kind, ParserConfig};
use serde::Deserialize;
use serde_json::{json, Map, Value};

use common::matlab::{matlab, matlab_string, MatlabRun};

/// The folders the test puts on the path, in order.
struct Roots {
    /// Added at the front of the path (`addpath -begin` in MATLAB).
    front: Vec<PathBuf>,
    /// Added at the end of the path (`addpath -end`), the extra folders last.
    back: Vec<PathBuf>,
}

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Where the facts come from. Two phases. Generation: MATLAB writes the facts file, with
/// `tests/matlab/generate_namespace_facts.m`; CI does that with `matlab-actions/run-command`,
/// which is the only licensed way to run MATLAB on a public repository. Comparison: this test
/// reads the file. With `MALT_MATLAB_FACTS` set, MATLAB is not started at all. Without it, MATLAB
/// is started here to generate the file in a temporary directory (the same script). `None` skips.
fn load_facts() -> Result<Option<Value>> {
    let required = env::var("MALT_MATLAB_REQUIRE").is_ok_and(|v| v == "1");
    let Some(path) = env::var_os("MALT_MATLAB_FACTS") else {
        let Some(run) = matlab() else {
            return Ok(None);
        };
        return generate(&run).map(Some);
    };
    if env::var_os("MALT_MATLAB_EXTRA_PATHS").is_some() {
        eprintln!(
            "note: MALT_MATLAB_EXTRA_PATHS is read only when MATLAB generates the facts; \
             the folders recorded in the facts file are compared"
        );
    }
    let path = facts_path(Path::new(&path));
    match fs::read_to_string(&path) {
        Ok(text) => {
            let facts = serde_json::from_str(&text)
                .with_context(|| format!("parsing the MATLAB facts {}", path.display()))?;
            eprintln!(
                "MATLAB facts read from {} (MATLAB not started)",
                path.display()
            );
            Ok(Some(facts))
        }
        Err(err) if required => bail!(
            "MALT_MATLAB_FACTS: cannot read {}: {err} (MALT_MATLAB_REQUIRE=1)",
            path.display()
        ),
        Err(err) => {
            eprintln!(
                "SKIP: cannot read the MATLAB facts {}: {err} (generate them with \
                 generate_namespace_facts.m; MALT_MATLAB_REQUIRE=1 makes this a failure)",
                path.display()
            );
            Ok(None)
        }
    }
}

/// A relative facts path is relative to the repository root: cargo runs a test in its crate.
fn facts_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let joined = manifest().join("../..").join(path);
    fs::canonicalize(&joined).unwrap_or(joined)
}

/// Generation with MATLAB: the generator script, as CI runs it, writes into a temporary file.
fn generate(run: &MatlabRun) -> Result<Value> {
    let dir = tempfile::tempdir()?;
    let out = dir.path().join("namespace_facts.json");
    let script = manifest().join("tests/matlab/generate_namespace_facts.m");
    run.run_batch_with(
        &format!("run({})", matlab_string(&script)),
        run.timeout(),
        &[("MALT_NAMESPACE_FACTS_OUT", out.as_os_str())],
    )?;
    let text = fs::read_to_string(&out)
        .context("reading the MATLAB facts written by generate_namespace_facts.m")?;
    Ok(serde_json::from_str(&text)?)
}

/// The folders the facts were taken with (the facts file records them, `matlab_facts.m`),
/// canonical, so the index is built from exactly the folders MATLAB saw.
fn roots_of(facts: &Value) -> Result<Roots> {
    let list = |key: &str| -> Result<Vec<PathBuf>> {
        facts["roots"][key]
            .as_array()
            .into_iter()
            .flatten()
            .map(|p| {
                let p = p
                    .as_str()
                    .context("a folder in the facts is not a string")?;
                fs::canonicalize(p).with_context(|| format!("folder {p}"))
            })
            .collect()
    };
    Ok(Roots {
        front: list("front")?,
        back: list("back")?,
    })
}

/// The index of the same folders, with the same precedence as MATLAB's path.
fn build_index(roots: &Roots) -> Result<Index> {
    let mut index = Index::scan(&roots.front, &ParserConfig::default());
    for dir in &roots.back {
        index.addpath(dir, true, false)?;
    }
    if let Some(err) = index.errors().first() {
        anyhow::bail!("the index could not read a folder: {err}");
    }
    Ok(index)
}

/// The fact of a `text`-like attribute: `Name` or `Name=value`, as the index stores them.
fn attr(entity: &Entity, name: &str) -> bool {
    entity.attributes.contains(name)
}

/// The value of an `Access`-like attribute (`GetAccess=private`), if set.
fn attr_value(entity: &Entity, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    entity
        .attributes
        .iter()
        .find_map(|a| a.strip_prefix(&prefix).map(str::to_string))
}

fn is_method(entity: &Entity) -> bool {
    entity.kind == Kind::Function
        && entity.path.as_deref().is_some_and(|p| {
            p.components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with('@'))
        })
}

/// The facts of one top-level entity, in the shape `matlab_facts.m` writes.
fn index_facts(index: &Index, key: &str, entity: &Entity) -> Value {
    let method = is_method(entity);
    let kind = match entity.kind {
        Kind::Function => "function",
        Kind::Script => "script",
        Kind::Class => "class",
        Kind::ClassFolder => "class_folder",
        Kind::Namespace => "package",
        _ => "other",
    };
    let file = entity
        .path
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned());
    // `which -all` for files: the top-level names list every folder that defines them.
    let all: Vec<String> = match kind {
        "class_folder" | "package" | "other" => Vec::new(),
        _ if !key.contains('.') => index
            .contributor_paths(key)
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect(),
        _ => file.iter().cloned().collect(),
    };
    let (mut nargin, mut nargout) = (Value::Null, Value::Null);
    if let (Some(function), false) = (entity.function(), method) {
        let inputs: Vec<_> = function
            .arguments
            .iter()
            .filter(|a| a.kind != ArgumentKind::KeywordOnly)
            .collect();
        let varargin = inputs.iter().any(|a| a.kind == ArgumentKind::Varargin);
        nargin = signed(inputs.len(), varargin);
        let varargout = function
            .returns
            .last()
            .is_some_and(|a| a.name == "varargout");
        nargout = signed(function.returns.len(), varargout);
    }
    let mut properties = Map::new();
    let mut methods = Map::new();
    let mut enumeration = Vec::new();
    let enumeration_class = entity.members.values().any(|m| m.kind == Kind::Enumeration);
    if entity.kind == Kind::Class {
        for (name, member) in &entity.members {
            match member.kind {
                Kind::Property => {
                    let getter = member.property().is_some_and(|p| p.getter.is_some());
                    let setter = member.property().is_some_and(|p| p.setter.is_some());
                    let constant = attr(member, "Constant");
                    // The defaults MATLAB applies when a property says nothing: a constant
                    // cannot be set ('none'), and a property of an enumeration class is
                    // immutable.
                    let default_set = if constant {
                        "none"
                    } else if enumeration_class {
                        "immutable"
                    } else {
                        "public"
                    };
                    let set_access = attr_value(member, "SetAccess")
                        .or_else(|| attr_value(member, "Access"))
                        .unwrap_or_else(|| default_set.to_string());
                    let get_access = attr_value(member, "GetAccess")
                        .or_else(|| attr_value(member, "Access"))
                        .unwrap_or_else(|| "public".to_string());
                    properties.insert(
                        name.clone(),
                        json!({
                            "get_access": get_access,
                            "set_access": set_access,
                            "constant": constant,
                            "dependent": attr(member, "Dependent"),
                            "hidden": attr(member, "Hidden"),
                            "abstract": attr(member, "Abstract"),
                            "get_method": getter,
                            "set_method": setter,
                        }),
                    );
                }
                Kind::Function => {
                    let function = member.function().cloned().unwrap_or_default();
                    methods.insert(
                        name.clone(),
                        json!({
                            "access": attr_value(member, "Access").unwrap_or_else(|| "public".into()),
                            "static": attr(member, "Static"),
                            "abstract": attr(member, "Abstract"),
                            "hidden": attr(member, "Hidden"),
                            "inputs": function.arguments.iter()
                                .filter(|a| a.kind != ArgumentKind::KeywordOnly)
                                .map(|a| a.name.clone()).collect::<Vec<_>>(),
                            "outputs": function.returns.iter().map(|a| a.name.clone()).collect::<Vec<_>>(),
                        }),
                    );
                }
                Kind::Enumeration => enumeration.push(name.clone()),
                _ => {}
            }
        }
    }
    let class = entity.kind == Kind::Class;
    json!({
        "kind": kind,
        "file": file,
        "all": all,
        "nargin": nargin,
        "nargout": nargout,
        "superclasses": if class { sorted(entity.bases().iter().map(|b| (**b).to_string()).collect()) } else { vec![] },
        "abstract": if class { Value::Bool(attr(entity, "Abstract")) } else { Value::Null },
        "hidden": if class { Value::Bool(attr(entity, "Hidden")) } else { Value::Null },
        "sealed": if class { Value::Bool(attr(entity, "Sealed")) } else { Value::Null },
        "dotted_resolves": if method { Value::Bool(true) } else { Value::Null },
        "properties": properties,
        "methods": methods,
        "events": if class { sorted(entity.events().to_vec()) } else { vec![] },
        "enumeration": sorted(enumeration),
    })
}

/// A signed count: negative when the list ends in a variable-length argument, as MATLAB's
/// `nargin` and `nargout` report it.
fn signed(count: usize, variable: bool) -> Value {
    let count = count as i64;
    json!(if variable { -count } else { count })
}

fn sorted(mut list: Vec<String>) -> Vec<String> {
    list.sort();
    list
}

/// One difference between MATLAB and the index.
#[derive(Debug, Clone)]
struct Difference {
    entity: String,
    field: String,
    matlab: Value,
    index: Value,
}

/// A value with its leaves flattened to dotted paths (`methods.bump.inputs[0]`).
fn flatten(prefix: &str, value: &Value, out: &mut BTreeMap<String, Value>) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, child) in map {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten(&path, child, out);
            }
        }
        Value::Array(list) if !list.is_empty() => {
            for (i, child) in list.iter().enumerate() {
                flatten(&format!("{prefix}[{i}]"), child, out);
            }
        }
        // An empty map has no leaves: its absence on one side is reported by its entries.
        Value::Object(_) => {}
        leaf => {
            out.insert(prefix.to_string(), leaf.clone());
        }
    }
}

/// The first two segments of a path: the field a missing item is reported under.
fn field_of(path: &str) -> String {
    let mut parts = path.splitn(3, '.');
    match (parts.next(), parts.next()) {
        (Some(a), Some(b)) => format!("{a}.{b}"),
        (Some(a), None) => a.to_string(),
        _ => path.to_string(),
    }
}

/// Whether a flattened map has a path at or below `prefix`.
fn has_under(map: &BTreeMap<String, Value>, prefix: &str) -> bool {
    map.keys().any(|k| {
        k == prefix || k.starts_with(&format!("{prefix}.")) || k.starts_with(&format!("{prefix}["))
    })
}

/// Compare two entities' facts (both already normalised to the same shape). A value that
/// differs, or a leaf one side lacks while the other side has its entry, is reported at the
/// leaf. A whole entry (a method, a property) that one side lacks is reported once, at
/// `methods.name`, with `present` or `absent` for each side.
fn compare_entity(key: &str, matlab: &Value, index: &Value, out: &mut Vec<Difference>) {
    let (mut a, mut b) = (BTreeMap::new(), BTreeMap::new());
    flatten("", matlab, &mut a);
    flatten("", index, &mut b);
    let paths: BTreeSet<&String> = a.keys().chain(b.keys()).collect();
    let mut whole: BTreeMap<String, (Value, Value)> = BTreeMap::new();
    for path in paths {
        match (a.get(path), b.get(path)) {
            (Some(x), Some(y)) if x == y => {}
            (Some(x), Some(y)) => out.push(Difference {
                entity: key.to_string(),
                field: path.clone(),
                matlab: x.clone(),
                index: y.clone(),
            }),
            (x, y) => {
                let entry = field_of(path);
                let (mine, theirs) = (has_under(&a, &entry), has_under(&b, &entry));
                if mine && theirs {
                    out.push(Difference {
                        entity: key.to_string(),
                        field: path.clone(),
                        matlab: x.cloned().unwrap_or_else(|| json!("absent")),
                        index: y.cloned().unwrap_or_else(|| json!("absent")),
                    });
                } else {
                    let side = |present: bool| json!(if present { "present" } else { "absent" });
                    whole.insert(entry, (side(mine), side(theirs)));
                }
            }
        }
    }
    for (field, (matlab, index)) in whole {
        out.push(Difference {
            entity: key.to_string(),
            field,
            matlab,
            index,
        });
    }
}

/// The MATLAB side in the shape both sides are compared in: no file for `""`, `which -all`
/// only in the folders the index was given (MATLAB's own path is not indexed), method
/// inputs without the object (the index drops it), superclasses sorted.
fn normalise_matlab(key: &str, fact: &Value, known: &[PathBuf]) -> Value {
    let mut fact = fact.clone();
    let class_name = key.rsplit('.').next().unwrap_or(key).to_string();
    if fact["file"] == json!("") {
        fact["file"] = Value::Null;
    }
    if let Some(all) = fact["all"].as_array() {
        let inside: Vec<Value> = all
            .iter()
            .filter(|p| {
                p.as_str()
                    .is_some_and(|p| known.iter().any(|root| Path::new(p).starts_with(root)))
            })
            .cloned()
            .collect();
        fact["all"] = json!(inside);
    }
    if let Some(bases) = fact["superclasses"].as_array() {
        let bases: Vec<String> = bases
            .iter()
            .filter_map(|b| b.as_str().map(str::to_string))
            .collect();
        fact["superclasses"] = json!(sorted(bases));
    }
    if let Some(methods) = fact["methods"].as_object_mut() {
        for (name, method) in methods.iter_mut() {
            let is_static = method["static"] == json!(true);
            let is_ctor = *name == class_name;
            // MATLAB's metadata shows `varargin` alone for a method with an arguments block
            // (the object is not listed): that is kept, as an accepted difference below.
            let metadata_varargin =
                method["inputs"].as_array().and_then(|i| i.first()) == Some(&json!("varargin"));
            if !is_static && !is_ctor && !metadata_varargin {
                if let Some(inputs) = method["inputs"].as_array_mut() {
                    if !inputs.is_empty() {
                        inputs.remove(0);
                    }
                }
            }
        }
    }
    fact
}

/// Glob match with `*` only (any run of characters).
fn glob(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let (first, last) = (parts[0], parts[parts.len() - 1]);
    if !text.starts_with(first) || text.len() < first.len() + last.len() {
        return false;
    }
    let mut rest = &text[first.len()..text.len() - last.len()];
    for middle in &parts[1..parts.len() - 1] {
        match rest.find(middle) {
            Some(at) => rest = &rest[at + middle.len()..],
            None => return false,
        }
    }
    true
}

#[derive(Debug, Deserialize)]
struct Allowlist {
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    /// Glob over the entity identifier (`*`, `a.*`, `oldcls.getv`).
    entity: String,
    /// Glob over the field, a flattened path (`methods.empty*`, `presence`). Several globs
    /// separated by `|`. `{class}` is the entity's own class name, and such an entry only
    /// accepts a class whose file declares no constructor of that name (an implicit one).
    field: String,
    /// Only for a MATLAB value equal to this text (for example `varargout`).
    #[serde(default)]
    matlab: Option<String>,
    /// `enumeration`: only for a class with enumeration members.
    #[serde(default)]
    requires: Option<String>,
    /// Why the difference is accepted. Required, not empty.
    reason: String,
}

/// What an allowlist entry may depend on, from the index.
struct IndexContext {
    /// The classes whose file declares a constructor (the index's `declares_constructor`).
    constructors: BTreeSet<String>,
    /// The classes with enumeration members.
    enumerations: BTreeSet<String>,
    /// The names MATLAB resolves to a `.p` file.
    p_files: BTreeSet<String>,
    /// The methods (as `class method`) whose MATLAB metadata is the placeholder `varargin`
    /// alone as inputs or `varargout` alone as outputs.
    metadata_methods: BTreeSet<String>,
}

/// The method a flattened field belongs to (`methods.rank.outputs[0]` gives `rank`).
fn method_of(field: &str) -> Option<&str> {
    let rest = field.strip_prefix("methods.")?;
    rest.split(['.', '[']).next()
}

fn accepted_by(entry: &Entry, diff: &Difference, ctx: &IndexContext) -> bool {
    let class = diff.entity.rsplit('.').next().unwrap_or(&diff.entity);
    if !glob(&entry.entity, &diff.entity) {
        return false;
    }
    let field = entry.field.replace("{class}", class);
    if !field.split('|').any(|pattern| glob(pattern, &diff.field)) {
        return false;
    }
    if entry.field.contains("{class}") && ctx.constructors.contains(&diff.entity) {
        return false;
    }
    let required = match entry.requires.as_deref() {
        Some("enumeration") => ctx.enumerations.contains(&diff.entity),
        Some("p-file") => ctx.p_files.contains(&diff.entity),
        Some("arguments-metadata") => method_of(&diff.field).is_some_and(|m| {
            ctx.metadata_methods
                .contains(&format!("{} {m}", diff.entity))
        }),
        Some(_) => false,
        None => true,
    };
    if !required {
        return false;
    }
    // `matlab` is a glob (or several separated by `|`) over the MATLAB value.
    entry.matlab.as_deref().is_none_or(|patterns| {
        diff.matlab
            .as_str()
            .is_some_and(|value| patterns.split('|').any(|p| glob(p, value)))
    })
}

fn load_allowlist() -> Result<Allowlist> {
    let path = manifest().join("tests/matlab/namespace_allowlist.json");
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let list: Allowlist = serde_json::from_str(&text)?;
    for entry in &list.entries {
        anyhow::ensure!(
            !entry.reason.trim().is_empty(),
            "allowlist entry for {} / {} has no reason",
            entry.entity,
            entry.field
        );
    }
    Ok(list)
}

/// The identifiers `what` lists: each `.m` file, class folder and package, as the index
/// names them (`Contents` excluded, as the index does).
fn listed_by_what(facts: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for record in facts["what"].as_array().into_iter().flatten() {
        let prefix = record["prefix"].as_str().unwrap_or_default();
        let class = record["class"].as_str().unwrap_or_default();
        let join = |name: &str| {
            if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{prefix}.{name}")
            }
        };
        // `what` lists the files with their extension.
        for file in record["m"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            let Some(stem) = file.strip_suffix(".m") else {
                continue;
            };
            if stem == "Contents" || (!class.is_empty() && stem == class) {
                continue;
            }
            out.insert(join(stem));
        }
        for name in record["classes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            out.insert(join(name));
        }
        for name in record["packages"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            out.insert(format!("+{}", join(name)));
        }
    }
    out
}

#[test]
fn index_agrees_with_matlab_on_the_project() -> Result<()> {
    let Some(facts) = load_facts()? else {
        return Ok(());
    };
    let roots = roots_of(&facts)?;
    let index = build_index(&roots)?;
    let members = index.members()?;

    let mut differences = Vec::new();
    let names = facts["names"].as_object().cloned().unwrap_or_default();
    let known: Vec<PathBuf> = roots.front.iter().chain(&roots.back).cloned().collect();
    let mut matlab_side: BTreeMap<String, Value> = BTreeMap::new();
    for (key, fact) in &names {
        matlab_side.insert(key.clone(), normalise_matlab(key, fact, &known));
    }
    let mut index_side: BTreeMap<String, Value> = BTreeMap::new();
    let mut context = IndexContext {
        constructors: BTreeSet::new(),
        enumerations: BTreeSet::new(),
        p_files: matlab_side
            .iter()
            .filter(|(_, fact)| fact["file"].as_str().is_some_and(|f| f.ends_with(".p")))
            .map(|(key, _)| key.clone())
            .collect(),
        metadata_methods: matlab_side
            .iter()
            .flat_map(|(key, fact)| {
                let methods = fact["methods"].as_object().into_iter().flatten();
                methods
                    .filter(|(_, m)| {
                        m["inputs"] == json!(["varargin"]) || m["outputs"] == json!(["varargout"])
                    })
                    .map(move |(name, _)| format!("{key} {name}"))
            })
            .collect(),
    };
    for (key, entity) in &members {
        index_side.insert(key.clone(), index_facts(&index, key, entity));
        if entity.kind == Kind::Class {
            if entity.declares_constructor() {
                context.constructors.insert(key.clone());
            }
            if entity.members.values().any(|m| m.kind == Kind::Enumeration) {
                context.enumerations.insert(key.clone());
            }
        }
    }

    // Presence: every name the index lists is one MATLAB lists, and the reverse.
    for key in index_side.keys().filter(|k| !matlab_side.contains_key(*k)) {
        differences.push(Difference {
            entity: key.clone(),
            field: "presence".into(),
            matlab: json!("absent"),
            index: json!("present"),
        });
    }
    for key in matlab_side.keys().filter(|k| !index_side.contains_key(*k)) {
        differences.push(Difference {
            entity: key.clone(),
            field: "presence".into(),
            matlab: json!("present"),
            index: json!("absent"),
        });
    }
    // Every name MATLAB's `what` lists is in the index.
    for key in listed_by_what(&facts) {
        if !index_side.contains_key(&key) {
            differences.push(Difference {
                entity: key,
                field: "presence".into(),
                matlab: json!("listed by what"),
                index: json!("absent"),
            });
        }
    }
    // Facts of the names both sides have.
    for (key, matlab_fact) in &matlab_side {
        if let Some(index_fact) = index_side.get(key) {
            compare_entity(key, matlab_fact, index_fact, &mut differences);
        }
    }
    // Private functions: not on the path, found from their own folder.
    for private in facts["private"].as_array().into_iter().flatten() {
        let name = private["name"].as_str().unwrap_or_default();
        let owner = Path::new(private["owner"].as_str().unwrap_or_default());
        let file = private["file"].as_str().unwrap_or_default();
        let key = format!("private {name} in {}", owner.display());
        let found = index
            .get_from(name, owner)?
            .map(|e| e.path.map(|p| p.to_string_lossy().into_owned()));
        let index_file = found.flatten().map_or(Value::Null, Value::from);
        let matlab_value = json!({ "file": file });
        let index_value = json!({ "file": index_file });
        compare_entity(&key, &matlab_value, &index_value, &mut differences);
    }

    let allowlist = load_allowlist()?;
    let mut used = vec![false; allowlist.entries.len()];
    let mut accepted = Vec::new();
    let mut unaccepted = Vec::new();
    for diff in differences {
        let hit = allowlist
            .entries
            .iter()
            .enumerate()
            .find(|(_, e)| accepted_by(e, &diff, &context));
        match hit {
            Some((i, entry)) => {
                used[i] = true;
                accepted.push((diff, entry.reason.clone()));
            }
            None => unaccepted.push(diff),
        }
    }
    report(
        &names,
        &index_side,
        &accepted,
        &unaccepted,
        &allowlist,
        &used,
        &facts,
    );
    assert!(
        unaccepted.is_empty(),
        "{} difference(s) between the index and MATLAB are not in the allowlist (see the report)",
        unaccepted.len()
    );
    Ok(())
}

/// Print the differences, grouped by field and then by entity.
fn report(
    names: &Map<String, Value>,
    index_side: &BTreeMap<String, Value>,
    accepted: &[(Difference, String)],
    unaccepted: &[Difference],
    allowlist: &Allowlist,
    used: &[bool],
    facts: &Value,
) {
    println!();
    println!("MATLAB namespace cross-check");
    println!(
        "  names: {} in MATLAB, {} in the index; {} private function(s); {} error(s) in MATLAB",
        names.len(),
        index_side.len(),
        facts["private"].as_array().map_or(0, Vec::len),
        facts["errors"].as_array().map_or(0, Vec::len),
    );
    for error in facts["errors"].as_array().into_iter().flatten() {
        println!("  MATLAB error: {error}");
    }
    let mut by_field: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    let mut line = |d: &Difference, status: &str| {
        let top = d
            .field
            .split(['.', '['])
            .next()
            .unwrap_or_default()
            .to_string();
        by_field
            .entry(format!("{top} {status}"))
            .or_default()
            .entry(d.entity.clone())
            .or_default()
            .push(format!(
                "{}: MATLAB {} / index {}",
                d.field,
                short(&d.matlab),
                short(&d.index)
            ));
    };
    for (diff, _) in accepted {
        line(diff, "accepted");
    }
    for diff in unaccepted {
        line(diff, "UNACCEPTED");
    }
    if by_field.is_empty() {
        println!("  no differences");
    }
    for (group, entities) in &by_field {
        println!("  [{group}]");
        for (entity, items) in entities {
            for item in items {
                println!("    {entity}: {item}");
            }
        }
    }
    for (i, entry) in allowlist.entries.iter().enumerate() {
        if !used[i] {
            println!(
                "  warning: allowlist entry matched nothing: {} / {} ({})",
                entry.entity, entry.field, entry.reason
            );
        }
    }
    let reasons: BTreeSet<&str> = accepted.iter().map(|(_, r)| r.as_str()).collect();
    if !reasons.is_empty() {
        println!("  accepted reasons:");
        for reason in reasons {
            println!("    - {reason}");
        }
    }
    println!(
        "  total: {} accepted, {} unaccepted",
        accepted.len(),
        unaccepted.len()
    );
}

fn short(value: &Value) -> String {
    let text = value.to_string();
    if text.len() > 200 {
        let cut = (0..=197)
            .rev()
            .find(|i| text.is_char_boundary(*i))
            .unwrap_or(0);
        format!("{}...", &text[..cut])
    } else {
        text
    }
}
