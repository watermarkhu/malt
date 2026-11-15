//! The index of a MATLAB search path.
//!
//! Adding a path lists its folders and files up front (no file is read). A file
//! is parsed the first time one of its entities is needed, and the result is
//! cached in the entry (`OnceLock`), so a lookup costs one parse per file it uses.
//!
//! Folder rules (MATLAB's, as maxx implements them):
//! - a folder on the path contributes its `.m` files, except `Contents.m`, which
//!   documents the folder;
//! - `+pkg` folders are packages: their files and sub-packages are members;
//! - `@cls` folders are class folders: `cls.m` is the class file, the other `.m`
//!   files are methods;
//! - `private` folders are visible only to the files of their parent folder;
//! - with `recursive`, plain subfolders are scanned too, and their files join the
//!   path's top level (as genpath does), except `private` folders.
//!
//! Precedence: the first path wins a name. Within one path, names are taken in
//! sorted order (maxx takes them in directory order, which is not deterministic).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use crate::comments::normalize;
use crate::config::ParserConfig;
use crate::error::IndexError;
use crate::model::{Docstring, Entity, Kind};
use crate::parse::parse_file;
use crate::paths::{resolve_paths, PathConfig};

/// The index of one entry: a file, a package, or a class folder.
pub type EntryId = usize;

#[derive(Debug)]
enum EntryKind {
    File,
    Namespace,
    ClassFolder,
}

#[derive(Debug)]
struct Entry {
    kind: EntryKind,
    /// The path: the file, or the folder of a package or class folder.
    path: PathBuf,
    /// The name at the top level: the file stem, `+pkg` or the class name.
    name: String,
    /// Members of a package or class folder: name (without prefix) to entries.
    children: BTreeMap<String, Vec<EntryId>>,
    /// The parsed file, parsed on first use.
    parsed: OnceLock<Result<Arc<Entity>, IndexError>>,
}

#[derive(Debug)]
struct Root {
    path: PathBuf,
    /// Top-level entries of this path, in sorted order.
    top: Vec<EntryId>,
    /// Every file entry of this path (for the local lookup), with its folder.
    files: Vec<EntryId>,
    /// Files of `private` folders, with the folder they belong to.
    private: Vec<(PathBuf, EntryId)>,
}

/// The MATLAB search path and what it defines.
#[derive(Debug)]
pub struct Index {
    config: ParserConfig,
    entries: Vec<Entry>,
    /// The search path, first = highest precedence.
    roots: Vec<Root>,
    /// Top-level name to its contributors, in precedence order. Rebuilt on change.
    top: BTreeMap<String, Vec<EntryId>>,
    /// Local lookup: folder to the names of its files.
    siblings: BTreeMap<PathBuf, BTreeMap<String, EntryId>>,
    /// Local lookup: folder to the names of its `private` files.
    private: BTreeMap<PathBuf, BTreeMap<String, EntryId>>,
    /// Paths that could not be read when they were added.
    errors: Vec<IndexError>,
}

/// One item of a folder listing.
struct Listed {
    name: String,
    path: PathBuf,
    is_dir: bool,
}

