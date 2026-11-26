---
icon: lucide/hammer
---

# Development

Where the index lives, how to build it, and what the tests cover.

## Layout

| Path | What it is |
| ---- | ---------- |
| `crates/malt_index` | The Rust crate `malt-index`. No dependency on the linter crates. |
| `crates/malt_py` | The Python bindings, built with PyO3 and maturin. The module is `malt_index`. |
| `crates/malt_py/python` | The type stubs (`malt_index.pyi`), the `py.typed` marker and the Python tests. |
| `crates/malt_py/src/matlab_builtins.json` | The builtin documentation pages behind `Expr.doc`. |

The Rust library name of the bindings, `malt_index_py`, differs from the crate it wraps so that the two do not clash. The
distribution is `malt-index` and the module is `malt_index`.

## Build and test

```sh
cargo test -p malt-index                 # the Rust crate
cargo test -p malt-py                    # the Rust side of the bindings

pip install maturin pytest
maturin build -m crates/malt_py/Cargo.toml --release -o dist    # the wheel
pip install dist/malt_index-*.whl
pytest crates/malt_py/python/tests       # the Python tests
```

The Python tests need the built wheel, because the module is a compiled extension. `maturin develop` installs it in the
current environment instead. `test_index.py` runs on the sample project in `crates/malt_py/python/tests/project`, and
`test_live_expr_folders.py` writes its own projects under `tmp_path`, including a `.mlx` built with `zipfile`.

## What the Rust tests cover

`cargo test -p malt-index`:

- `src/comments.rs` has the docstring text rules. `src/expr.rs`, `src/live.rs` and `src/source.rs` have unit tests of the
  expression text, the cell and marker rules, and the line splitting.
- `tests/parse.rs` runs the parser on small samples: every kind, argument forms, attributes, enumerations, placement,
  block comments, nested functions, BOM, CRLF and Latin-1, malformed input, `events` names and the declared constructor.
- `tests/expr.rs` covers expressions: text, span, the joined type, bases, enumeration values and the `opts.?` class path.
- `tests/live.rs` builds `.mlx` files in the test with `zip`: paragraph runs, namespace-free XML, malformed XML, a file
  that is not a zip, a zip without `document.xml`, and truncated and random archives. It also covers plain-text live
  code (cells, markers, outputs, the appendix, CRLF) and the `parse_live_scripts` flag on the path.
- `tests/index.rs` scans and looks up on `tests/project`: packages, class folders, `private/`, precedence,
  addpath/rmpath and recursion. It holds the golden snapshot `tests/golden/project.json`.
- `tests/folder.rs` covers folders (members, subfolders, `Contents.m`), paths of files and folders, and what is not a
  folder on the path.
- `tests/paths.rs` covers the path configuration on temporary projects: the `[index]` table, each glob rule, missing
  folders, deduplication, order and config parse errors.
- `tests/hierarchy.rs` covers C3 order (a diamond), unresolved bases, inherited members with overrides, class folders
  and cycles.
- `tests/sources.rs` covers the line cache, and `tests/logging.rs` the records of parsing and warnings, through a test
  logger.

The golden snapshot changes whenever the output of the index does. To update it, run
`MALT_INDEX_UPDATE_GOLDEN=1 cargo test -p malt-index` and review the diff.

## Cross-check against MATLAB

The integration test `crates/malt_cli/tests/matlab_namespace.rs` compares the crate with MATLAB on the same folders:
names, the file each resolves to, `which -all`, signatures and class members. It runs `matlab -batch` and skips without
MATLAB. CI runs it, and it is required. See [Cross-check against MATLAB](../VERIFY.md).

## Dependencies

- `zip` reads `.mlx` archives, with `default-features = false` and the `deflate` feature, the smallest set that
  decompresses.
- `quick-xml` reads `document.xml`. It is event-based and builds no tree.
- `log` is the logging facade and has no dependencies.
- `serde_json` writes the dump.

The Python bindings add `pyo3`, and read the builtin table with `serde_json`. Exact versions are in the `Cargo.toml` of
each crate.
