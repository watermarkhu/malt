//! The JSON dump of an index, in the schema of `docs/scripts/maxx_dump.py`, so the
//! two outputs can be compared (see `docs/scripts/compare_index.py`).
//!
//! Keys are sorted. Fields follow maxx's rules per kind: `bases` only for classes,
//! `arguments`, `returns`, `is_getter`, `is_setter` only for functions, and `type`,
//! `dimensions`, `validators`, `default` only for properties. `events` (sorted names) and
//! `declares_constructor` are index-side only: maxx has no such keys, and
//! `docs/scripts/compare_index.py` compares only the fields of maxx's schema. `path` is relative
//! to the first path of the search path (absolute when outside it).

use std::path::Path;

use serde_json::{json, Map, Value};

use crate::error::IndexError;
use crate::index::Index;
use crate::model::{Argument, Entity, Kind};

/// The dump of every top-level entity of the index.
pub fn dump(index: &Index, root: &Path) -> Result<Value, IndexError> {
    let mut members = Map::new();
    for (name, entity) in index.members()? {
        members.insert(name, entity_json(&entity, root));
    }
    Ok(json!({
        "root": root.to_string_lossy(),
        "members": members,
    }))
}

/// One entity as the schema's object.
pub fn entity_json(entity: &Entity, root: &Path) -> Value {
    let mut out = Map::new();
    out.insert("kind".into(), json!(entity.kind.as_str()));
    out.insert("name".into(), json!(entity.name));
    out.insert("lineno".into(), json!(entity.lineno));
    out.insert("endlineno".into(), json!(entity.endlineno));
    out.insert(
        "docstring".into(),
        json!(entity.docstring.as_ref().map(|d| d.value.as_str())),
    );
    if let Some(path) = &entity.path {
        let shown = match path.strip_prefix(root) {
            // Forward slashes on every platform, so the JSON does not depend on it.
            Ok(relative) => relative.to_string_lossy().replace('\\', "/"),
            Err(_) => path.to_string_lossy().into_owned(),
        };
        out.insert("path".into(), json!(shown));
    }
    out.insert(
        "attributes".into(),
        json!(entity.attributes.iter().collect::<Vec<_>>()),
    );
    if entity.kind == Kind::Class {
        out.insert("bases".into(), json!(entity.bases()));
        let mut events = entity.events().to_vec();
        events.sort();
        out.insert("events".into(), json!(events));
        out.insert(
            "declares_constructor".into(),
            json!(entity.declares_constructor()),
        );
    }
    if let Some(function) = entity.function() {
        out.insert("arguments".into(), arguments_json(&function.arguments));
        out.insert("returns".into(), arguments_json(&function.returns));
        out.insert("is_getter".into(), json!(function.is_getter));
        out.insert("is_setter".into(), json!(function.is_setter));
    }
    if let Some(property) = entity.property() {
        out.insert("type".into(), json!(property.type_));
        out.insert("dimensions".into(), json!(property.dimensions));
        out.insert("validators".into(), json!(property.validators));
        out.insert("default".into(), json!(property.default));
    }
    if entity.kind == Kind::LiveScript {
        let sections: Vec<Value> = entity
            .sections()
            .iter()
            .map(|s| {
                json!({
                    "kind": s.kind.as_str(),
                    "content": s.content,
                    "lineno": s.lineno,
                    "endlineno": s.endlineno,
                })
            })
            .collect();
        out.insert("sections".into(), Value::Array(sections));
    }
    let members: Map<String, Value> = entity
        .members
        .iter()
        .map(|(name, member)| (name.clone(), entity_json(member, root)))
        .collect();
    out.insert("members".into(), Value::Object(members));
    Value::Object(out)
}

fn arguments_json(arguments: &[Argument]) -> Value {
    Value::Array(
        arguments
            .iter()
            .map(|a| {
                json!({
                    "name": a.name,
                    "kind": a.kind.as_str(),
                    "type": a.type_,
                    "dimensions": a.dimensions,
                    "validators": a.validators,
                    "default": a.default,
                    "docstring": a.docstring.as_ref().map(|d| d.value.as_str()),
                })
            })
            .collect(),
    )
}
