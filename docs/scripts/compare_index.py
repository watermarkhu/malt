#!/usr/bin/env python3
"""Compare the malt_index dump with maxx's dump of the same MATLAB paths (the oracle).

Both inputs use the schema of maxx_dump.py (see crates/malt_index/README.md). Every
difference is printed, grouped by field, with the entity path and its context. Each
difference is either accepted by an allowlist entry (documented, with a reason) or
reported as unresolved. The exit code is 1 when any difference is unresolved.

    python3 -I docs/scripts/compare_index.py --maxx maxx.json --rust rust.json \\
        --project NAME [--allow crates/malt_index/oracle_allowlist.json]

Produce maxx.json inside a maxx checkout (uv works there):

    cd ../maxx && uv run python ../malt/docs/scripts/maxx_dump.py PATH... > maxx.json

Produce rust.json from the malt checkout:

    cargo run -q -p malt-index --example dump -- PATH... > rust.json

Allowlist file: a JSON list of objects with keys
    "reason"  (required) why the difference is accepted and what malt does instead,
    "project" (optional, glob) which project it applies to, default "*",
    "entity"  (optional, glob) the entity path (dotted, e.g. "Widget.Size"), default "*",
    "field"   (optional, glob) the field (e.g. "kind", "lineno", "arguments.kind",
              "presence"), default "*".
    "detail"  (optional, glob) what the difference is about: the argument name for
              "arguments.*" and "returns.*"; for "presence", 'parent>kind' of the
              member (for example 'function>function' for a local function);
              for "attributes", 'kind:attributes that differ'; otherwise the entity
              kind. Default "*".
Nothing is copied from maxx: the file only names differences and reasons.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import sys
from collections import Counter
from pathlib import Path
from typing import Any

DEFAULT_ALLOWLIST = Path(__file__).resolve().parents[2] / "crates" / "malt_index" / "oracle_allowlist.json"
SCALAR_FIELDS = (
    "kind",
    "name",
    "lineno",
    "endlineno",
    "docstring",
    "path",
    "attributes",
    "bases",
    "is_getter",
    "is_setter",
    "type",
    "dimensions",
    "validators",
    "default",
)
ARGUMENT_FIELDS = ("kind", "type", "dimensions", "validators", "default", "docstring")
ABSENT = "<absent>"


class Difference:
    def __init__(self, entity: str, field: str, maxx: Any, rust: Any, context: str, detail: str = "") -> None:
        self.entity = entity
        self.field = field
        self.maxx = maxx
        self.rust = rust
        self.context = context
        # What the difference is about: an argument, an attribute, or the entity kind.
        self.detail = detail


def describe(node: dict | None) -> str:
    """The context of an entity: kind, span, attributes, bases, arguments, docstring."""
    if node is None:
        return "(absent)"
    parts = [str(node.get("kind"))]
    if node.get("lineno") is not None or node.get("endlineno") is not None:
        parts.append(f"lines {node.get('lineno')}-{node.get('endlineno')}")
    else:
        parts.append("no span")
    if node.get("path"):
        parts.append(f"file {node['path']}")
    if node.get("attributes"):
        parts.append("attributes " + ",".join(node["attributes"]))
    if node.get("bases"):
        parts.append("bases " + ",".join(node["bases"]))
    if node.get("arguments"):
        parts.append("arguments (" + ", ".join(a["name"] for a in node["arguments"]) + ")")
    if node.get("docstring"):
        first = node["docstring"].strip().splitlines()[0] if node["docstring"].strip() else ""
        parts.append("doc " + repr(first[:40]))
    return "; ".join(parts)


def compare_values(
    out: list[Difference], entity: str, field: str, maxx: Any, rust: Any, context: str, detail: str = ""
) -> None:
    if maxx != rust:
        out.append(Difference(entity, field, maxx, rust, context, detail))


def compare_arguments(out: list[Difference], entity: str, context: str, field: str, a: list, b: list) -> None:
    a_by_name = {arg["name"]: arg for arg in a}
    b_by_name = {arg["name"]: arg for arg in b}
    if [arg["name"] for arg in a] != [arg["name"] for arg in b] and set(a_by_name) == set(b_by_name):
        out.append(Difference(entity, f"{field}.order", [x["name"] for x in a], [x["name"] for x in b], context))
    for name in a_by_name.keys() - b_by_name.keys():
        out.append(Difference(entity, f"{field}.presence", f"{name} present", f"{name} absent", context, name))
    for name in b_by_name.keys() - a_by_name.keys():
        out.append(Difference(entity, f"{field}.presence", f"{name} absent", f"{name} present", context, name))
    for name in a_by_name.keys() & b_by_name.keys():
        for sub in ARGUMENT_FIELDS:
            compare_values(
                out,
                entity,
                f"{field}.{sub}",
                a_by_name[name].get(sub),
                b_by_name[name].get(sub),
                f"{context}; {field} {name}",
                name,
            )


def compare_entity(out: list[Difference], entity: str, a: dict | None, b: dict | None) -> None:
    """Compare two entities (maxx side a, Rust side b) and recurse into their members."""
    context = describe(a if a is not None else b)
    if a is None or b is None:
        out.append(Difference(entity, "presence", "present" if a else "absent", "present" if b else "absent", context))
        return
    kind = b.get("kind", "")
    for field in SCALAR_FIELDS:
        if (field in a) != (field in b):
            out.append(Difference(entity, field, a.get(field, ABSENT), b.get(field, ABSENT), context, kind))
        elif field == "attributes":
            # The detail is the attributes that differ, so an allowlist can name one.
            only = sorted(set(a.get(field) or []) ^ set(b.get(field) or []))
            compare_values(out, entity, field, a.get(field), b.get(field), context, f"{kind}:{','.join(only)}")
        else:
            compare_values(out, entity, field, a.get(field), b.get(field), context, kind)
    if "arguments" in a or "arguments" in b:
        compare_arguments(out, entity, context, "arguments", a.get("arguments", []), b.get("arguments", []))
    if "returns" in a or "returns" in b:
        compare_arguments(out, entity, context, "returns", a.get("returns", []), b.get("returns", []))
    compare_members(out, entity, a.get("members", {}), b.get("members", {}), a.get("kind", ""))


def compare_members(out: list[Difference], prefix: str, a: dict, b: dict, parent_kind: str = "root") -> None:
    """Compare two member maps. A presence difference's detail is 'parent>kind' of the
    entity that exists on one side, so an allowlist can name a kind of member."""
    for name in sorted(set(a) | set(b)):
        child = name if not prefix else f"{prefix}.{name}"
        if name not in a:
            context = describe(b[name])
            detail = f"{parent_kind}>{b[name].get('kind')}"
            out.append(Difference(child, "presence", "absent", "present", context, detail))
            continue
        if name not in b:
            context = describe(a[name])
            detail = f"{parent_kind}>{a[name].get('kind')}"
            out.append(Difference(child, "presence", "present", "absent", context, detail))
            continue
        compare_entity(out, child, a[name], b[name])


def load_allowlist(path: Path | None) -> list[dict]:
    if path is None or not path.exists():
        return []
    entries = json.loads(path.read_text(encoding="utf-8"))
    for entry in entries:
        if not str(entry.get("reason", "")).strip():
            raise SystemExit(f"allowlist entry without a reason: {entry}")
    return entries


def allowed(diff: Difference, project: str, entries: list[dict]) -> str | None:
    for entry in entries:
        if (
            fnmatch.fnmatchcase(project, entry.get("project", "*"))
            and fnmatch.fnmatchcase(diff.entity, entry.get("entity", "*"))
            and fnmatch.fnmatchcase(diff.field, entry.get("field", "*"))
            and fnmatch.fnmatchcase(diff.detail, entry.get("detail", "*"))
        ):
            return entry["reason"]
    return None


def fmt(value: Any) -> str:
    text = json.dumps(value, ensure_ascii=False)
    return text if len(text) <= 70 else text[:67] + "..."


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--maxx", type=Path, required=True)
    parser.add_argument("--rust", type=Path, required=True)
    parser.add_argument("--project", default="project")
    parser.add_argument("--allow", type=Path, default=DEFAULT_ALLOWLIST)
    parser.add_argument("--quiet", action="store_true", help="print only the summary")
    args = parser.parse_args(argv)

    maxx = json.loads(args.maxx.read_text(encoding="utf-8"))
    rust = json.loads(args.rust.read_text(encoding="utf-8"))
    differences: list[Difference] = []
    compare_members(differences, "", maxx.get("members", {}), rust.get("members", {}))
    entries = load_allowlist(args.allow)

    unresolved: list[Difference] = []
    accepted: list[tuple[Difference, str]] = []
    for diff in differences:
        reason = allowed(diff, args.project, entries)
        if reason is None:
            unresolved.append(diff)
        else:
            accepted.append((diff, reason))

    if not args.quiet:
        print(f"== {args.project}: {len(differences)} differences ({len(accepted)} accepted, {len(unresolved)} unresolved)")
        for label, group in (("UNRESOLVED", unresolved), ("ACCEPTED", [d for d, _ in accepted])):
            if not group:
                continue
            print(f"\n-- {label}, grouped by field")
            by_field: dict[str, list[Difference]] = {}
            for diff in group:
                by_field.setdefault(diff.field, []).append(diff)
            for field in sorted(by_field):
                print(f"\n[{field}] {len(by_field[field])}")
                for diff in by_field[field]:
                    print(f"  {diff.entity}  ({diff.context})")
                    print(f"      maxx={fmt(diff.maxx)}  rust={fmt(diff.rust)}  detail={diff.detail!r}")
                    if label == "ACCEPTED":
                        reason = next(r for d, r in accepted if d is diff)
                        print(f"      accepted: {reason}")

    counts: Counter = Counter()
    accepted_counts: Counter = Counter()
    for diff in unresolved:
        counts[diff.field] += 1
    for diff, _ in accepted:
        accepted_counts[diff.field] += 1
    fields = sorted(set(counts) | set(accepted_counts))
    print(f"\n== summary: {args.project}")
    print(f"{'field':<28}{'unresolved':>12}{'accepted':>10}")
    for field in fields:
        print(f"{field:<28}{counts[field]:>12}{accepted_counts[field]:>10}")
    print(f"{'TOTAL':<28}{len(unresolved):>12}{len(accepted):>10}")
    return 1 if unresolved else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