/// The entries of a folder, sorted by name. Unreadable entries are skipped.
fn list(dir: &Path) -> Result<Vec<Listed>, IndexError> {
    let read = fs::read_dir(dir).map_err(|e| IndexError::io(dir, &e))?;
    let mut items: Vec<Listed> = read
        .filter_map(Result::ok)
        .map(|entry| {
            let path = entry.path();
            let is_dir = path.is_dir();
            Listed {
                name: entry.file_name().to_string_lossy().into_owned(),
                path,
                is_dir,
            }
        })
        .collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

/// The file stem of a `.m` file name, or `None` for anything else (and for `Contents.m`).
fn m_stem(name: &str) -> Option<&str> {
    let stem = name.strip_suffix(".m")?;
    if stem.is_empty() || name == "Contents.m" {
        None
    } else {
        Some(stem)
    }
}

/// The stem of a file that a folder lists: a `.m` file, or a `.mlx` when live scripts
/// are on (`live`).
fn source_stem(name: &str, live: bool) -> Option<&str> {
    if let Some(stem) = m_stem(name) {
        return Some(stem);
    }
    let stem = name.strip_suffix(".mlx")?;
    if live && !stem.is_empty() {
        Some(stem)
    } else {
        None
    }
}

impl Index {
    /// An empty search path.
    pub fn new(config: ParserConfig) -> Self {
        Index {
            config,
            entries: Vec::new(),
            roots: Vec::new(),
            top: BTreeMap::new(),
            siblings: BTreeMap::new(),
            private: BTreeMap::new(),
            errors: Vec::new(),
        }
    }

    /// An index of `paths`, each added to the end of the search path, not recursively.
    /// A path that cannot be read is skipped and reported by [`Index::errors`].
    pub fn scan(paths: &[PathBuf], config: &ParserConfig) -> Index {
        let mut index = Index::new(*config);
        for path in paths {
            if let Err(err) = index.addpath(path, true, false) {
                log::warn!("{err}");
                index.errors.push(err);
            }
        }
        index
    }

    /// The index of the search path that [`resolve_paths`] gives for `config`, the
    /// working directory `cwd`, the linted `file` and `extra_dirs`. The returned
    /// warnings are those of the resolution and of the folders that could not be read.
    pub fn from_config(
        config: &PathConfig,
        cwd: &Path,
        file: Option<&Path>,
        extra_dirs: &[PathBuf],
        parser: &ParserConfig,
    ) -> (Index, Vec<String>) {
        let (dirs, mut warnings) = resolve_paths(config, cwd, file, extra_dirs);
        let index = Index::scan(&dirs, parser);
        warnings.extend(index.errors().iter().map(ToString::to_string));
        (index, warnings)
    }

    /// The paths that could not be read by [`Index::scan`].
    pub fn errors(&self) -> &[IndexError] {
        &self.errors
    }

    /// The parser configuration.
    pub fn config(&self) -> &ParserConfig {
        &self.config
    }

    /// Add a folder to the search path: at the end (`to_end`) or at the front.
    /// A folder already on the path moves to the new position.
    pub fn addpath(
        &mut self,
        path: impl AsRef<Path>,
        to_end: bool,
        recursive: bool,
    ) -> Result<(), IndexError> {
        let path =
            fs::canonicalize(path.as_ref()).map_err(|e| IndexError::io(path.as_ref(), &e))?;
        self.roots.retain(|r| r.path != path);
        let root = self.scan_root(path, recursive)?;
        if to_end {
            self.roots.push(root);
        } else {
            self.roots.insert(0, root);
        }
        self.rebuild();
        Ok(())
    }

    /// Remove a folder from the search path, and with `recursive` the folders below it.
    /// Returns whether it was on the path.
    pub fn rmpath(&mut self, path: impl AsRef<Path>, recursive: bool) -> bool {
        let path = match fs::canonicalize(path.as_ref()) {
            Ok(p) => p,
            Err(_) => path.as_ref().to_path_buf(),
        };
        let before = self.roots.len();
        self.roots.retain(|r| {
            let below = recursive && r.path.starts_with(&path);
            r.path != path && !below
        });
        let removed = self.roots.len() != before;
        if removed {
            self.rebuild();
        }
        removed
    }

    /// The folders on the search path, in precedence order.
    pub fn search_path(&self) -> Vec<&Path> {
        self.roots.iter().map(|r| r.path.as_path()).collect()
    }

    /// The top-level names, in sorted order (packages as `+pkg`).
    pub fn member_names(&self) -> Vec<&str> {
        self.top.keys().map(String::as_str).collect()
    }

    /// Every top-level entity, merged across the paths that define the name, as maxx's
    /// collection lists them. Members of packages are also listed under their dotted
    /// identifiers (`pkg.fn`, `+pkg.sub` for a sub-package, `pkg.Cls` for a class folder),
    /// and the methods of a class folder under `Cls.method`, as maxx does. Packages
    /// defined by several paths are the union of their members.
    pub fn members(&self) -> Result<BTreeMap<String, Entity>, IndexError> {
        let mut out = BTreeMap::new();
        for (name, ids) in &self.top {
            if let Some(package) = name.strip_prefix('+') {
                self.add_package(&mut out, package, ids)?;
            } else if matches!(self.entries[ids[0]].kind, EntryKind::ClassFolder) {
                self.add_class_methods(&mut out, name, ids)?;
            }
            out.insert(name.clone(), self.materialize(ids)?);
        }
        Ok(out)
    }

    /// The members of a package under dotted identifiers, recursively.
    fn add_package(
        &self,
        out: &mut BTreeMap<String, Entity>,
        package: &str,
        ids: &[EntryId],
    ) -> Result<(), IndexError> {
        for (key, members) in self.merged_children(ids) {
            let dotted = format!("{package}.{key}");
            match self.entries[members[0]].kind {
                EntryKind::Namespace => {
                    self.add_package(out, &dotted, &members)?;
                    out.insert(format!("+{dotted}"), self.materialize(&members)?);
                }
                EntryKind::ClassFolder => {
                    self.add_class_methods(out, &dotted, &members)?;
                    out.insert(dotted, self.materialize(&members)?);
                }
                EntryKind::File => {
                    out.insert(dotted, self.materialize(&members)?);
                }
            }
        }
        Ok(())
    }

    /// The methods of a class folder (the files other than the class file) under `Class.method`.
    fn add_class_methods(
        &self,
        out: &mut BTreeMap<String, Entity>,
        class: &str,
        ids: &[EntryId],
    ) -> Result<(), IndexError> {
        let class_file = self.entries[ids[0]].name.clone();
        for (key, members) in self.merged_children(ids) {
            if key != class_file {
                out.insert(format!("{class}.{key}"), self.materialize(&members)?);
            }
        }
        Ok(())
    }

    /// The entity of an identifier: `fn`, `Class`, `pkg.sub.fn`, `+pkg.Class`,
    /// `ClassFolder.method`, or `Class.Property`. `None` when nothing matches.
    pub fn get(&self, identifier: &str) -> Result<Option<Entity>, IndexError> {
        self.resolve(identifier, None)
    }

    /// Like [`Index::get`], but a plain name first looks at the files next to
    /// `dir` and in its `private` folder, as MATLAB does for a file in `dir`.
    pub fn get_from(&self, identifier: &str, dir: &Path) -> Result<Option<Entity>, IndexError> {
        self.resolve(identifier, Some(dir))
    }

    /// The contributors of a name in the search path, in precedence order.
    pub fn contributors(&self, name: &str) -> &[EntryId] {
        self.top.get(name).map(Vec::as_slice).unwrap_or(&[])
    }

    fn resolve(&self, identifier: &str, dir: Option<&Path>) -> Result<Option<Entity>, IndexError> {
        let parts: Vec<&str> = identifier.split('.').collect();
        if parts.iter().any(|p| p.is_empty()) {
            return Err(IndexError::InvalidIdentifier(identifier.to_string()));
        }
        if parts.len() == 1 {
            if let Some(dir) = dir {
                if let Some(entity) = self.local(parts[0], dir)? {
                    return Ok(Some(entity));
                }
            }
        }
        let head = parts[0];
        let mut found: Vec<EntryId> = Vec::new();
        if parts.len() > 1 && !head.starts_with('+') {
            found.extend(self.contributors(&format!("+{head}")));
        }
        found.extend(self.contributors(head));
        if found.is_empty() {
            return Ok(None);
        }

        // Walk the packages and class folders while the parts name entries.
        let mut i = 0;
        while i + 1 < parts.len() {
            let next: Vec<EntryId> = found
                .iter()
                .filter_map(|id| match self.entries[*id].kind {
                    EntryKind::Namespace | EntryKind::ClassFolder => {
                        self.entries[*id].children.get(parts[i + 1])
                    }
                    EntryKind::File => None,
                })
                .flatten()
                .copied()
                .collect();
            if next.is_empty() {
                break;
            }
            found = next;
            i += 1;
        }
        if i + 1 == parts.len() {
            return self.materialize(&found).map(Some);
        }

        // The remaining parts name members of a class: parse the class and walk its members.
        let rest = &parts[i + 1..];
        let class_entry = found.iter().find_map(|id| {
            let entry = &self.entries[*id];
            match entry.kind {
                EntryKind::File => Some(*id),
                EntryKind::ClassFolder => entry
                    .children
                    .get(&entry.name)
                    .and_then(|ids| ids.first().copied()),
                EntryKind::Namespace => None,
            }
        });
        let Some(class_id) = class_entry else {
            return Ok(None);
        };
        let mut entity: Entity = (*self.file_entity(class_id)?).clone();
        for part in rest {
            match entity.members.remove(*part) {
                Some(member) => entity = member,
                None => return Ok(None),
            }
        }
        Ok(Some(entity))
    }

    /// The entity of a plain name next to `dir`: its `private` files first, then its siblings.
    fn local(&self, name: &str, dir: &Path) -> Result<Option<Entity>, IndexError> {
        let found = self
            .private
            .get(dir)
            .and_then(|m| m.get(name))
            .or_else(|| self.siblings.get(dir).and_then(|m| m.get(name)));
        match found {
            Some(id) => Ok(Some((*self.file_entity(*id)?).clone())),
            None => Ok(None),
        }
    }

    /// The entity of a set of contributors: the first file, or the union of the packages
    /// and class folders among them.
    fn materialize(&self, ids: &[EntryId]) -> Result<Entity, IndexError> {
        let Some(&first) = ids.first() else {
            return Err(IndexError::InvalidIdentifier(String::new()));
        };
        match self.entries[first].kind {
            EntryKind::File => Ok((*self.file_entity(first)?).clone()),
            EntryKind::Namespace => {
                let entry = &self.entries[first];
                let mut out = Entity::new(Kind::Namespace, entry.name.trim_start_matches('+'));
                out.path = Some(entry.path.clone());
                out.docstring = folder_docstring(&entry.path)?;
                for (key, members) in self.merged_children(ids) {
                    out.members.insert(key, self.materialize(&members)?);
                }
                Ok(out)
            }
            EntryKind::ClassFolder => self.class_folder(ids),
        }
    }

    /// A class folder: its methods and its class file (the file named after the folder).
    fn class_folder(&self, ids: &[EntryId]) -> Result<Entity, IndexError> {
        let first = &self.entries[ids[0]];
        let class_name = first.name.clone();
        let mut out = Entity::new(Kind::ClassFolder, &class_name);
        out.path = Some(first.path.clone());
        let folder_doc = folder_docstring(&first.path)?;
        let mut class_file: Option<Entity> = None;
        for (key, members) in self.merged_children(ids) {
            let entity = self.materialize(&members)?;
            if key == class_name {
                class_file = Some(entity);
            } else {
                out.members.insert(key, entity);
            }
        }
        match class_file {
            Some(class) if class.kind == Kind::Class => {
                // The class members win over methods of the same name.
                out.docstring = class.docstring.clone().or(folder_doc);
                for (name, member) in class.members {
                    out.members.insert(name, member);
                }
            }
            Some(function) => {
                // A function class file (old-style constructor) is a method of the folder.
                out.docstring = function.docstring.clone().or(folder_doc);
                out.members.insert(class_name, function);
            }
            None => out.docstring = folder_doc,
        }
        Ok(out)
    }

    /// The members of a set of contributors, grouped by name, in precedence order.
    fn merged_children(&self, ids: &[EntryId]) -> BTreeMap<String, Vec<EntryId>> {
        let mut merged: BTreeMap<String, Vec<EntryId>> = BTreeMap::new();
        for id in ids {
            for (key, members) in &self.entries[*id].children {
                let list = merged.entry(key.clone()).or_default();
                for m in members {
                    if !list.contains(m) {
                        list.push(*m);
                    }
                }
            }
        }
        merged
    }

    /// The parsed file of an entry (cached).
    fn file_entity(&self, id: EntryId) -> Result<Arc<Entity>, IndexError> {
        let entry = &self.entries[id];
        let result = entry
            .parsed
            .get_or_init(|| parse_file(&entry.path, &self.config).map(Arc::new));
        match result {
            Ok(entity) => Ok(Arc::clone(entity)),
            Err(err) => Err(err.clone()),
        }
    }

    /// Recompute the top-level names and the local lookup from the search path.
    fn rebuild(&mut self) {
        self.top.clear();
        self.siblings.clear();
        self.private.clear();
        for root in &self.roots {
            for id in &root.top {
                self.top
                    .entry(self.entries[*id].name.clone())
                    .or_default()
                    .push(*id);
            }
            for id in &root.files {
                let entry = &self.entries[*id];
                if let Some(dir) = entry.path.parent() {
                    let stem = entry.name.clone();
                    self.siblings
                        .entry(dir.to_path_buf())
                        .or_default()
                        .entry(stem)
                        .or_insert(*id);
                }
            }
            for (dir, id) in &root.private {
                let stem = self.entries[*id].name.clone();
                self.private
                    .entry(dir.clone())
                    .or_default()
                    .entry(stem)
                    .or_insert(*id);
            }
        }
    }

    /// List one search-path folder and create its entries.
    fn scan_root(&mut self, path: PathBuf, recursive: bool) -> Result<Root, IndexError> {
        let mut root = Root {
            path: path.clone(),
            top: Vec::new(),
            files: Vec::new(),
            private: Vec::new(),
        };
        self.scan_folder(&path, recursive, &mut root)?;
        Ok(root)
    }

    /// The entries of one plain folder: its files, packages and class folders, and with
    /// `recursive` its subfolders (whose entries join the top level).
    fn scan_folder(
        &mut self,
        dir: &Path,
        recursive: bool,
        root: &mut Root,
    ) -> Result<(), IndexError> {
        let items = list(dir)?;
        let mut files_here = 0;
        for item in items {
            if item.is_dir {
                if item.name.starts_with('+') {
                    let id = self.scan_package(&item.path, root)?;
                    root.top.push(id);
                } else if item.name.starts_with('@') {
                    let id = self.scan_class_folder(&item.path, root)?;
                    root.top.push(id);
                } else if recursive && item.name != "private" {
                    self.scan_folder(&item.path, recursive, root)?;
                }
                continue;
            }
            if let Some(stem) = source_stem(&item.name, self.config.parse_live_scripts) {
                let id = self.push_file(item.path, stem.to_string());
                root.files.push(id);
                root.top.push(id);
                files_here += 1;
            }
        }
        if files_here > 0 {
            self.scan_private(dir, root)?;
        }
        Ok(())
    }

    /// A package: its files and sub-packages, and its class folders.
    fn scan_package(&mut self, dir: &Path, root: &mut Root) -> Result<EntryId, IndexError> {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut children: BTreeMap<String, Vec<EntryId>> = BTreeMap::new();
        let mut files_here = 0;
        for item in list(dir)? {
            if item.is_dir {
                let child = if item.name.starts_with('+') {
                    Some(self.scan_package(&item.path, root)?)
                } else if item.name.starts_with('@') {
                    Some(self.scan_class_folder(&item.path, root)?)
                } else {
                    None
                };
                if let Some(id) = child {
                    let key = self.entries[id]
                        .name
                        .trim_start_matches(['+', '@'])
                        .to_string();
                    children.entry(key).or_default().push(id);
                }
            } else if let Some(stem) = source_stem(&item.name, self.config.parse_live_scripts) {
                let id = self.push_file(item.path, stem.to_string());
                root.files.push(id);
                children.entry(stem.to_string()).or_default().push(id);
                files_here += 1;
            }
        }
        if files_here > 0 {
            self.scan_private(dir, root)?;
        }
        Ok(self.push_entry(EntryKind::Namespace, dir.to_path_buf(), name, children))
    }

    /// A class folder: its `.m` files (the class file and the methods).
    fn scan_class_folder(&mut self, dir: &Path, root: &mut Root) -> Result<EntryId, IndexError> {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
            .trim_start_matches('@')
            .to_string();
        let mut children: BTreeMap<String, Vec<EntryId>> = BTreeMap::new();
        let mut files_here = 0;
        for item in list(dir)? {
            if item.is_dir {
                continue;
            }
            if let Some(stem) = m_stem(&item.name) {
                let id = self.push_file(item.path, stem.to_string());
                root.files.push(id);
                children.entry(stem.to_string()).or_default().push(id);
                files_here += 1;
            }
        }
        if files_here > 0 {
            self.scan_private(dir, root)?;
        }
        Ok(self.push_entry(EntryKind::ClassFolder, dir.to_path_buf(), name, children))
    }

    /// The `private` files of a folder: visible to the files of that folder only.
    fn scan_private(&mut self, dir: &Path, root: &mut Root) -> Result<(), IndexError> {
        let private = dir.join("private");
        if !private.is_dir() {
            return Ok(());
        }
        for item in list(&private)? {
            if item.is_dir {
                continue;
            }
            if let Some(stem) = m_stem(&item.name) {
                let id = self.push_file(item.path, stem.to_string());
                root.private.push((dir.to_path_buf(), id));
            }
        }
        Ok(())
    }

    fn push_file(&mut self, path: PathBuf, name: String) -> EntryId {
        self.push_entry(EntryKind::File, path, name, BTreeMap::new())
    }

    fn push_entry(
        &mut self,
        kind: EntryKind,
        path: PathBuf,
        name: String,
        children: BTreeMap<String, Vec<EntryId>>,
    ) -> EntryId {
        let name = match kind {
            EntryKind::Namespace => format!("+{}", name.trim_start_matches('+')),
            _ => name,
        };
        self.entries.push(Entry {
            kind,
            path,
            name,
            children,
            parsed: OnceLock::new(),
        });
        self.entries.len() - 1
    }
}

/// The docstring of a folder: the `Contents.m` header, else its `README.md` text.
pub(crate) fn folder_docstring(dir: &Path) -> Result<Option<Docstring>, IndexError> {
    let contents = dir.join("Contents.m");
    if contents.is_file() {
        if let Some(doc) = parse_file(&contents, &ParserConfig::default())?.docstring {
            return Ok(Some(doc));
        }
    }
    for readme in ["README.md", "readme.md"] {
        let path = dir.join(readme);
        if path.is_file() {
            let bytes = fs::read(&path).map_err(|e| IndexError::io(&path, &e))?;
            let text = String::from_utf8_lossy(&bytes).into_owned();
            let lines = text.lines().count().max(1);
            return Ok(Some(Docstring {
                value: normalize(&text),
                raw: text,
                lineno: 1,
                endlineno: lines,
            }));
        }
    }
    Ok(None)
}
