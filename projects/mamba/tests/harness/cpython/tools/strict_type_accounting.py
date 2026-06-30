#!/usr/bin/env python3.12
"""Strict-type replacement accounting for #704.

This is the machine-readable bridge between the typeshed-derived type wall,
the executable type fixtures, and declared behavior-denominator exclusions.
It is intentionally strict: sampled runs are useful development evidence, but
only a full run can go green.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import sys
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import harness_lib


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]
REPO_ROOT = MAMBA_DIR.parents[1]
FIXTURES_DIR = MAMBA_DIR / "tests" / "cpython"
TYPE_DIR = FIXTURES_DIR / "type"
SOUND_DIR = FIXTURES_DIR / "behavior" / "core"
GENERATED_SIGS = MAMBA_DIR / "src" / "types" / "stdlib_sigs_generated.rs"
TYPESHED_STDLIB = MAMBA_DIR / "vendor" / "typeshed" / "stdlib"
TYPE_DIVERGENCES = TOOLS_DIR.parent / "config" / "type_divergences.txt"

EXIT_NOT_READY = 70
NON_RUNTIME_STUB_TYPE_LIB_PREFIXES = ("_typeshed",)
PLATFORM_SPECIFIC_TYPE_LIBS = {"_winapi": "win32"}
VERSION_SPECIFIC_TYPE_LIBS = {
    "_zstd": (3, 14),
    "annotationlib": (3, 14),
    "asyncio_graph": (3, 14),
    "asyncio_tools": (3, 14),
    "compression_zstd": (3, 14),
    "compression_zstd__zstdfile": (3, 14),
    "concurrent_interpreters": (3, 14),
    "concurrent_interpreters__crossinterp": (3, 14),
    "concurrent_interpreters__queues": (3, 14),
}
VERSION_REMOVED_TYPE_LIBS = {
    "asyncore": (3, 12),
    "asynchat": (3, 12),
    "smtpd": (3, 12),
}
VERSION_SPECIFIC_TYPE_FIXTURES = {
    "std-libs/ast/TemplateStr__init__values_as_list_wrong.py": (3, 14),
    "std-libs/base64/z85decode__s_as_typed_wrong.py": (3, 13),
    "std-libs/base64/z85encode__s_as_ReadableBuffer_wrong.py": (3, 13),
}
VERSION_REMOVED_TYPE_FIXTURES = {
    "std-libs/asyncio_coroutines/coroutine__func_as__FunctionT_wrong.py": (3, 12),
}

SOUND_FAMILIES = [
    "float_return_inference",
    "builtin_numeric_inference",
    "comprehension_float_inference",
    "generator_float_inference",
    "container_float_roundtrip",
    "value_equality_inference",
    "mixed_numeric_inference",
]

BEHAVIOR_FACETS = {"behavior", "surface", "errors", "real_world"}

TYPE_REJECTION_MARKERS = (
    "type mismatch",
    "type error",
    "requires numeric type",
    "requires numeric types",
    "require int types",
    "TypeError:",
)
NON_TYPE_REJECTION_MARKERS = (
    "undefined name",
    "unknown type",
    "unknown generic type",
)


@dataclass
class Divergence:
    path: str
    owner_refs: list[str]


def default_mamba_bin() -> str:
    if env := os.environ.get("MAMBA_BIN"):
        return env
    debug = (MAMBA_DIR / "../../target/debug/mamba").resolve()
    if debug.exists():
        return str(debug)
    release = (MAMBA_DIR / "../../target/release/mamba").resolve()
    if release.exists():
        return str(release)
    return "mamba"


def selected(paths: list[Path], limit: int) -> tuple[list[Path], bool]:
    if limit <= 0 or len(paths) <= limit:
        return paths, False
    step = max(1, len(paths) // limit)
    return paths[::step][:limit], True


def repo_rel(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def is_non_runtime_stub_type_fixture(path: Path) -> bool:
    lib = type_fixture_lib(path)
    if lib is None:
        return False
    return any(
        lib == prefix or lib.startswith(f"{prefix}_")
        for prefix in NON_RUNTIME_STUB_TYPE_LIB_PREFIXES
    )


def type_fixture_lib(path: Path) -> str | None:
    try:
        rel = path.relative_to(TYPE_DIR).parts
    except ValueError:
        return None
    if len(rel) < 3 or rel[0] != "std-libs":
        return None
    return rel[1]


def is_platform_specific_unavailable_type_fixture(path: Path) -> bool:
    lib = type_fixture_lib(path)
    if lib is None:
        return False
    required = PLATFORM_SPECIFIC_TYPE_LIBS.get(lib)
    return required is not None and sys.platform != required


def is_version_specific_unavailable_type_fixture(path: Path) -> bool:
    try:
        rel = "/".join(path.relative_to(TYPE_DIR).parts)
    except ValueError:
        rel = ""
    required = VERSION_SPECIFIC_TYPE_FIXTURES.get(rel)
    if required is not None:
        return sys.version_info[:2] < required
    removed = VERSION_REMOVED_TYPE_FIXTURES.get(rel)
    if removed is not None:
        return sys.version_info[:2] >= removed
    lib = type_fixture_lib(path)
    if lib is None:
        return False
    required = VERSION_SPECIFIC_TYPE_LIBS.get(lib)
    if required is not None and sys.version_info[:2] < required:
        return True
    removed = VERSION_REMOVED_TYPE_LIBS.get(lib)
    return removed is not None and sys.version_info[:2] >= removed


def is_excluded_type_fixture(path: Path) -> bool:
    return (
        is_non_runtime_stub_type_fixture(path)
        or is_platform_specific_unavailable_type_fixture(path)
        or is_version_specific_unavailable_type_fixture(path)
    )


def executable_type_fixtures(paths: list[Path]) -> list[Path]:
    return [path for path in paths if not is_excluded_type_fixture(path)]


def run_mamba(mamba_bin: str, fixture: Path, timeout: int) -> tuple[int | None, str, str]:
    inner = (
        f"ulimit -t {timeout} 2>/dev/null; "
        f"ulimit -c 0 2>/dev/null; "
        f"exec {shlex.quote(mamba_bin)} run {shlex.quote(str(fixture))}"
    )
    return harness_lib.run_fixture(["/bin/sh", "-c", inner], timeout + 5)


def is_type_rejection(stdout: str, stderr: str) -> bool:
    blob = f"{stderr}\n{stdout}"
    if any(marker in blob for marker in NON_TYPE_REJECTION_MARKERS):
        return False
    return any(marker in blob for marker in TYPE_REJECTION_MARKERS)


def parse_generated_signature_counts() -> dict[str, Any]:
    text = GENERATED_SIGS.read_text(encoding="utf-8", errors="replace")
    header = re.search(
        r"rows:\s*(?P<rows>\d+)\s*.*?enforceable \(scalar\):\s*"
        r"(?P<enforceable>\d+)\s*.*?unknown-skipped:\s*(?P<unknown>\d+)",
        text,
        re.S,
    )
    if header:
        return {
            "source": "generated_table_header",
            "rows": int(header.group("rows")),
            "enforceable": int(header.group("enforceable")),
            "unknown_skipped": int(header.group("unknown")),
            "vendor_typeshed_available": TYPESHED_STDLIB.exists(),
        }
    rows = len(re.findall(r"\bStdlibSig\s*\{", text))
    enforceable = len(re.findall(r"enforceable:\s*true", text))
    return {
        "source": "generated_table_scan",
        "rows": rows,
        "enforceable": enforceable,
        "unknown_skipped": max(0, rows - enforceable),
        "vendor_typeshed_available": TYPESHED_STDLIB.exists(),
    }


def grade_enforcement(mamba_bin: str, fixture: Path, timeout: int) -> tuple[str, str, str]:
    rc, out, err = run_mamba(mamba_bin, fixture, timeout)
    rel = fixture.relative_to(TYPE_DIR).parts
    bucket = rel[0] if len(rel) > 1 else "core"
    if rc is None:
        return bucket, repo_rel(fixture), "ungradable"
    if rc != 0:
        return (
            bucket,
            repo_rel(fixture),
            "enforced" if is_type_rejection(out, err) else "ungradable",
        )
    if "no_typeerror:" in out:
        return bucket, repo_rel(fixture), "leaked"
    if "typeerror:" in out:
        return bucket, repo_rel(fixture), "enforced"
    return bucket, repo_rel(fixture), "ungradable"


def grade_soundness(mamba_bin: str, fixture: Path, timeout: int) -> tuple[str, str]:
    orc, oout, _ = harness_lib.run_fixture(["python3.12", str(fixture)], timeout)
    rel = repo_rel(fixture)
    if orc != 0:
        return rel, "oracle_skip"
    mrc, mout, _ = run_mamba(mamba_bin, fixture, timeout)
    if mrc == 0 and mout.strip() == oout.strip():
        return rel, "passed"
    return rel, "failed"


def load_divergences() -> list[Divergence]:
    if not TYPE_DIVERGENCES.exists():
        return []
    out: list[Divergence] = []
    current_owner_refs: list[str] = []
    for raw in TYPE_DIVERGENCES.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.startswith("#"):
            if "owner:" in line:
                current_owner_refs = re.findall(r"#\d+", line)
            continue
        out.append(Divergence(path=line, owner_refs=current_owner_refs))
        current_owner_refs = []
    return out


def validate_divergence(
    divergence: Divergence, mamba_bin: str, timeout: int
) -> dict[str, Any]:
    fixture = REPO_ROOT / divergence.path
    problems: list[str] = []
    if not divergence.owner_refs:
        problems.append("missing owner")
    if not fixture.exists():
        problems.append("fixture missing")
        return {
            "path": divergence.path,
            "owner_refs": divergence.owner_refs,
            "valid": False,
            "problems": problems,
        }
    try:
        facet = fixture.relative_to(FIXTURES_DIR).parts[0]
    except ValueError:
        facet = ""
    if facet not in BEHAVIOR_FACETS:
        problems.append(f"not a behavior-denominator facet: {facet or '<outside>'}")
    orc, _oout, oerr = harness_lib.run_fixture(["python3.12", str(fixture)], timeout)
    if orc != 0:
        problems.append(f"cpython oracle failed: rc={orc} stderr={oerr[:160]}")
    mrc, mout, merr = run_mamba(mamba_bin, fixture, timeout)
    if mrc is None:
        problems.append("mamba timed out or could not run")
    elif mrc == 0:
        problems.append("mamba did not reject")
    elif not is_type_rejection(mout, merr):
        problems.append("mamba rejected, but not with a verified type rejection")
    return {
        "path": divergence.path,
        "owner_refs": divergence.owner_refs,
        "valid": not problems,
        "problems": problems,
    }


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    mamba_bin = args.mamba_bin or default_mamba_bin()
    type_fixture_candidates = sorted(TYPE_DIR.rglob("*.py")) if TYPE_DIR.exists() else []
    excluded_non_runtime_stubs = [
        path for path in type_fixture_candidates if is_non_runtime_stub_type_fixture(path)
    ]
    excluded_platform_specific = [
        path
        for path in type_fixture_candidates
        if is_platform_specific_unavailable_type_fixture(path)
    ]
    excluded_version_specific = [
        path
        for path in type_fixture_candidates
        if is_version_specific_unavailable_type_fixture(path)
    ]
    type_fixtures_all = executable_type_fixtures(type_fixture_candidates)
    type_fixtures, enforcement_sampled = selected(type_fixtures_all, args.limit)
    sound_fixtures_all = sorted(
        path for family in SOUND_FAMILIES for path in (SOUND_DIR / family).glob("*.py")
    )
    sound_fixtures, sound_sampled = selected(sound_fixtures_all, args.limit)

    by_bucket: dict[str, Counter] = {}
    enforcement_blockers: list[dict[str, str]] = []
    with ThreadPoolExecutor(max_workers=max(1, args.jobs)) as executor:
        for bucket, rel, verdict in executor.map(
            lambda path: grade_enforcement(mamba_bin, path, args.timeout),
            type_fixtures,
        ):
            by_bucket.setdefault(bucket, Counter())[verdict] += 1
            if verdict != "enforced" and len(enforcement_blockers) < args.show:
                enforcement_blockers.append({"path": rel, "verdict": verdict})

    enforcement_counts = Counter()
    for counter in by_bucket.values():
        enforcement_counts.update(counter)
    enforcement_gradable = enforcement_counts["enforced"] + enforcement_counts["leaked"]
    enforcement_rate = (
        100.0 * enforcement_counts["enforced"] / enforcement_gradable
        if enforcement_gradable
        else 0.0
    )

    sound_counts: Counter = Counter()
    sound_blockers: list[dict[str, str]] = []
    with ThreadPoolExecutor(max_workers=max(1, args.jobs)) as executor:
        for rel, verdict in executor.map(
            lambda path: grade_soundness(mamba_bin, path, args.timeout), sound_fixtures
        ):
            sound_counts[verdict] += 1
            if verdict != "passed" and len(sound_blockers) < args.show:
                sound_blockers.append({"path": rel, "verdict": verdict})
    sound_gradable = sound_counts["passed"] + sound_counts["failed"]
    sound_rate = (
        100.0 * sound_counts["passed"] / sound_gradable if sound_gradable else 0.0
    )

    divergence_entries = [
        validate_divergence(item, mamba_bin, args.timeout) for item in load_divergences()
    ]
    invalid_divergences = [item for item in divergence_entries if not item["valid"]]
    missing_owner = [item for item in divergence_entries if not item["owner_refs"]]

    typeshed = parse_generated_signature_counts()
    sampled = enforcement_sampled or sound_sampled
    ready = (
        not sampled
        and typeshed["enforceable"] > 0
        and enforcement_counts["leaked"] == 0
        and enforcement_counts["ungradable"] == 0
        and enforcement_counts["enforced"] == len(type_fixtures_all)
        and sound_counts["failed"] == 0
        and sound_counts["oracle_skip"] == 0
        and sound_counts["passed"] == len(sound_fixtures_all)
        and not invalid_divergences
        and not missing_owner
    )

    blockers: list[dict[str, Any]] = []
    if sampled:
        blockers.append(
            {
                "kind": "sampled_runtime_accounting",
                "reason": "sampled runs are development evidence, not replacement proof",
            }
        )
    if enforcement_counts["leaked"] or enforcement_counts["ungradable"]:
        blockers.extend(
            {"kind": "type_enforcement", **item} for item in enforcement_blockers
        )
    if sound_counts["failed"] or sound_counts["oracle_skip"]:
        blockers.extend({"kind": "type_soundness", **item} for item in sound_blockers)
    blockers.extend(
        {
            "kind": "invalid_type_divergence",
            "path": item["path"],
            "problems": item["problems"],
        }
        for item in invalid_divergences[: args.show]
    )

    return {
        "schema_version": 1,
        "profile": "strict-type-accounting",
        "status": "green" if ready else "red",
        "ready": ready,
        "sampled": sampled,
        "mamba_bin": mamba_bin,
        "typeshed": {
            **typeshed,
            "type_fixture_wall": len(type_fixtures_all),
            "measured_type_fixtures": len(type_fixtures),
            "excluded_non_runtime_stub_fixtures": len(excluded_non_runtime_stubs),
            "excluded_non_runtime_stub_lib_prefixes": list(NON_RUNTIME_STUB_TYPE_LIB_PREFIXES),
            "excluded_platform_specific_type_fixtures": len(excluded_platform_specific),
            "platform_specific_type_libs": PLATFORM_SPECIFIC_TYPE_LIBS,
            "host_platform": sys.platform,
            "excluded_version_specific_type_fixtures": len(excluded_version_specific),
            "version_specific_type_libs": VERSION_SPECIFIC_TYPE_LIBS,
            "version_removed_type_libs": VERSION_REMOVED_TYPE_LIBS,
            "version_specific_type_fixture_cases": VERSION_SPECIFIC_TYPE_FIXTURES,
            "version_removed_type_fixture_cases": VERSION_REMOVED_TYPE_FIXTURES,
            "host_python_version": list(sys.version_info[:2]),
        },
        "enforcement": {
            "fixtures": len(type_fixtures_all),
            "measured": len(type_fixtures),
            "sampled": enforcement_sampled,
            "gradable": enforcement_gradable,
            "enforced": enforcement_counts["enforced"],
            "leaked": enforcement_counts["leaked"],
            "ungradable": enforcement_counts["ungradable"],
            "rate": round(enforcement_rate, 3),
            "by_bucket": {
                bucket: dict(sorted(counter.items())) for bucket, counter in by_bucket.items()
            },
            "blockers": enforcement_blockers,
        },
        "soundness": {
            "fixtures": len(sound_fixtures_all),
            "measured": len(sound_fixtures),
            "sampled": sound_sampled,
            "gradable": sound_gradable,
            "passed": sound_counts["passed"],
            "failed": sound_counts["failed"],
            "oracle_skip": sound_counts["oracle_skip"],
            "rate": round(sound_rate, 3),
            "blockers": sound_blockers,
        },
        "divergences": {
            "declared": len(divergence_entries),
            "valid": len(divergence_entries) - len(invalid_divergences),
            "invalid": len(invalid_divergences),
            "missing_owner": len(missing_owner),
            "entries": divergence_entries[: args.show],
        },
        "blockers": blockers[: args.show],
    }


def print_human(report: dict[str, Any]) -> None:
    print(f"strict-type accounting: {report['status']}")
    print(f"  mamba: {report['mamba_bin']}")
    print(f"  sampled: {report['sampled']}")
    typeshed = report["typeshed"]
    print(
        "  typeshed: "
        f"rows={typeshed['rows']} enforceable={typeshed['enforceable']} "
        f"unknown_skipped={typeshed['unknown_skipped']} "
        f"fixtures={typeshed['type_fixture_wall']}"
    )
    enforcement = report["enforcement"]
    print(
        "  enforcement: "
        f"measured={enforcement['measured']} gradable={enforcement['gradable']} "
        f"enforced={enforcement['enforced']} leaked={enforcement['leaked']} "
        f"ungradable={enforcement['ungradable']} rate={enforcement['rate']:.1f}%"
    )
    soundness = report["soundness"]
    print(
        "  soundness: "
        f"measured={soundness['measured']} gradable={soundness['gradable']} "
        f"passed={soundness['passed']} failed={soundness['failed']} "
        f"oracle_skip={soundness['oracle_skip']} rate={soundness['rate']:.1f}%"
    )
    divergences = report["divergences"]
    print(
        "  divergences: "
        f"declared={divergences['declared']} valid={divergences['valid']} "
        f"invalid={divergences['invalid']} missing_owner={divergences['missing_owner']}"
    )
    for blocker in report["blockers"]:
        label = blocker.get("path") or blocker.get("kind")
        reason = blocker.get("verdict") or blocker.get("reason") or blocker.get("problems")
        print(f"  blocker: {label} - {reason}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--show", type=int, default=10)
    parser.add_argument("--jobs", type=int, default=max(1, min(8, os.cpu_count() or 1)))
    parser.add_argument("--limit", type=int, default=0, help="sample N type/soundness fixtures")
    parser.add_argument("--timeout", type=int, default=10)
    parser.add_argument("--mamba-bin")
    args = parser.parse_args(argv)

    report = build_report(args)
    if args.json:
        print(json.dumps(report, sort_keys=True))
    else:
        print_human(report)
    return 0 if report["ready"] else EXIT_NOT_READY


if __name__ == "__main__":
    raise SystemExit(main())
