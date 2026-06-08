#!/usr/bin/env python3
"""fixture_gen.py — manifest -> mamba conformance fixture generator.

Reads a per-lib manifest TOML (one [[case]] table per fixture) and emits one
self-contained PEP 723 fixture file per case under

    tests/cpython/fixtures/<bucket>/<lib>/<dimension>/<case>.py

Every emitted file embeds a [tool.mamba] metadata table inside its PEP 723
'# /// script' block. Mechanical cases (surface / errors with a declared probe
or call) are written complete and CPython-green. Cases with an explicit `body`
are written complete. Other semantic cases (behavior / security / real_world,
plus any case marked kind="semantic") are written as skeletons: the inferred
imports + an AGENT-FILL marker + an UNFILLED SystemExit, with
status="generated" so a downstream agent fills only the body.

The generator is IDEMPOTENT: a file whose [tool.mamba].status == "filled" is
never overwritten; a "generated" skeleton may be regenerated; new files are
created.

CLI:
    python fixture_gen.py <manifest.toml>     # one lib
    python fixture_gen.py --all               # walk config/manifests/

stdlib only (tomllib, pathlib, re, argparse, sys).
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

# ---------------------------------------------------------------------------
# Repo geometry
# ---------------------------------------------------------------------------

TOOLS_DIR = Path(__file__).resolve().parent
CPYTHON_DIR = TOOLS_DIR.parent / "tests" / "cpython"
FIXTURES_ROOT = CPYTHON_DIR / "fixtures"
MANIFESTS_ROOT = CPYTHON_DIR / "config" / "manifests"

BUCKETS = {"core", "builtin-libs", "std-libs", "pep", "type-strict", "3rd-libs"}
DIMENSIONS = {"surface", "behavior", "errors", "bench", "real_world", "security"}
SEMANTIC_DIMENSIONS = {"behavior", "security", "real_world"}

# ---------------------------------------------------------------------------
# PEP 723 / [tool.mamba] block helpers (shared shape with the linter)
# ---------------------------------------------------------------------------

BLOCK_RE = re.compile(
    r"^# /// script[ \t]*\n(.*?)^# ///[ \t]*$",
    re.DOTALL | re.MULTILINE,
)


def extract_block_toml(text: str) -> str | None:
    """Return the TOML source inside the '# /// script ... # ///' block.

    Strips the '# ' / '#' line prefixes exactly as the linter does, so the
    round-trip is identical on both sides.
    """
    m = BLOCK_RE.search(text)
    if not m:
        return None
    lines = []
    for raw in m.group(1).splitlines():
        if raw.startswith("# "):
            lines.append(raw[2:])
        elif raw == "#":
            lines.append("")
        else:
            # Unprefixed line inside the block — invalid PEP 723.
            return None
    return "\n".join(lines)


def read_status(path: Path) -> str | None:
    """Return [tool.mamba].status of an existing fixture, or None."""
    if not path.exists():
        return None
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    toml_src = extract_block_toml(text)
    if toml_src is None:
        return None
    try:
        data = tomllib.loads(toml_src)
    except tomllib.TOMLDecodeError:
        return None
    return data.get("tool", {}).get("mamba", {}).get("status")


# ---------------------------------------------------------------------------
# Import inference
# ---------------------------------------------------------------------------


def infer_module(subject: str) -> str:
    """Best-effort importable module for a dotted subject.

    Heuristic: a module path component is module-shaped (starts lowercase);
    a class is CapWords. Stop at the first CapWords component.

      'calendar.isleap'                  -> 'calendar'   (func leaf dropped)
      'calendar.day_name'                -> 'calendar'   (attr leaf dropped)
      'calendar.TextCalendar.formatmonth'-> 'calendar'   (stop at class)
      'calendar'                         -> 'calendar'

    Ambiguous cases (e.g. a CapWords *module* like
    'xml.etree.ElementTree.fromstring') can't be inferred from the name —
    the manifest must set an explicit `module` or `import` for those.
    """
    parts = subject.split(".")
    if len(parts) == 1:
        return parts[0]
    mod_parts: list[str] = []
    for p in parts:
        if p[:1].isupper():  # a class component — module ends before it
            break
        mod_parts.append(p)
    if not mod_parts:
        return parts[0]
    if len(mod_parts) == len(parts):
        # all components are module-shaped: the leaf is a func/attr — drop it
        return ".".join(parts[:-1])
    return ".".join(mod_parts)


def import_stmt(case: dict, meta: dict) -> str:
    """The import line for a fixture: explicit override or inferred module."""
    if case.get("import"):
        return str(case["import"])
    mod = case.get("module") or infer_module(meta["subject"])
    return f"import {mod}"


# ---------------------------------------------------------------------------
# Metadata block rendering
# ---------------------------------------------------------------------------

# Order is fixed so regenerated files are byte-stable.
_META_ORDER = [
    "bucket",
    "lib",
    "dimension",
    "case",
    "subject",
    "kind",
    "xfail",
    "mem_carveout",
    "source",
    "status",
]


def _toml_str(value: str) -> str:
    """Render a Python string as a TOML basic string literal."""
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def _safe_doc(text: str) -> str:
    """Make text safe to drop inside a triple-quoted docstring.

    Neutralizes an embedded triple-quote sequence and a trailing double
    quote, either of which would otherwise fuse with the closing delimiter
    and break the file.
    """
    text = text.replace("\\", "\\\\").replace('"""', '\\"\\"\\"')
    if text.endswith('"'):
        text += " "
    return text


