//! The scanner and the lookups on real folders: packages, class folders, `private`
//! folders, name precedence, addpath and rmpath, and the golden snapshot of the sample
//! project under `tests/project` (its dump is `tests/golden/project.json`).

use std::fs;
use std::path::{Path, PathBuf};

use malt_index::dump::dump;
use malt_index::{Index, Kind, ParserConfig};

fn project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/project")
}

/// A fresh folder under the system temp directory, with the given files.
fn scratch(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("malt-index-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    for (rel, text) in files {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    malt_index::fsutil::canonicalize(&root).unwrap()
}

fn scan(paths: &[PathBuf]) -> Index {
    Index::scan(paths, &ParserConfig::default())
}

#[test]
fn top_level_names_follow_the_folder_rules() {
    let index = scan(&[project()]);
    let names = index.member_names();
    for expected in [
        "Shapes",
        "Widget",
        "helper",
        "ScriptFile",
        "Broken",
        "+geo",
        "Account",
    ] {
        assert!(names.contains(&expected), "missing {expected} in {names:?}");
    }
    // Contents.m documents the folder; private/ is visible only next to its files.
    assert!(!names.contains(&"Contents"));
    assert!(!names.contains(&"secret"));
    assert!(index.errors().is_empty());
}

#[test]
fn package_members_resolve_with_and_without_the_plus() {
    let index = scan(&[project()]);
    let point = index.get("geo.Point").unwrap().expect("geo.Point");
    let same = index.get("+geo.Point").unwrap().expect("+geo.Point");
    assert_eq!(point, same);
    assert_eq!(point.kind, Kind::Class);
    assert!(point.members.contains_key("norm"));

    let circle = index
        .get("geo.shape.Circle")
        .unwrap()
        .expect("nested package");
    assert_eq!(circle.bases(), ["geo.Point"]);
    assert_eq!(
        circle
            .path
            .as_deref()
            .map(|p| p.ends_with("+geo/+shape/Circle.m")),
        Some(true)
    );

    let distance = index
        .get("geo.distance")
        .unwrap()
        .expect("function in a package");
    assert_eq!(distance.kind, Kind::Function);
}

#[test]
fn package_docstring_is_the_contents_file() {
    let index = scan(&[project()]);
    let geo = index.members().unwrap().remove("+geo").expect("+geo");
    assert_eq!(geo.kind, Kind::Namespace);
    assert_eq!(
        geo.docstring.unwrap().value,
        "Geometry package: points, distances and shapes."
    );
    assert!(geo.members.contains_key("Point"));
    assert!(geo.members.contains_key("shape"));
}

#[test]
fn class_folder_has_its_methods_and_the_class_members() {
    let index = scan(&[project()]);
    let account = index.get("Account").unwrap().expect("Account");
    assert_eq!(account.kind, Kind::ClassFolder);
    assert!(account.members.contains_key("deposit"));
    assert!(account.members.contains_key("Balance"));
    assert!(
        account.members.contains_key("Account"),
        "the constructor stays a member"
    );
    assert_eq!(account.docstring.unwrap().value, "A bank account.");

    let deposit = index.get("Account.deposit").unwrap().expect("method");
    assert_eq!(deposit.kind, Kind::Function);
    assert_eq!(deposit.name, "deposit");
}

#[test]
fn members_of_a_class_are_reached_by_name() {
    let index = scan(&[project()]);
    let size = index.get("Widget.Size").unwrap().expect("property");
    assert_eq!(size.kind, Kind::Property);
    assert!(size.attributes.contains("Dependent"));
    assert!(index.get("Widget.Nope").unwrap().is_none());
}

#[test]
fn private_functions_are_visible_next_to_their_folder_only() {
    let root = project();
    let index = scan(std::slice::from_ref(&root));
    assert!(index.get("secret").unwrap().is_none());
    let next_to = index
        .get_from("secret", &root)
        .unwrap()
        .expect("private visible to the folder");
    assert_eq!(next_to.name, "secret");
    assert!(index
        .get_from("secret", &root.join("+geo"))
        .unwrap()
        .is_none());
}

#[test]
fn local_names_shadow_the_path() {
    let root = scratch(
        "shadow",
        &[
            ("a/dup.m", "function r = dup()\nr = 1;\nend\n"),
            ("b/dup.m", "function r = dup()\nr = 2;\nend\n"),
        ],
    );
    let index = scan(&[root.join("a"), root.join("b")]);
    assert_eq!(
        index.get("dup").unwrap().unwrap().path,
        Some(root.join("a/dup.m"))
    );
    let local = index.get_from("dup", &root.join("b")).unwrap().unwrap();
    assert_eq!(local.path, Some(root.join("b/dup.m")));
}

#[test]
fn addpath_front_and_rmpath_change_the_winner() {
    let root = scratch(
        "addpath",
        &[
            ("a/dup.m", "function r = dup()\nr = 1;\nend\n"),
            ("b/dup.m", "function r = dup()\nr = 2;\nend\n"),
        ],
    );
    let mut index = scan(&[root.join("a")]);
    index.addpath(root.join("b"), false, false).unwrap();
    assert_eq!(
        index.get("dup").unwrap().unwrap().path,
        Some(root.join("b/dup.m"))
    );
    assert_eq!(index.search_path().len(), 2);

    assert!(index.rmpath(root.join("b"), false));
    assert_eq!(
        index.get("dup").unwrap().unwrap().path,
        Some(root.join("a/dup.m"))
    );
    assert!(
        !index.rmpath(root.join("b"), false),
        "removing twice reports false"
    );
}

#[test]
fn recursive_addpath_brings_subfolders_to_the_top_level() {
    let root = scratch(
        "recursive",
        &[
            ("top.m", "function r = top()\nr = 1;\nend\n"),
            ("sub/inner.m", "function r = inner()\nr = 1;\nend\n"),
            (
                "sub/private/hidden.m",
                "function r = hidden()\nr = 1;\nend\n",
            ),
        ],
    );
    let mut shallow = Index::new(ParserConfig::default());
    shallow.addpath(&root, true, false).unwrap();
    assert!(shallow.get("inner").unwrap().is_none());

    let mut deep = Index::new(ParserConfig::default());
    deep.addpath(&root, true, true).unwrap();
    assert!(deep.get("inner").unwrap().is_some());
    assert!(
        deep.get("hidden").unwrap().is_none(),
        "private folders of subfolders are not on the path"
    );
}

#[test]
fn unreadable_paths_are_reported_and_skipped() {
    let index = scan(&[PathBuf::from("/definitely/not/a/folder"), project()]);
    assert_eq!(index.errors().len(), 1);
    assert!(index.get("Shapes").unwrap().is_some());
}

#[test]
fn invalid_identifiers_are_errors_and_unknown_names_are_none() {
    let index = scan(&[project()]);
    assert!(index.get("geo..Point").is_err());
    assert!(index.get("").is_err());
    assert!(index.get("no.such.thing").unwrap().is_none());
}

#[test]
fn dotted_names_of_package_members_are_listed_as_maxx_lists_them() {
    let index = scan(&[project()]);
    let members = index.members().unwrap();
    assert!(members.contains_key("geo.Point"));
    assert!(members.contains_key("+geo.shape"));
    assert!(members.contains_key("geo.shape.Circle"));
    assert!(members.contains_key("Account.deposit"));
}

#[test]
fn golden_snapshot_of_the_sample_project() {
    let root = malt_index::fsutil::canonicalize(&project()).unwrap();
    let index = scan(std::slice::from_ref(&root));
    let mut value = dump(&index, &root).unwrap();
    value.as_object_mut().unwrap().remove("root");
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/project.json");
    let text = format!("{}\n", serde_json::to_string_pretty(&value).unwrap());
    if std::env::var_os("MALT_INDEX_UPDATE_GOLDEN").is_some() {
        fs::write(&golden, &text).unwrap();
    }
    let expected = fs::read_to_string(&golden).expect("golden file");
    assert!(
        expected == text,
        "golden snapshot differs; rerun with MALT_INDEX_UPDATE_GOLDEN=1 and review the diff"
    );
}
