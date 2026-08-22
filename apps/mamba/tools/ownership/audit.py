#!/usr/bin/env python3
"""Emit and verify the deterministic Mamba ownership-constructor inventory."""

from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import asdict, dataclass
import hashlib
import json
from pathlib import Path
import re
import sys
import tempfile

from provenance import UNCLASSIFIED, classify
from rust_scan import ScanError, rust_files, scan_calls


TOOL_DIR = Path(__file__).resolve().parent
PROJECT_DIR = TOOL_DIR.parents[1]
DEFAULT_ROOT = PROJECT_DIR / "src" / "runtime"
FIXTURES = TOOL_DIR / "fixtures"


@dataclass(frozen=True)
class Site:
    site_id: str
    path: str
    symbol: str
    constructor: str
    constructor_contract: str
    classification: str
    origin_fingerprint: str
    evidence: str
    presentation_line: int


def display_path(path: Path, scan_root: Path) -> str:
    try:
        return path.resolve().relative_to(PROJECT_DIR.parent.parent.resolve()).as_posix()
    except ValueError:
        if scan_root.is_file():
            return scan_root.name
        return path.relative_to(scan_root).as_posix()


def inventory(scan_root: Path) -> dict[str, object]:
    rows: list[Site] = []
    diagnostics: list[str] = []
    semantic_occurrences: Counter[str] = Counter()
    files = rust_files(scan_root)
    if not files:
        diagnostics.append(f"{scan_root}: no Rust source files")
    sources = {path: path.read_text(encoding="utf-8") for path in files}
    known_helpers = frozenset(
        match.group(1)
        for source in sources.values()
        for match in re.finditer(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)", source)
    )
    for path in files:
        source = sources[path]
        calls, call_diagnostics = scan_calls(source)
        diagnostics.extend(f"{display_path(path, scan_root)}: {item}" for item in call_diagnostics)
        for call in calls:
            argument = source[call.argument_start : call.argument_end]
            function_prefix = source[call.function_start : call.start]
            try:
                origin = classify(
                    argument,
                    function_prefix,
                    constructor=call.constructor,
                    known_helpers=known_helpers,
                )
            except ScanError as error:
                origin = classify("", "", constructor=call.constructor)
                origin = type(origin)(UNCLASSIFIED, "truncated", str(error))
            identity = "\0".join(
                (
                    display_path(path, scan_root),
                    call.symbol,
                    call.constructor,
                    origin.fingerprint,
                )
            )
            semantic_occurrences[identity] += 1
            occurrence = semantic_occurrences[identity]
            digest_input = f"{identity}\0{occurrence}".encode()
            site_id = hashlib.sha256(digest_input).hexdigest()
            rows.append(
                Site(
                    site_id=site_id,
                    path=display_path(path, scan_root),
                    symbol=call.symbol,
                    constructor=call.constructor,
                    constructor_contract=(
                        "RETAINS_BORROWED"
                        if call.constructor.endswith("_borrowed")
                        else "CONSUMES_OWNED"
                    ),
                    classification=origin.classification,
                    origin_fingerprint=origin.fingerprint,
                    evidence=origin.evidence,
                    presentation_line=call.line,
                )
            )
    rows.sort(key=lambda row: row.site_id)
    row_dicts = [asdict(row) for row in rows]
    counts = Counter(row.classification for row in rows)
    constructors = Counter(row.constructor for row in rows)
    normalized_rows = [
        {key: value for key, value in row.items() if key != "presentation_line"}
        for row in row_dicts
    ]
    inventory_digest = hashlib.sha256(
        json.dumps(normalized_rows, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()
    return {
        "schema": "mamba.ownership-sites.v1",
        "total": len(rows),
        "counts": dict(sorted(counts.items())),
        "constructors": dict(sorted(constructors.items())),
        "inventory_digest": inventory_digest,
        "diagnostics": sorted(diagnostics),
        "sites": row_dicts,
    }


def encoded(report: dict[str, object]) -> bytes:
    return (json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n").encode()


def require(condition: bool, message: str, failures: list[str]) -> None:
    if not condition:
        failures.append(message)


def fixture_report(name: str) -> dict[str, object]:
    return inventory(FIXTURES / name)


def run_checks() -> int:
    failures: list[str] = []
    first = inventory(DEFAULT_ROOT)
    second = inventory(DEFAULT_ROOT)
    require(encoded(first) == encoded(second), "unchanged-tree output differs", failures)
    require(first["total"] > 0, "current source inventory is empty", failures)
    require(first["total"] == len(first["sites"]), "row total does not reconcile", failures)
    require(sum(first["counts"].values()) == first["total"], "class counts do not reconcile", failures)
    require(
        sum(first["constructors"].values()) == first["total"],
        "constructor counts do not reconcile",
        failures,
    )
    require(not first["diagnostics"], f"source diagnostics: {first['diagnostics']}", failures)
    require(
        first["counts"].get(UNCLASSIFIED, 0) == 0,
        f"current source has {first['counts'].get(UNCLASSIFIED, 0)} unclassified rows",
        failures,
    )

    renamed = fixture_report("renamed_collection.rs")
    require(renamed["total"] == 1, "renamed fixture row count", failures)
    require(
        renamed["sites"][0]["classification"] == "BORROWED",
        "renamed fixture did not follow borrowed parameter",
        failures,
    )
    nested = fixture_report("nested_commas.rs")
    require(nested["total"] == 3, "nested-comma fixture did not find three calls", failures)
    suffix = fixture_report("exact_suffix.rs")
    require(suffix["total"] == 3, "exact suffix fixture matched a prefix or missed siblings", failures)
    comments = fixture_report("comments_and_strings.rs")
    require(comments["total"] == 0, "comments or strings produced sites", failures)
    opaque = fixture_report("opaque_helper.rs")
    require(
        opaque["counts"].get(UNCLASSIFIED) == 1,
        "opaque helper did not fail closed",
        failures,
    )
    truncated = fixture_report("truncated_input.rs")
    require(bool(truncated["diagnostics"]), "truncated input had no diagnostic", failures)

    # An unseen fixture is generated outside the repository and must be parsed;
    # no current-tree total participates in this assertion.
    with tempfile.TemporaryDirectory(prefix="mamba-ownership-") as directory:
        unseen = Path(directory) / "unseen.rs"
        unseen.write_text(
            "fn unseen(x: MbValue) { MbObject::new_set_borrowed(vec![x]); }\n",
            encoding="utf-8",
        )
        dynamic = inventory(unseen)
        require(dynamic["total"] == 1, "unseen fixture was not recomputed", failures)
        require(
            dynamic["sites"][0]["classification"] == "BORROWED",
            "unseen fixture classification",
            failures,
        )
        direct_source = (
            "fn renamed(item: MbValue) { MbObject::new_list(vec![item]); }\n"
        )
        renamed_source = (
            "fn renamed(item: MbValue) { "
            "let original = vec![item]; let different_name = original; "
            "MbObject::new_list(different_name); }\n"
        )
        unseen.write_text(direct_source, encoding="utf-8")
        direct_origin = inventory(unseen)["sites"][0]["origin_fingerprint"]
        unseen.write_text(renamed_source, encoding="utf-8")
        renamed_origin = inventory(unseen)["sites"][0]["origin_fingerprint"]
        require(
            direct_origin == renamed_origin,
            "local rename changed the semantic origin fingerprint",
            failures,
        )

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        return 1
    print(
        "PASS "
        f"total={first['total']} digest={first['inventory_digest']} "
        f"counts={json.dumps(first['counts'], sort_keys=True)}"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    parser.add_argument("--fixture", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        return run_checks()
    report = inventory(args.fixture or args.root)
    sys.stdout.buffer.write(encoded(report))
    if report["diagnostics"] or report["counts"].get(UNCLASSIFIED, 0):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
