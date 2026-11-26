# malt-index

<img src="https://raw.githubusercontent.com/watermarkhu/malt/main/img/malt-pixel.svg" alt="malt logo, created by Mark Shui Hu" width="100" align="right">

The MATLAB search path as a Python module. `malt-index` lists the functions, classes, properties, enumerations,
arguments, packages, folders and live scripts on the path, as read by the native index of
[malt](https://github.com/watermarkhu/malt). Nothing is evaluated: every value is a fact read from the source.

It is built for documentation generators, such as [mkdocstrings-matlab](https://github.com/watermarkhu/mkdocstrings-matlab),
and it is the successor of the Python library maxx. The distribution is `malt-index` and the module is `malt_index`.

## Install

```sh
pip install malt-index
```

Python 3.10 or later. One abi3 wheel serves every supported version, and the package ships type stubs and `py.typed`.

## Use

```python
from malt_index import PathsCollection

paths = PathsCollection(["src"], working_directory="/path/to/project")

fn = paths["pkg.fn"]
fn.kind, fn.filepath, fn.lineno
[argument.name for argument in fn.arguments]
fn.docstring
```

## Documentation

The API reference, the rules for how MATLAB is read, and a guide for users of maxx are in the
[malt documentation](https://watermarkhu.nl/malt/api/):

- [Python API](https://watermarkhu.nl/malt/api/python)
- [How MATLAB is read](https://watermarkhu.nl/malt/api/behaviour)
- [Coming from maxx](https://watermarkhu.nl/malt/api/maxx)
- [Development](https://watermarkhu.nl/malt/api/development)

## License

MIT. See the [repository](https://github.com/watermarkhu/malt) for the source and to report issues.
