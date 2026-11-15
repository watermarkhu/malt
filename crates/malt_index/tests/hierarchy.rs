//! The class hierarchy: method resolution order (C3) and inherited members, on small
//! classdef trees built under the system temp directory. Each test passes its own
//! folder, so the classes of one test never see another's.

use std::fs;
use std::path::PathBuf;

use malt_index::{Index, ParserConfig};

/// A fresh project folder with the given files, each written with `source`.
fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("malt-hier-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    for (rel, source) in files {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    fs::canonicalize(&root).unwrap()
}

fn names(chain: &[malt_index::Ancestor]) -> Vec<&str> {
    chain.iter().map(|a| a.name.as_str()).collect()
}

#[test]
fn diamond_follows_c3() {
    // D < B, C;  B < A;  C < A.  C3 gives D, B, C, A (as Python does).
    let root = project(
        "diamond",
        &[
            ("A.m", "classdef A\nend\n"),
            ("B.m", "classdef B < A\nend\n"),
            ("C.m", "classdef C < A\nend\n"),
            ("D.m", "classdef D < B & C\nend\n"),
        ],
    );
    let index = Index::scan(std::slice::from_ref(&root), &ParserConfig::default());
    let d = index.get("D").unwrap().unwrap();
    let chain = index.mro(&d).unwrap();
    assert_eq!(names(&chain), ["D", "B", "C", "A"]);
    assert!(chain.iter().all(|a| a.entity.is_some()));
}

#[test]
fn unresolved_base_is_kept_by_name() {
    let root = project(
        "unresolved",
        &[("K.m", "classdef K < matlab.mixin.Copyable\nend\n")],
    );
    let index = Index::scan(std::slice::from_ref(&root), &ParserConfig::default());
    let k = index.get("K").unwrap().unwrap();
    let chain = index.mro(&k).unwrap();
    assert_eq!(names(&chain), ["K", "matlab.mixin.Copyable"]);
    assert!(chain[1].entity.is_none());
}

#[test]
fn inherited_members_skip_overridden_names() {
    let root = project(
        "inherit",
        &[
            (
                "Base.m",
                "classdef Base < handle\n    properties\n        Size = 1\n    end\n    methods\n        function a = area(obj)\n            a = 0;\n        end\n        function s = name(obj)\n            s = 'b';\n        end\n    end\nend\n",
            ),
            (
                "Sub.m",
                "classdef Sub < Base\n    methods\n        function a = area(obj)\n            a = 1;\n        end\n    end\nend\n",
            ),
        ],
    );
    let index = Index::scan(std::slice::from_ref(&root), &ParserConfig::default());
    let sub = index.get("Sub").unwrap().unwrap();
    let inherited = index.inherited_members(&sub).unwrap();
    let got: Vec<(&str, &str)> = inherited
        .iter()
        .map(|i| (i.from.as_str(), i.name.as_str()))
        .collect();
    assert_eq!(got, [("Base", "Size"), ("Base", "name")]);
}

#[test]
fn class_folder_and_function_have_no_bases() {
    let root = project(
        "plain",
        &[("@Money/Money.m", "function m = Money(a)\nm.a = a;\nend\n")],
    );
    let index = Index::scan(std::slice::from_ref(&root), &ParserConfig::default());
    let money = index.get("Money").unwrap().unwrap();
    assert_eq!(names(&index.mro(&money).unwrap()), ["Money"]);
    assert!(index.inherited_members(&money).unwrap().is_empty());
}

#[test]
fn cycle_stops_at_the_repeated_class() {
    let root = project(
        "cycle",
        &[
            ("P.m", "classdef P < Q\nend\n"),
            ("Q.m", "classdef Q < P\nend\n"),
        ],
    );
    let index = Index::scan(std::slice::from_ref(&root), &ParserConfig::default());
    let p = index.get("P").unwrap().unwrap();
    assert_eq!(names(&index.mro(&p).unwrap()), ["P", "Q"]);
}
