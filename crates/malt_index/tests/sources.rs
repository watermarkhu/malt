//! The source-line cache (`Sources`): read once per path, shared, and errors for missing files.

use std::fs;
use std::sync::Arc;

use malt_index::{IndexError, Sources};

#[test]
fn lines_are_read_once_and_cached_by_path() {
    let dir = std::env::temp_dir().join(format!("malt-sources-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("a.m");
    fs::write(&file, "a = 1;\r\nb = 2;\r\n").unwrap();

    let sources = Sources::new();
    assert!(!sources.is_cached(&file));
    let first = sources.lines(&file).unwrap();
    assert_eq!(first.as_slice(), ["a = 1;", "b = 2;"]);
    assert!(sources.is_cached(&file));

    // The second read is the cached value, even if the file has changed on disk.
    fs::write(&file, "changed\n").unwrap();
    let second = sources.lines(&file).unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(sources.cached_paths(), [file]);
}

#[test]
fn a_missing_file_is_an_io_error_and_is_not_cached() {
    let sources = Sources::new();
    let missing = std::env::temp_dir().join("malt-sources-no-such-file.m");
    assert!(matches!(
        sources.lines(&missing),
        Err(IndexError::Io { .. })
    ));
    assert!(!sources.is_cached(&missing));
}
