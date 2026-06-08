#!/usr/bin/env python3
"""Dump the CPython 3.12 stdlib public-name surface as a checked-in JSON.

This script is Fire A of GitHub issue #1749 — the conformance "denominator".
It produces the machine-readable list of every public name in every stdlib
module that the mamba conformance suite is graded against (95%-surface gate
for conformance issues #1414–#1532).

## Deliverable shape

Output file: ``projects/mamba/data/cpython312_surface.json`` (relative to
the repo root). The file is committed; this script is the single source of
truth for regenerating it.

The umbrella issue (#1749) sketches a multi-section TOML schema with
``kind``, ``signature``, ``since``, ``deprecated`` fields. That richer
schema is deferred to Fires C/D when the diff/report tool needs it. For
Fire A — building the **denominator** — a flat ``{module: [names]}`` JSON
is sufficient and matches how each per-module conformance issue
(#1414–#1532) cites its surface count (e.g. "47 / 47 on dir(str)").

JSON schema::

    {
      "python_version": "3.12.x",
      "module_count": <int>,
      "total_name_count": <int>,
      "modules": {
        "<module.path>": {
          "names": ["<public_name>", ...],   # sorted, deduplicated
          "count": <int>,
          "uses_all": <bool>,                 # true iff __all__ was used
          "error": <string or null>           # import / introspection failure
        }
      }
    }

## "Public name" definition

For each module ``m`` we attempt to import it under the host's
``python3`` (must be 3.12.x). Then:

1. If ``m.__all__`` is defined, use it verbatim — this is the module's
   self-declared public surface, the strictest signal.
2. Else fall back to ``[n for n in dir(m) if not n.startswith("_")]`` —
   every non-underscore attribute reachable via ``dir()``.

Names are sorted ASCII-ascending and deduplicated.

## Module list

The hardcoded ``MODULES`` list is derived from the conformance issues
#1414–#1532 (titles of the form ``conformance(mamba/stdlib): <module>``)
plus core stdlib modules referenced by the umbrella issue (#1749) and
already-shipped mamba surfaces (functools, itertools, collections, enum,
struct, etc.).

When new conformance issues land, append to ``MODULES`` and re-run this
script. The output JSON is regenerable from this script + the running
CPython 3.12 — no other inputs.

## Usage

::

    python3 projects/mamba/scripts/dump_cpython_surface.py \\
        --output projects/mamba/data/cpython312_surface.json

Requires CPython 3.12. Refuses to run under any other minor version so
the committed JSON stays pinned to the documented denominator.
"""

from __future__ import annotations

import argparse
import importlib
import json
import sys
from pathlib import Path


