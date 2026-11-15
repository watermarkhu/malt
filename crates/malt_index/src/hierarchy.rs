//! The class hierarchy: the method resolution order of a classdef, and the members a
//! class inherits from its bases.
//!
//! A base is resolved as MATLAB resolves a name in the class file: from the class
//! file's own folder first, then the search path (see [`Index::get_from`]). A base that
//! is not on the path is kept by name only and has no bases of its own. The order is
//! C3 linearisation, as Python uses. MATLAB does not define one, so an inconsistent
//! hierarchy takes the first base that is left instead of failing.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::error::IndexError;
use crate::index::Index;
use crate::model::{Entity, Kind};

/// A class in a hierarchy: its name as written, and the class when it resolves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ancestor {
    /// The class's name as written in the classdef (the file's name for the class itself).
    pub name: String,
    /// The class, when the name resolves on the path to a classdef.
    pub entity: Option<Entity>,
}

/// A member a class inherits from one of its bases.
#[derive(Debug, Clone, PartialEq)]
pub struct Inherited {
    /// The base the member comes from, as written in the classdef.
    pub from: String,
    /// The member's name.
    pub name: String,
    /// The member, as the base declares it.
    pub entity: Entity,
}

impl Index {
    /// The classes of `class` in method resolution order. The first entry is `class`
    /// itself, then its bases, each before its own bases. A class that is not a classdef
    /// (a function or a class folder) has no bases and its order is itself alone.
    pub fn mro(&self, class: &Entity) -> Result<Vec<Ancestor>, IndexError> {
        let start = Ancestor {
            name: class.name.clone(),
            entity: Some(class.clone()),
        };
        let dir = folder_of(class);
        self.linearize(start, dir, &mut Vec::new())
    }

    /// The members that `class` inherits: the members of each base in method resolution
    /// order, skipping names the class declares itself and names an earlier base gave.
    pub fn inherited_members(&self, class: &Entity) -> Result<Vec<Inherited>, IndexError> {
        let mut seen: BTreeSet<String> = class.members.keys().cloned().collect();
        let mut out = Vec::new();
        for ancestor in self.mro(class)?.iter().skip(1) {
            let Some(entity) = &ancestor.entity else {
                continue;
            };
            for (name, member) in &entity.members {
                if seen.insert(name.clone()) {
                    out.push(Inherited {
                        from: ancestor.name.clone(),
                        name: name.clone(),
                        entity: member.clone(),
                    });
                }
            }
        }
        Ok(out)
    }

    /// The linearisation of `node`, whose bases are resolved from `dir`. `stack` holds
    /// the names being linearised, so a cycle stops at the repeated class.
    fn linearize(
        &self,
        node: Ancestor,
        dir: Option<PathBuf>,
        stack: &mut Vec<String>,
    ) -> Result<Vec<Ancestor>, IndexError> {
        let Some(entity) = node.entity.clone() else {
            return Ok(vec![node]);
        };
        let mut bases = Vec::new();
        for name in entity.bases() {
            bases.push(self.resolve_base(name, dir.as_deref())?);
        }
        stack.push(node.name.clone());
        let mut sequences = Vec::new();
        for base in &bases {
            if stack.contains(&base.name) {
                sequences.push(vec![base.clone()]);
            } else {
                let base_dir = base.entity.as_ref().and_then(folder_of);
                sequences.push(self.linearize(base.clone(), base_dir, stack)?);
            }
        }
        stack.pop();
        sequences.push(bases);

        // A cycle can bring the node back through a base: it is listed once, first.
        let merged = c3_merge(sequences);
        let mut out = vec![node.clone()];
        out.extend(merged.into_iter().filter(|a| a.name != node.name));
        Ok(out)
    }

    /// A base as written, resolved to its classdef when it is on the path.
    fn resolve_base(&self, name: &str, dir: Option<&Path>) -> Result<Ancestor, IndexError> {
        let found = match dir {
            Some(dir) => self.get_from(name, dir)?,
            None => self.get(name)?,
        };
        Ok(Ancestor {
            name: name.to_string(),
            entity: found.filter(|e| e.kind == Kind::Class),
        })
    }
}

/// The folder that holds the file of `entity`, where its bases are looked up.
fn folder_of(entity: &Entity) -> Option<PathBuf> {
    entity
        .path
        .as_deref()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

/// C3 merge: the next class is the first head of a sequence that is in no sequence's
/// tail. When none is, the first head is taken, so an inconsistent hierarchy still ends.
fn c3_merge(mut sequences: Vec<Vec<Ancestor>>) -> Vec<Ancestor> {
    let mut out: Vec<Ancestor> = Vec::new();
    sequences.retain(|s| !s.is_empty());
    while !sequences.is_empty() {
        let next = sequences
            .iter()
            .map(|s| &s[0])
            .find(|head| {
                !sequences
                    .iter()
                    .any(|s| s[1..].iter().any(|other| other.name == head.name))
            })
            .unwrap_or(&sequences[0][0])
            .clone();
        for sequence in &mut sequences {
            if sequence[0].name == next.name {
                sequence.remove(0);
            }
        }
        sequences.retain(|s| !s.is_empty());
        if !out.iter().any(|a| a.name == next.name) {
            out.push(next);
        }
    }
    out
}
