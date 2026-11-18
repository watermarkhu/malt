//! Plain folders as entities, and the lookup of a path.
//!
//! A folder on the search path (or below one) is a [`Kind::Folder`] entity: its `.m` and
//! `.mlx` files and its packages and class folders, as the index reads them, and its
//! plain subfolders as folder entities too. Its files are read with a throwaway index of
//! the folder, so the lookup does not change the parse cache of the search path.
//!
//! A folder that is not on the path (and not below a folder on it) has no entity.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::IndexError;
use crate::index::{folder_docstring, Index};
use crate::model::{Entity, Kind};

impl Index {
    /// The folder at `dir` as an entity, if it is on the search path or below a folder on
    /// it. `None` otherwise (and for a path that does not exist).
    pub fn folder(&self, dir: impl AsRef<Path>) -> Result<Option<Entity>, IndexError> {
        let Ok(dir) = crate::fsutil::canonicalize(dir.as_ref()) else {
            return Ok(None);
        };
        if !self.is_folder_path(&dir) {
            return Ok(None);
        }
        self.folder_entity(&dir).map(Some)
    }

    /// The entity at a path: the folder (as [`Index::folder`]), or the file's entity when
    /// the path is a file in such a folder. `None` for anything else.
    pub fn get_path(&self, path: impl AsRef<Path>) -> Result<Option<Entity>, IndexError> {
        let Ok(path) = crate::fsutil::canonicalize(path.as_ref()) else {
            return Ok(None);
        };
        if path.is_dir() {
            return self.folder(&path);
        }
        let (Some(parent), Some(stem)) = (path.parent(), path.file_stem()) else {
            return Ok(None);
        };
        if !self.is_folder_path(parent) {
            return Ok(None);
        }
        let mut sub = Index::new(*self.config());
        sub.addpath(parent, true, false)?;
        sub.get(&stem.to_string_lossy())
    }

    /// Whether `dir` is a folder on the path, or below one through folders that are not
    /// packages, class folders or `private` folders.
    fn is_folder_path(&self, dir: &Path) -> bool {
        dir.is_dir()
            && self.search_path().iter().any(|root| {
                dir.strip_prefix(root).is_ok_and(|rel| {
                    rel.components().all(|c| {
                        let name = c.as_os_str().to_string_lossy();
                        !(name.starts_with(['+', '@', '.']) || name == "private")
                    })
                })
            })
    }

    /// The folder entity of a canonical directory: its files, packages and class folders
    /// (read by a sub-index of the folder), and its plain subfolders.
    fn folder_entity(&self, dir: &Path) -> Result<Entity, IndexError> {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut out = Entity::new(Kind::Folder, &name);
        out.path = Some(dir.to_path_buf());
        out.docstring = folder_docstring(dir)?;

        let mut sub = Index::new(*self.config());
        sub.addpath(dir, true, false)?;
        for key in sub.member_names() {
            if let Some(entity) = sub.get(key)? {
                out.members.insert(entity.name.clone(), entity);
            }
        }
        for subdir in plain_subfolders(dir)? {
            let entity = self.folder_entity(&subdir)?;
            out.members.insert(entity.name.clone(), entity);
        }
        Ok(out)
    }
}

/// The plain subfolders of a folder, sorted: not packages, class folders, `private` or
/// hidden folders.
fn plain_subfolders(dir: &Path) -> Result<Vec<PathBuf>, IndexError> {
    let read = fs::read_dir(dir).map_err(|e| IndexError::io(dir, &e))?;
    let mut out: Vec<PathBuf> = read
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
            path.is_dir()
                && name.is_some_and(|n| !(n.starts_with(['+', '@', '.']) || n == "private"))
        })
        .collect();
    out.sort();
    Ok(out)
}