def render_header(meta: dict) -> str:
    """Render the full PEP 723 + [tool.mamba] header (no trailing blank)."""
    lines = [
        "# /// script",
        '# requires-python = ">=3.12"',
        "# dependencies = []",
        "#",
        "# [tool.mamba]",
    ]
    for key in _META_ORDER:
        if key in meta:
            lines.append(f"# {key} = {_toml_str(str(meta[key]))}")
    lines.append("# ///")
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Body rendering
# ---------------------------------------------------------------------------


def render_mechanical_surface(case: dict, meta: dict) -> str:
    subject = meta["subject"]
    name = meta["case"]
    probe = case.get("probe", "callable")
    imp = import_stmt(case, meta)
    expr = str(case.get("expr", subject))

    if probe == "callable":
        check = f"assert callable({expr})"
    elif probe == "not_callable":
        check = f"assert not callable({expr})"
    elif probe == "attr":
        attr = case["attr"]
        check = f'assert hasattr({expr}, "{attr}")'
    elif probe == "type":
        typename = case["typename"]
        check = f"assert type({expr}).__name__ == {_toml_str(typename)}"
    elif probe == "sequence":
        if "length" in case:
            check = f"assert len({expr}) == {int(case['length'])}"
        else:
            check = f"assert hasattr({expr}, \"__len__\")"
    else:
        raise ValueError(f"unknown surface probe {probe!r} for case {name!r}")

    return (
        f"{imp}\n\n"
        f"{check}\n"
        f'print("{name} OK")\n'
    )


def render_mechanical_errors(case: dict, meta: dict) -> str:
    # SELF-ASSERT (assert-it-raised): the fixture asserts the documented
    # exception actually fires, so `mamba run` exit-0 is a complete gate —
    # a missing raise (mamba silently returns where CPython raises) becomes
    # a failed assert -> nonzero exit -> caught, instead of a false PASS.
    # `except <expect>` catches the exact type or a subclass; a wrong/no
    # exception leaves _raised False (or propagates) -> assertion fails.
    name = meta["case"]
    call = case["call"]
    expect = case["expect_exc"]
    imp = import_stmt(case, meta)
    return (
        f"{imp}\n\n"
        f"_raised = False\n"
        f"try:\n"
        f"    {call}\n"
        f"except {expect}:\n"
        f"    _raised = True\n"
        f'assert _raised, "{name}: expected {expect}"\n'
        f'print("{name} OK")\n'
    )


