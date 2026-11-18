//! Plain folders as entities, and path lookups (`Index::folder`, `Index::get_path`).

use std::fs;
use std::path::{Path, PathBuf};

use malt_index::{Index, Kind, ParserConfig};

/// A project folder with the given files (relative path, contents).
fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("malt-folder-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    for (rel, text) in files {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    root
}

/// The standard layout, in a folder of its own (the tests run in parallel).
fn layout(name: &str) -> PathBuf {
    project(
        name,
        &[
            ("Contents.m", "% Folder docs.\n"),
            ("alpha.m", "function a = alpha()\na = 1;\nend\n"),
            ("+pkg/Beta.m", "function b = Beta()\nb = 2;\nend\n"),
            ("@Cls/Cls.m", "classdef Cls\nend\n"),
            ("sub/gamma.m", "function g = gamma()\ng = 3;\nend\n"),
            ("private/hidden.m", "function h = hidden()\nh = 4;\nend\n"),
        ],
    )
}

#[test]
fn a_folder_lists_its_files_packages_class_folders_and_subfolders() {
    let root = layout("lists");
    let index = Index::scan(std::slice::from_ref(&root), &ParserConfig::default());
    let folder = index
        .folder(&root)
        .unwrap()
        .expect("the folder is on the path");
    assert_eq!(folder.kind, Kind::Folder);
    assert_eq!(folder.name, root.file_name().unwrap().to_string_lossy());
    assert_eq!(folder.docstring.as_ref().unwrap().value, "Folder docs.");
    let names: Vec<&str> = folder.members.keys().map(String::as_str).collect();
    assert_eq!(names, ["Cls", "alpha", "pkg", "sub"]);
    assert_eq!(folder.members["alpha"].kind, Kind::Function);
    assert_eq!(folder.members["pkg"].kind, Kind::Namespace);
    assert_eq!(folder.members["Cls"].kind, Kind::ClassFolder);
    let sub = &folder.members["sub"];
    assert_eq!(sub.kind, Kind::Folder);
    assert!(sub.members.contains_key("gamma"));
}

#[test]
fn a_path_lookup_gives_a_file_or_a_folder_below_the_path() {
    let root = layout("lookup");
    let index = Index::scan(std::slice::from_ref(&root), &ParserConfig::default());

    let alpha = index.get_path(root.join("alpha.m")).unwrap().unwrap();
    assert_eq!((alpha.kind, alpha.name.as_str()), (Kind::Function, "alpha"));

    // Below the folder on the path: a subfolder is a folder, and its file a function.
    let gamma = index.get_path(root.join("sub/gamma.m")).unwrap().unwrap();
    assert_eq!(gamma.name, "gamma");
    assert_eq!(
        index.get_path(root.join("sub")).unwrap().unwrap().kind,
        Kind::Folder
    );
}

#[test]
fn what_is_not_a_folder_on_the_path_has_no_entity() {
    let root = layout("not-folder");
    let index = Index::scan(std::slice::from_ref(&root), &ParserConfig::default());
    // A package or class folder is not a folder, and neither is `private`.
    assert!(index.folder(root.join("+pkg")).unwrap().is_none());
    assert!(index.folder(root.join("@Cls")).unwrap().is_none());
    assert!(index
        .get_path(root.join("private/hidden.m"))
        .unwrap()
        .is_none());
    // A folder that is not on the path at all.
    let outside = std::env::temp_dir();
    assert!(index.folder(&outside).unwrap().is_none());
    assert!(index.get_path(root.join("missing.m")).unwrap().is_none());
    assert!(index.get_path(root.join("alpha.txt")).unwrap().is_none());
}

#[test]
fn a_folder_entity_does_not_change_the_search_path_or_the_names() {
    let root = layout("unchanged");
    let index = Index::scan(std::slice::from_ref(&root), &ParserConfig::default());
    let before = index.member_names().len();
    let _ = index.folder(&root).unwrap();
    let _ = index.get_path(root.join("alpha.m")).unwrap();
    assert_eq!(index.member_names().len(), before);
    assert!(index.get("alpha").unwrap().is_some());
}

#[test]
fn a_folder_of_a_path_below_the_root_is_on_the_path_too() {
    let root = layout("below");
    let sub: &Path = &root.join("sub");
    let index = Index::scan(std::slice::from_ref(&root), &ParserConfig::default());
    assert!(index.folder(sub).unwrap().is_some());
}