# Modules graded by the conformance suite. Sourced from issue titles
# #1414–#1532 plus core-language stdlib called out in umbrella #1749.
# Keep this list sorted (ASCII) for reviewability; the JSON output
# preserves this order via dict-insertion-order.
MODULES: list[str] = [
    # Core builtins / language services
    "builtins",
    "sys",
    "types",
    "typing",
    # Text / data
    "string",
    "re",
    "struct",
    "codecs",
    "unicodedata",
    "stringprep",
    # Numeric
    "math",
    "cmath",
    "decimal",
    "fractions",
    "random",
    "statistics",
    "numbers",
    # Functional / iteration
    "functools",
    "itertools",
    "operator",
    # Containers
    "collections",
    "collections.abc",
    "array",
    "heapq",
    "bisect",
    "weakref",
    "copy",
    "pprint",
    "reprlib",
    "enum",
    "dataclasses",
    "contextvars",
    # OS / filesystem
    "os",
    "os.path",
    "pathlib",
    "shutil",
    "tempfile",
    "glob",
    "fnmatch",
    "linecache",
    "stat",
    "filecmp",
    "fileinput",
    # I/O / serialization
    "io",
    "pickle",
    "copyreg",
    "shelve",
    "marshal",
    "csv",
    "json",
    "configparser",
    "tomllib",
    # Compression / archives
    "gzip",
    "bz2",
    "lzma",
    "zipfile",
    "tarfile",
    "zlib",
    # Crypto / hashing / secrets
    "hashlib",
    "hmac",
    "secrets",
    "base64",
    "binascii",
    "uuid",
    # Date / time
    "datetime",
    "time",
    "calendar",
    "zoneinfo",
    # Networking
    "socket",
    "ssl",
    "selectors",
    "ipaddress",
    "asyncio",
    # HTTP / URL
    "http",
    "http.client",
    "http.server",
    "http.cookies",
    "http.cookiejar",
    "urllib",
    "urllib.parse",
    "urllib.request",
    "urllib.response",
    "urllib.error",
    "urllib.robotparser",
    # Email / MIME
    "email",
    "email.message",
    "email.parser",
    "email.policy",
    "email.utils",
    "mimetypes",
    # Markup
    "html",
    "html.parser",
    "html.entities",
    "xml",
    "xml.etree.ElementTree",
    "xml.sax",
    "xml.dom",
    "xml.dom.minidom",
    # Concurrency / processes
    "threading",
    "multiprocessing",
    "subprocess",
    "queue",
    "concurrent.futures",
    "signal",
    "sched",
    # Database
    "sqlite3",
    "dbm",
    # Logging / errors / warnings
    "logging",
    "logging.config",
    "logging.handlers",
    "warnings",
    "traceback",
    # Inspection / runtime / debugging
    "inspect",
    "ast",
    "dis",
    "tokenize",
    "keyword",
    "symtable",
    "gc",
    "atexit",
    "argparse",
    "getopt",
    "getpass",
    "platform",
    "errno",
    "ctypes",
    # Development tools
    "unittest",
    "unittest.mock",
    "doctest",
    "pdb",
    "trace",
    "timeit",
    "cProfile",
    "profile",
    "pstats",
    # Misc
    "abc",
    "io",
    "locale",
    "gettext",
    "textwrap",
    "difflib",
]


def public_names(module: object) -> tuple[list[str], bool]:
    """Return (sorted_unique_public_names, uses_all)."""
    explicit = getattr(module, "__all__", None)
    if isinstance(explicit, (list, tuple)) and all(
        isinstance(n, str) for n in explicit
    ):
        return sorted(set(explicit)), True
    return sorted({n for n in dir(module) if not n.startswith("_")}), False


def introspect(name: str) -> dict:
    try:
        mod = importlib.import_module(name)
    except Exception as exc:  # noqa: BLE001 — record any import failure
        return {
            "names": [],
            "count": 0,
            "uses_all": False,
            "error": f"{type(exc).__name__}: {exc}",
        }
    names, uses_all = public_names(mod)
    return {
        "names": names,
        "count": len(names),
        "uses_all": uses_all,
        "error": None,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=(__doc__ or "").splitlines()[0])
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).resolve().parents[1]
        / "data"
        / "cpython312_surface.json",
        help="Path to write the JSON denominator (default: "
        "projects/mamba/data/cpython312_surface.json).",
    )
    args = parser.parse_args()

    if sys.version_info[:2] != (3, 12):
        print(
            f"error: this script must run under CPython 3.12 "
            f"(got {sys.version.split()[0]}); the committed JSON is pinned "
            f"to 3.12 so other versions would silently shift the "
            f"denominator.",
            file=sys.stderr,
        )
        return 2

    modules: dict[str, dict] = {}
    for name in MODULES:
        modules[name] = introspect(name)

    total = sum(entry["count"] for entry in modules.values())
    payload = {
        "python_version": sys.version.split()[0],
        "module_count": len(modules),
        "total_name_count": total,
        "modules": modules,
    }

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8") as fh:
        json.dump(payload, fh, indent=2, ensure_ascii=False, sort_keys=False)
        fh.write("\n")
    print(
        f"wrote {args.output} — {len(modules)} modules, {total} public names"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