def render_mechanical_type_strict(case: dict, meta: dict) -> str:
    """Render a strict-typing probe.

    The fixture is CPython-green because it catches TypeError and prints a
    marker. The mamba runner classifies the mamba side: `typeerror:` is the
    required strict result; `no_typeerror:` means mamba leaked a wrong-typed
    call through.
    """
    name = meta["case"]
    call = case["call"]
    setup = str(case.get("setup", "")).strip()
    imp = str(case.get("import", "")).strip()

    prefix = ""
    if imp:
        prefix += f"{imp}\n"
    if setup:
        prefix += f"{setup}\n"
    if prefix:
        prefix += "\n"

    return (
        f"{prefix}"
        f"try:\n"
        f"    result = {call}\n"
        f'    print("no_typeerror:", repr(result))\n'
        f"except TypeError as e:\n"
        f'    print("typeerror:", type(e).__name__, str(e)[:80])\n'
    )


def render_semantic(case: dict, meta: dict) -> str:
    subject = meta["subject"]
    name = meta["case"]
    intent = case.get("intent", f"exercise {subject}")
    imp = import_stmt(case, meta)
    return (
        f"{imp}\n\n"
        f"# >>> AGENT-FILL: {intent}\n"
        f'raise SystemExit("UNFILLED: {name}")\n\n'
        f'print("{name} OK")\n'
    )


# ---------------------------------------------------------------------------
# Case -> file
# ---------------------------------------------------------------------------


def build_meta(bucket: str, lib: str, case: dict) -> dict:
    dimension = case["dimension"]
    name = case["case"]
    subject = case["subject"]
    kind = case["kind"]

    has_explicit_body = "body" in case
    is_semantic = kind == "semantic" or dimension in SEMANTIC_DIMENSIONS
    status = "filled" if has_explicit_body else ("generated" if is_semantic else "filled")

    meta = {
        "bucket": bucket,
        "lib": lib,
        "dimension": dimension,
        "case": name,
        "subject": subject,
        "kind": kind,
        "xfail": case.get("xfail", ""),
        "mem_carveout": case.get("mem_carveout", ""),
        "source": case.get("source", ""),
        "status": status,
    }
    return meta


def render_file(bucket: str, lib: str, case: dict) -> str:
    meta = build_meta(bucket, lib, case)
    dimension = meta["dimension"]
    kind = meta["kind"]
    name = meta["case"]
    subject = meta["subject"]

    header = render_header(meta)
    if bucket == "type-strict":
        header = f"{header}\n# mamba-strict-type: TypeError"

    if "body" in case:
        intent = case.get("intent", f"exercise {subject}")
        docstring = f'"""{_safe_doc(f"{subject}: {intent}")}"""'
        body = str(case["body"]).rstrip() + "\n"
    else:
        is_semantic = kind == "semantic" or dimension in SEMANTIC_DIMENSIONS
        if is_semantic:
            intent = case.get("intent", f"exercise {subject}")
            docstring = f'"""{_safe_doc(f"{subject}: {intent}")}"""'
            body = render_semantic(case, meta)
        else:
            docstring = f'"""{_safe_doc(f"{subject}: {name} ({dimension}).")}"""'
            if bucket == "type-strict" and "call" in case:
                body = render_mechanical_type_strict(case, meta)
            elif dimension == "surface":
                body = render_mechanical_surface(case, meta)
            elif dimension == "errors":
                body = render_mechanical_errors(case, meta)
            else:
                # Mechanical bench is not auto-bodied; treat as a generated skeleton.
                body = render_semantic(case, meta)

    return f"{header}\n{docstring}\n{body}"


def output_path(bucket: str, lib: str, case: dict) -> Path:
    return FIXTURES_ROOT / bucket / lib / case["dimension"] / f"{case['case']}.py"


