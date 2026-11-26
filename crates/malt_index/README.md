# malt-index

<img src="https://raw.githubusercontent.com/watermarkhu/malt/main/img/malt-pixel.svg" alt="malt logo, created by Mark Shui Hu" width="100" align="right">

A native index of the MATLAB path: the functions, classes, scripts, live scripts, packages (`+pkg`), class folders
(`@cls`), plain folders and `private/` folders that MATLAB resolves names against. It is the Rust core of
[malt](https://github.com/watermarkhu/malt). The linter uses it when `[index] enabled = true` or `--index` is given, and
the Python module [`malt-index`](https://pypi.org/project/malt-index/) wraps it for documentation generators.

The crate does not depend on the linter crates.

## Use

```rust
use malt_index::{Index, ParserConfig};

let index = Index::scan(&[PathBuf::from("src")], &ParserConfig::default());
let entity = index.get("pkg.sub.fn")?;
let chain = index.mro(&class)?;
```

`Index::scan` lists the folders and reads no file. Files are parsed on first use and cached, and an `Index` is
`Send + Sync`.

## Documentation

The API tour, the rules for how MATLAB is read, and the development guide are in the
[malt documentation](https://watermarkhu.nl/malt/api/):

- [Rust API](https://watermarkhu.nl/malt/api/rust)
- [How MATLAB is read](https://watermarkhu.nl/malt/api/behaviour)
- [Development](https://watermarkhu.nl/malt/api/development)

The item documentation is on [docs.rs](https://docs.rs/malt-index).

## License

MIT. See the [repository](https://github.com/watermarkhu/malt) for the source and to report issues.
