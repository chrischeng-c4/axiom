#!/usr/bin/env python3.12
"""Platform/OS/process/network/TLS readiness accounting for #710.

This is a replacement-readiness gate, not a fixture runner. It keeps the
platform-sensitive surface visible as its own dimension so socket, subprocess,
TLS, signal, selector, filesystem, and host-OS gaps cannot be hidden inside the
general promotion wall.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sqlite3
import sys
import tomllib
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any


TOOLS_DIR = Path(__file__).resolve().parent
CPYTHON_DIR = TOOLS_DIR.parents[2] / "cpython"
MAMBA_DIR = CPYTHON_DIR.parent.parent
RESULTS_DB = CPYTHON_DIR / ".cache" / "conformance" / "results.sqlite"
PERF_PINS_DIR = TOOLS_DIR.parent / "config" / "perf" / "pins"

EXIT_NOT_READY = 70
EXIT_PARSE_ERROR = 71

TARGET_SCOPES: dict[str, tuple[str, ...]] = {
    "filesystem_environment": (
        "glob",
        "os",
        "pathlib",
        "platform",
        "posixpath",
        "shutil",
        "tempfile",
    ),
    "process_signal": (
        "multiprocessing",
        "signal",
        "subprocess",
        "threading",
    ),
    "network_io": (
        "http_client",
        "http_server",
        "selectors",
        "socket",
        "urllib_error",
        "urllib_request",
    ),
    "tls": ("ssl",),
}

TARGET_LIB_TO_SCOPE = {
    lib: scope for scope, libs in TARGET_SCOPES.items() for lib in libs
}

DIMENSION_NAMES = {
    "surface",
    "behavior",
    "errors",
    "real_world",
    "security",
    "type",
    "perf",
    "concurrency",
    "security-matrix",
}

ISSUE_REF_RE = re.compile(
    r"(?:\bWI\s*)?#\d+\b|\b(?:issue|tracker|GH)[-: ]#?\d+\b",
    re.IGNORECASE,
)
PROMOTION_PENDING_RE = re.compile(
    r"\bpromotion pending\b|auto-(?:ported|extracted)\s+CPython\s+test|CPython 3\.12 seed",
    re.IGNORECASE,
)
SANDBOX_DENIED_RE = re.compile(
    r"\b(sandbox|permission denied|operation not permitted|not permitted|"
    r"requires root|privilege|network disabled|network is unreachable|"
    r"address already in use|resource temporarily unavailable)\b",
    re.IGNORECASE,
)
UNSUPPORTED_PLATFORM_RE = re.compile(
    r"(unsupported|not available|win32|windows|linux|darwin|macos|freebsd|"
    r"android|ios|aix|solaris|hyper[_-]?v|bluetooth|af_bluetooth|af_can|"
    r"can[_-]?socket|rds|vsock|tipc|kqueue|epoll|devpoll|wasm)",
    re.IGNORECASE,
)
SKIP_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("mamba-skip-directive", re.compile(r"^\s*#\s*mamba-skip\s*:", re.MULTILINE)),
    ("pytest-skip-marker", re.compile(r"^\s*@pytest\.mark\.skip(?:if)?\b", re.MULTILINE)),
    ("pytest-skip-call", re.compile(r"\bpytest\.skip\s*\(")),
    ("unittest-skip-marker", re.compile(r"^\s*@(?:unittest\.)?skip(?:If|Unless)?\b", re.MULTILINE)),
    ("unittest-skip-call", re.compile(r"\b(?:self\.)?skipTest\s*\(")),
    ("unittest-skip-exception", re.compile(r"\bSkipTest\b")),
)


@dataclass(frozen=True)
class Fixture:
    path: Path
    rel: str
    scope: str
    lib: str
    dimension: str
    case: str
    xfail: str
    skip_reason: str
    parse_error: str = ""


def repo_rel(path: Path) -> str:
    return path.resolve().relative_to(MAMBA_DIR.resolve()).as_posix()


def issue_refs(text: str) -> list[str]:
    return sorted(set(match.group(0).replace("WI", "").strip() for match in ISSUE_REF_RE.finditer(text)))


def extract_script_toml(text: str) -> str | None:
    start = text.find("# /// script")
    if start < 0:
        return None
    rest = text[start + len("# /// script") :]
    end = rest.find("\n# ///")
    if end < 0:
        return None
    lines: list[str] = []
    for raw in rest[:end].splitlines():
        if raw.startswith("# "):
            lines.append(raw[2:])
        elif raw in {"#", ""}:
            lines.append("")
        else:
            return None
    return "\n".join(lines)


def tool_mamba(text: str) -> tuple[dict[str, Any], str]:
    block = extract_script_toml(text)
    if block is None:
        return {}, "missing PEP 723 script block"
    try:
        parsed = tomllib.loads(block)
    except tomllib.TOMLDecodeError as exc:
        return {}, f"TOML parse failed: {exc}"
    meta = parsed.get("tool", {}).get("mamba")
    if not isinstance(meta, dict):
        return {}, "missing [tool.mamba] metadata"
    return meta, ""


def detect_target(path: Path) -> tuple[str, str] | None:
    rel_parts = path.relative_to(CPYTHON_DIR).parts
    for index, part in enumerate(rel_parts[:-1]):
        if part in {"std-libs", "primitives"}:
            lib = rel_parts[index + 1]
            scope = TARGET_LIB_TO_SCOPE.get(lib)
            if scope:
                return scope, lib
    return None


def detect_dimension(path: Path, meta: dict[str, Any]) -> str:
    dimension = meta.get("dimension")
    if isinstance(dimension, str) and dimension:
        return dimension
    rel_parts = path.relative_to(CPYTHON_DIR).parts
    for part in rel_parts:
        if part in DIMENSION_NAMES:
            return part
    if "bench" in rel_parts:
        return "bench"
    return rel_parts[0] if rel_parts else "unknown"


def detect_skip_reason(text: str) -> str:
    reasons = [name for name, pattern in SKIP_PATTERNS if pattern.search(text)]
    return ", ".join(reasons)


def fixture_case(path: Path, meta: dict[str, Any]) -> str:
    case = meta.get("case")
    return case if isinstance(case, str) and case else path.stem


def discover_fixtures() -> tuple[list[Fixture], list[dict[str, str]]]:
    fixtures: list[Fixture] = []
    parse_errors: list[dict[str, str]] = []
    for path in sorted(CPYTHON_DIR.rglob("*.py")):
        if path.name.endswith("_stub.py"):
            continue
        rel_parts = path.relative_to(CPYTHON_DIR).parts
        if "_invalid" in rel_parts:
            continue
        if "bench" in rel_parts:
            # Perf fixtures are covered through #707 perf pins. #710 owns the
            # semantic platform surface plus result-store evidence.
            continue
        target = detect_target(path)
        if target is None:
            continue
        scope, lib = target
        text = path.read_text(encoding="utf-8", errors="replace")
        meta, parse_error = tool_mamba(text)
        rel = repo_rel(path)
        if parse_error:
            parse_errors.append({"path": rel, "error": parse_error})
        xfail = meta.get("xfail", "")
        fixtures.append(
            Fixture(
                path=path,
                rel=rel,
                scope=scope,
                lib=lib,
                dimension=detect_dimension(path, meta),
                case=fixture_case(path, meta),
                xfail=xfail if isinstance(xfail, str) else "",
                skip_reason=detect_skip_reason(text),
                parse_error=parse_error,
            )
        )
    return fixtures, parse_errors


def classify_metadata(fixture: Fixture) -> tuple[str, str, list[str]]:
    reason = fixture.skip_reason or fixture.xfail
    haystack = "\n".join([fixture.rel, fixture.case, reason])
    owners = issue_refs(reason)
    if fixture.parse_error:
        return "metadata_error", fixture.parse_error, owners
    if not reason:
        return "pass_candidate", "", owners
    if SANDBOX_DENIED_RE.search(haystack):
        return "sandbox_denied", reason, owners
    if UNSUPPORTED_PLATFORM_RE.search(haystack):
        return "unsupported_platform", reason, owners
    if PROMOTION_PENDING_RE.search(reason):
        return "promotion_pending", reason, owners
    return "runtime_failure_debt", reason, owners


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_mamba_rows(fixtures: list[Fixture], db: Path) -> dict[str, dict[str, Any]]:
    if not db.exists() or not fixtures:
        return {}
    conn = sqlite3.connect(db)
    conn.row_factory = sqlite3.Row
    rows: dict[str, dict[str, Any]] = {}
    try:
        for fixture in fixtures:
            content_hash = sha256_file(fixture.path)
            row = conn.execute(
                "SELECT fixture_id, verdict, exit_code, signal, timed_out, recorded_at "
                "FROM results WHERE runtime='mamba' AND fixture_id=? AND content_hash=? "
                "ORDER BY recorded_at DESC LIMIT 1",
                (fixture.rel, content_hash),
            ).fetchone()
            if row is not None:
                rows[fixture.rel] = dict(row)
    finally:
        conn.close()
    return rows


def runtime_state(row: dict[str, Any] | None) -> str:
    if row is None:
        return "unmeasured"
    verdict = str(row.get("verdict") or "")
    if verdict == "MAMBA_OK":
        return "runtime_ok"
    if verdict == "MAMBA_TIMEOUT":
        return "runtime_timeout"
    if verdict.startswith("CRASH_SIG_"):
        return "runtime_crash"
    return "runtime_fail"


def load_perf_pin_counts() -> dict[str, Any]:
    counts = {
        "total": 0,
        "by_scope": {scope: 0 for scope in TARGET_SCOPES},
        "by_lib": {lib: 0 for lib in TARGET_LIB_TO_SCOPE},
        "malformed": [],
    }
    if not PERF_PINS_DIR.exists():
        counts["missing_dir"] = str(repo_rel(PERF_PINS_DIR))
        return counts
    for path in sorted(PERF_PINS_DIR.glob("*.toml")):
        rel = repo_rel(path)
        try:
            parsed = tomllib.loads(path.read_text(encoding="utf-8"))
        except tomllib.TOMLDecodeError as exc:
            counts["malformed"].append({"path": rel, "error": str(exc)})
            continue
        lib = parsed.get("lib")
        if not isinstance(lib, str):
            counts["malformed"].append({"path": rel, "error": "missing lib"})
            continue
        scope = TARGET_LIB_TO_SCOPE.get(lib)
        if scope is None:
            continue
        counts["total"] += 1
        counts["by_scope"][scope] += 1
        counts["by_lib"][lib] += 1
    return counts


def build_report(show: int, db: Path) -> dict[str, Any]:
    fixtures, parse_errors = discover_fixtures()
    rows = load_mamba_rows(fixtures, db)
    by_scope: dict[str, Counter[str]] = defaultdict(Counter)
    by_lib: dict[str, Counter[str]] = defaultdict(Counter)
    by_dimension: dict[str, Counter[str]] = defaultdict(Counter)
    metadata_counts: Counter[str] = Counter()
    runtime_counts: Counter[str] = Counter()
    gaps: list[dict[str, Any]] = []

    for fixture in fixtures:
        metadata_state, reason, owners = classify_metadata(fixture)
        runtime = runtime_state(rows.get(fixture.rel))
        metadata_counts[metadata_state] += 1
        runtime_counts[runtime] += 1
        by_scope[fixture.scope][metadata_state] += 1
        by_scope[fixture.scope][runtime] += 1
        by_scope[fixture.scope]["fixtures"] += 1
        by_lib[fixture.lib][metadata_state] += 1
        by_lib[fixture.lib][runtime] += 1
        by_lib[fixture.lib]["fixtures"] += 1
        by_dimension[fixture.dimension][metadata_state] += 1
        by_dimension[fixture.dimension][runtime] += 1
        by_dimension[fixture.dimension]["fixtures"] += 1

        if metadata_state != "pass_candidate":
            gaps.append(
                {
                    "kind": metadata_state,
                    "path": fixture.rel,
                    "scope": fixture.scope,
                    "lib": fixture.lib,
                    "dimension": fixture.dimension,
                    "case": fixture.case,
                    "reason": reason,
                    "owner_refs": owners,
                    "owned": bool(owners),
                }
            )
        if runtime != "runtime_ok":
            gaps.append(
                {
                    "kind": runtime,
                    "path": fixture.rel,
                    "scope": fixture.scope,
                    "lib": fixture.lib,
                    "dimension": fixture.dimension,
                    "case": fixture.case,
                    "reason": "no current mamba results-store row for this fixture"
                    if runtime == "unmeasured"
                    else runtime,
                    "owner_refs": [],
                    "owned": False,
                }
            )

    missing_libs = [
        {"kind": "missing_target_lib", "scope": scope, "lib": lib}
        for scope, libs in TARGET_SCOPES.items()
        for lib in libs
        if by_lib[lib]["fixtures"] == 0
    ]
    perf_pins = load_perf_pin_counts()
    unowned_gap_count = sum(1 for gap in gaps if not gap.get("owned"))
    runtime_problem_count = sum(
        runtime_counts[state]
        for state in ("unmeasured", "runtime_fail", "runtime_timeout", "runtime_crash")
    )
    metadata_problem_count = sum(
        metadata_counts[state]
        for state in (
            "metadata_error",
            "promotion_pending",
            "runtime_failure_debt",
            "sandbox_denied",
            "unsupported_platform",
        )
    )
    ready = (
        not missing_libs
        and not parse_errors
        and metadata_problem_count == 0
        and runtime_problem_count == 0
        and unowned_gap_count == 0
        and not perf_pins.get("malformed")
    )

    blockers = [
        *missing_libs,
        *({"kind": "metadata_parse_error", **item} for item in parse_errors[:show]),
        *[gap for gap in gaps if not gap.get("owned")][:show],
    ]
    if perf_pins.get("malformed"):
        blockers.extend(
            {"kind": "malformed_perf_pin", **item}
            for item in perf_pins["malformed"][:show]
        )

    return {
        "schema_version": 1,
        "owner_issue": "#710",
        "ready": ready,
        "status": "green" if ready else "red",
        "host": {
            "sys_platform": sys.platform,
            "platform": sys.implementation.name,
            "python": sys.version.split()[0],
        },
        "target_scopes": TARGET_SCOPES,
        "results_db": repo_rel(db) if db.is_absolute() else str(db),
        "counts": {
            "fixtures": len(fixtures),
            "target_libs": len(TARGET_LIB_TO_SCOPE),
            "missing_target_libs": len(missing_libs),
            "parse_errors": len(parse_errors),
            "metadata_problem_count": metadata_problem_count,
            "runtime_problem_count": runtime_problem_count,
            "unowned_gap_count": unowned_gap_count,
            "pass_candidate": metadata_counts["pass_candidate"],
            "promotion_pending": metadata_counts["promotion_pending"],
            "runtime_failure_debt": metadata_counts["runtime_failure_debt"],
            "sandbox_denied": metadata_counts["sandbox_denied"],
            "unsupported_platform": metadata_counts["unsupported_platform"],
            "metadata_error": metadata_counts["metadata_error"],
            "runtime_ok": runtime_counts["runtime_ok"],
            "runtime_fail": runtime_counts["runtime_fail"],
            "runtime_timeout": runtime_counts["runtime_timeout"],
            "runtime_crash": runtime_counts["runtime_crash"],
            "unmeasured": runtime_counts["unmeasured"],
            "perf_pins": perf_pins["total"],
            "malformed_perf_pins": len(perf_pins.get("malformed", [])),
        },
        "by_scope": {
            key: dict(sorted(value.items())) for key, value in sorted(by_scope.items())
        },
        "by_lib": {
            key: dict(sorted(value.items())) for key, value in sorted(by_lib.items())
        },
        "by_dimension": {
            key: dict(sorted(value.items()))
            for key, value in sorted(by_dimension.items())
        },
        "perf_pins": perf_pins,
        "blocker_count": len(missing_libs)
        + len(parse_errors)
        + unowned_gap_count
        + len(perf_pins.get("malformed", [])),
        "blockers": blockers,
        "evidence_commands": [
            "python3.12 projects/mamba/tests/harness/cpython/tools/platform_readiness.py --json",
            "python3.12 projects/mamba/tests/harness/cpython/tools/results_store.py collect --bucket std-libs --dimension behavior --limit 20",
            "python3.12 projects/mamba/tests/harness/cpython/tools/verify_cpython_oracle.py --ready-only --bucket behavior",
            "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/socket/inheritable_flag_toggles_and_dup.py",
        ],
    }


def print_human(report: dict[str, Any]) -> None:
    print(f"platform readiness: {report['status']}")
    counts = report["counts"]
    print(
        "  fixtures={fixtures} pass_candidate={pass_candidate} "
        "unmeasured={unmeasured} promotion_pending={promotion_pending} "
        "runtime_failure_debt={runtime_failure_debt} sandbox_denied={sandbox_denied} "
        "unsupported_platform={unsupported_platform}".format(**counts)
    )
    print(
        "  runtime: ok={runtime_ok} fail={runtime_fail} timeout={runtime_timeout} "
        "crash={runtime_crash}".format(**counts)
    )
    for scope, scope_counts in report["by_scope"].items():
        print(f"- {scope}: {scope_counts}")
    for blocker in report["blockers"][:5]:
        label = blocker.get("path") or f"{blocker.get('scope')}/{blocker.get('lib')}"
        print(f"  blocker: {blocker['kind']} {label}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--show", type=int, default=10)
    parser.add_argument("--db", type=Path, default=RESULTS_DB)
    args = parser.parse_args(argv)

    report = build_report(args.show, args.db)
    if args.json:
        print(json.dumps(report, sort_keys=True))
    else:
        print_human(report)
    if report["counts"]["parse_errors"]:
        return EXIT_PARSE_ERROR
    return 0 if report["ready"] else EXIT_NOT_READY


if __name__ == "__main__":
    raise SystemExit(main())