# ---------------------------------------------------------------------------
# Manifest processing
# ---------------------------------------------------------------------------


def validate_case(bucket: str, lib: str, case: dict, idx: int) -> None:
    where = f"{bucket}/{lib} case[{idx}]"
    for key in ("dimension", "case", "subject", "kind"):
        if key not in case:
            raise ValueError(f"{where}: missing required key {key!r}")
    if case["dimension"] not in DIMENSIONS:
        raise ValueError(
            f"{where}: dimension {case['dimension']!r} not in {sorted(DIMENSIONS)}"
        )
    if case["kind"] not in ("mechanical", "semantic"):
        raise ValueError(f"{where}: kind must be 'mechanical' or 'semantic'")
    stem = case["case"]
    if not re.fullmatch(r"[a-z0-9]+(?:_[a-z0-9]+)*", stem):
        raise ValueError(f"{where}: case {stem!r} is not snake_case")


def process_manifest(path: Path, *, dry_run: bool = False) -> list[tuple[Path, str]]:
    """Generate all files for one manifest. Returns [(path, action), ...]."""
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    bucket = data.get("bucket")
    lib = data.get("lib")
    if bucket not in BUCKETS:
        raise ValueError(f"{path}: bucket {bucket!r} not in {sorted(BUCKETS)}")
    if not lib:
        raise ValueError(f"{path}: missing top-level lib")

    results: list[tuple[Path, str]] = []
    for idx, case in enumerate(data.get("case", [])):
        validate_case(bucket, lib, case, idx)
        out = output_path(bucket, lib, case)
        existing_status = read_status(out)
        is_semantic = (case["kind"] == "semantic"
                       or case["dimension"] in SEMANTIC_DIMENSIONS)
        has_explicit_body = "body" in case

        # Idempotency rule: never clobber a hand-FILLED SEMANTIC body (the
        # agent's work). MECHANICAL files are fully determined by the manifest
        # (no hand edits), so they always re-emit — this lets a template
        # change (e.g. the errors assert-it-raised flip) propagate on re-run.
        if existing_status == "filled" and is_semantic and not has_explicit_body:
            results.append((out, "skip-filled"))
            continue

        content = render_file(bucket, lib, case)
        action = ("regenerate" if existing_status in ("generated", "filled")
                  else "create")
        if not dry_run:
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(content, encoding="utf-8")
        results.append((out, action))

    return results


def iter_manifests(root: Path):
    for p in sorted(root.rglob("*.toml")):
        yield p


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def main(argv: list[str] | None = None) -> int:
    _doc = (__doc__ or "").splitlines()
    ap = argparse.ArgumentParser(description=_doc[0] if _doc else "fixture generator")
    ap.add_argument("manifest", nargs="?", help="path to a single manifest TOML")
    ap.add_argument(
        "--all",
        action="store_true",
        help=f"walk {MANIFESTS_ROOT} and generate every manifest",
    )
    ap.add_argument(
        "--dry-run",
        action="store_true",
        help="report planned actions without writing",
    )
    args = ap.parse_args(argv)

    if args.all:
        manifests = list(iter_manifests(MANIFESTS_ROOT))
        if not manifests:
            print(f"no manifests under {MANIFESTS_ROOT}", file=sys.stderr)
            return 0
    elif args.manifest:
        manifests = [Path(args.manifest).resolve()]
    else:
        ap.error("give a manifest path or --all")  # raises SystemExit(2)

    totals = {"create": 0, "regenerate": 0, "skip-filled": 0}
    for mpath in manifests:
        results = process_manifest(mpath, dry_run=args.dry_run)
        for out, action in results:
            totals[action] += 1
            rel = out.relative_to(FIXTURES_ROOT)
            print(f"[{action:>11}] {rel}")

    prefix = "(dry-run) " if args.dry_run else ""
    print(
        f"{prefix}created={totals['create']} "
        f"regenerated={totals['regenerate']} "
        f"skipped-filled={totals['skip-filled']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
