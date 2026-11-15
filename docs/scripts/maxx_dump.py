"""Dump what maxx collects from MATLAB paths as JSON: the oracle of the malt_index crate.

Run it inside a maxx checkout, which provides the `maxx` package:

    cd ../maxx && uv run python ../malt/docs/scripts/maxx_dump.py PATH [PATH ...] > maxx.json

The output is the schema documented in crates/malt_index/README.md: a tree of entities,
each with kind, name, path (relative to the first PATH), line range, attributes,
bases, arguments, outputs and members. Docstrings are the raw text as written.
Only facts are compared: nothing here is copied into the repository.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def text(value) -> str | None:
    """A type, default or validator as the text it was written with."""
    if value is None:
        return None
    return str(value)


def docstring(obj) -> str | None:
    doc = getattr(obj, "docstring", None)
    if doc is None:
        return None
    return getattr(doc, "value", str(doc))


def argument(arg) -> dict:
    return {
        "name": arg.name,
        "kind": None if arg.kind is None else str(arg.kind.value),
        "type": text(arg.type),
        "dimensions": None if arg.dimensions is None else [str(d) for d in arg.dimensions],
        "validators": text(arg.validators),
        "default": text(arg.default),
        "docstring": docstring(arg),
    }


def resolve(obj):
    """The object an alias points at (the collection hands out aliases)."""
    seen = 0
    while getattr(obj, "is_alias", False) and seen < 10:
        obj = obj.target
        seen += 1
    return obj


def entity(obj, root: Path) -> dict:
    out: dict = {
        "kind": str(obj.kind.value),
        "name": obj.name,
        "lineno": obj.lineno,
        "endlineno": obj.endlineno,
        "docstring": docstring(obj),
    }
    path = getattr(obj, "filepath", None) or getattr(obj, "path", None)
    if path is not None:
        try:
            out["path"] = str(Path(path).resolve().relative_to(root))
        except ValueError:
            out["path"] = str(path)
    attributes = getattr(obj, "attributes", None)
    if attributes is not None:
        out["attributes"] = sorted(str(a) for a in attributes)
    if hasattr(obj, "bases") and obj.kind.value == "class":
        out["bases"] = list(obj.bases)
    if hasattr(obj, "arguments") and obj.kind.value == "function":
        out["arguments"] = [argument(a) for a in obj.arguments]
        out["returns"] = [argument(a) for a in obj.returns]
        out["is_getter"] = bool(getattr(obj, "is_getter", False))
        out["is_setter"] = bool(getattr(obj, "is_setter", False))
    if obj.kind.value == "property":
        out["type"] = text(getattr(obj, "type", None))
        dims = getattr(obj, "dimensions", None)
        out["dimensions"] = None if dims is None else [str(d) for d in dims]
        out["validators"] = text(getattr(obj, "validators", None))
        out["default"] = text(getattr(obj, "default", None))
    members = {}
    for name, member in sorted(getattr(obj, "members", {}).items()):
        if getattr(member, "is_alias", False):
            continue  # imports and inherited members are not part of the structure
        members[name] = entity(member, root)
    out["members"] = members
    return out


def main(argv: list[str]) -> int:
    from maxx.collection import PathsCollection

    paths = [Path(p).resolve() for p in argv]
    if not paths:
        print("usage: maxx_dump.py PATH [PATH ...]", file=sys.stderr)
        return 2
    root = paths[0]
    collection = PathsCollection(paths)
    result = {
        "root": str(root),
        "members": {
            name: entity(resolve(member), root)
            for name, member in sorted(collection.members.items())
        },
    }
    json.dump(result, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
