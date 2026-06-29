#!/usr/bin/env python3.12
"""Concurrency and free-threaded readiness accounting for #713.

This report tracks the Python-visible concurrency denominator separately from
the general platform and import surfaces. It intentionally reports missing
free-threaded/no-GIL contract evidence as a readiness blocker instead of
assuming CPython's GIL behavior or mamba's future runtime model.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from platform_readiness import (
    CPYTHON_DIR,
    EXIT_NOT_READY,
    Fixture,
    RESULTS_DB,
    classify_metadata,
    detect_dimension,
    detect_skip_reason,
    fixture_case,
    load_mamba_rows,
    repo_rel,
    runtime_state,
    tool_mamba,
)


TOOLS_DIR = Path(__file__).resolve().parent
PERF_PINS_DIR = TOOLS_DIR.parent / "config" / "perf" / "pins"

TARGET_SCOPES: dict[str, tuple[str, ...]] = {
    "free_threaded_atomicity": (
        "list",
        "dict",
        "set",
    ),
    "thread_primitives": (
        "_thread",
        "thread",
        "threading",
        "threading_primitives",
    ),
    "async_context": (
        "asyncio",
        "contextvars",
    ),
    "process_parallelism": (
        "multiprocessing",
        "subprocess",
        "concurrent_futures",
    ),
    "synchronization_queues": (
        "lock",
        "queue",
    ),
    "signal_interaction": (
        "signal",
    ),
}

TARGET_LIB_TO_SCOPE = {
    lib: scope for scope, libs in TARGET_SCOPES.items() for lib in libs
}

REAL_WORLD_PATTERNS: tuple[tuple[str, str, re.Pattern[str]], ...] = (
    ("thread_primitives", "threading_real_world", re.compile(r"\bthreading\b|\b_thread\b")),
    ("async_context", "async_context_real_world", re.compile(r"\basyncio\b|\bcontextvars\b|\bawait\b")),
    (
        "process_parallelism",
        "process_parallelism_real_world",
        re.compile(r"\bmultiprocessing\b|\bsubprocess\b|\bconcurrent_futures\b"),
    ),
    ("synchronization_queues", "queue_real_world", re.compile(r"\bqueue\b|task_done|join\(")),
    ("signal_interaction", "signal_real_world", re.compile(r"\bsignal\b|SIG[A-Z]+|wakeup_fd")),
)

SEMANTIC_CLASS_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("thread_lifecycle", re.compile(r"(thread|threading|_thread|start|join|daemon|ident|native_id)", re.I)),
    ("synchronization_locks", re.compile(r"(lock|rlock|condition|semaphore|event|barrier|mutex|wait)", re.I)),
    ("queue_semantics", re.compile(r"(queue|put|get|task_done|join|empty|full)", re.I)),
    ("executor_future", re.compile(r"(concurrent_futures|future|executor|threadpoolexecutor|processpoolexecutor)", re.I)),
    ("multiprocessing_processes", re.compile(r"(multiprocessing|process|pool|spawn|fork|pipe|manager)", re.I)),
    ("asyncio_tasks_event_loop", re.compile(r"(asyncio|event_loop|coroutine|task|gather|await|timeout)", re.I)),
    ("contextvars_propagation", re.compile(r"(contextvars|contextvar|copy_context|context propagation)", re.I)),
    ("signal_thread_interaction", re.compile(r"(signal|sigint|sigterm|wakeup_fd|handler|setitimer)", re.I)),
    ("subprocess_process_io", re.compile(r"(subprocess|popen|pipe|communicate|returncode|stdin|stdout|stderr)", re.I)),
    ("race_deadlock_determinism", re.compile(r"(race|deadlock|nondetermin|stress|timeout|hang|barrier|wait|no_lost|no_corruption)", re.I)),
    ("free_threaded_no_gil_contract", re.compile(r"(free[_-]?thread|no[_-]?gil|gil|Py_GIL_DISABLED|atomicity|single[_-]?mutation)", re.I)),
)

REQUIRED_SEMANTIC_CLASSES = (
    "thread_lifecycle",
    "synchronization_locks",
    "queue_semantics",
    "executor_future",
    "multiprocessing_processes",
    "asyncio_tasks_event_loop",
    "contextvars_propagation",
    "signal_thread_interaction",
    "subprocess_process_io",
    "race_deadlock_determinism",
    "free_threaded_no_gil_contract",
)


def detect_path_target(path: Path) -> tuple[str, str] | None:
    rel_parts = path.relative_to(CPYTHON_DIR).parts
    if len(rel_parts) >= 3 and rel_parts[0] == "concurrency":
        if rel_parts[1] == "atomicity" and rel_parts[2] in {"list", "dict", "set"}:
            return "free_threaded_atomicity", rel_parts[2]
        if rel_parts[1] == "safety" and rel_parts[2] == "lock":
            return "synchronization_queues", "lock"
        if rel_parts[1] == "primitives" and rel_parts[2] == "threading":
            return "thread_primitives", "threading_primitives"

    for index, part in enumerate(rel_parts[:-1]):
        if part == "std-libs":
            lib = rel_parts[index + 1]
            scope = TARGET_LIB_TO_SCOPE.get(lib)
            if scope:
                return scope, lib
    return None


def detect_real_world_target(path: Path, text: str) -> tuple[str, str] | None:
    rel_parts = path.relative_to(CPYTHON_DIR).parts
    if "real_world" not in rel_parts:
        return None
    if len(rel_parts) >= 3 and rel_parts[0] == "_regression" and rel_parts[1] == "3rd-libs":
        return None
    haystack = "\n".join((repo_rel(path), text[:4000]))
    for scope, lib, pattern in REAL_WORLD_PATTERNS:
        if pattern.search(haystack):
            return scope, lib
    return None


def discover_fixtures() -> tuple[list[Fixture], list[dict[str, str]]]:
    fixtures: list[Fixture] = []
    parse_errors: list[dict[str, str]] = []
    for path in sorted(CPYTHON_DIR.rglob("*.py")):
        rel_parts = path.relative_to(CPYTHON_DIR).parts
        if ".cache" in rel_parts:
            continue
        if path.name.endswith("_stub.py"):
            continue
        if "_invalid" in rel_parts:
            continue
        if "bench" in rel_parts:
            continue

        text = path.read_text(encoding="utf-8", errors="replace")
        target = detect_path_target(path) or detect_real_world_target(path, text)
        if target is None:
            continue
        scope, lib = target
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


def semantic_classes(fixture: Fixture) -> list[str]:
    haystack = "\n".join([fixture.rel, fixture.case, fixture.lib, fixture.scope])
    classes = [
        name for name, pattern in SEMANTIC_CLASS_PATTERNS if pattern.search(haystack)
    ]
    if fixture.dimension == "concurrency" and "free_threaded_no_gil_contract" not in classes:
        classes.append("free_threaded_no_gil_contract")
    if classes:
        return classes
    if fixture.scope == "thread_primitives":
        return ["thread_lifecycle"]
    if fixture.scope == "async_context":
        return ["asyncio_tasks_event_loop"]
    if fixture.scope == "synchronization_queues":
        return ["queue_semantics"]
    return ["concurrency_surface_misc"]


def load_perf_pin_counts() -> dict[str, Any]:
    counts: dict[str, Any] = {
        "total": 0,
        "by_scope": {scope: 0 for scope in TARGET_SCOPES},
        "by_lib": {lib: 0 for lib in TARGET_LIB_TO_SCOPE},
        "malformed": [],
    }
    if not PERF_PINS_DIR.exists():
        counts["missing_dir"] = repo_rel(PERF_PINS_DIR)
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
    by_semantic_class: dict[str, Counter[str]] = defaultdict(Counter)
    metadata_counts: Counter[str] = Counter()
    runtime_counts: Counter[str] = Counter()
    gaps: list[dict[str, Any]] = []

    for fixture in fixtures:
        metadata_state, reason, owners = classify_metadata(fixture)
        runtime = runtime_state(rows.get(fixture.rel))
        classes = semantic_classes(fixture)
        metadata_counts[metadata_state] += 1
        runtime_counts[runtime] += 1
        for counter in (by_scope[fixture.scope], by_lib[fixture.lib], by_dimension[fixture.dimension]):
            counter["fixtures"] += 1
            counter[metadata_state] += 1
            counter[runtime] += 1
        for semantic_class in classes:
            by_semantic_class[semantic_class]["fixtures"] += 1
            by_semantic_class[semantic_class][metadata_state] += 1
            by_semantic_class[semantic_class][runtime] += 1

        if metadata_state != "pass_candidate":
            gaps.append(
                {
                    "kind": metadata_state,
                    "path": fixture.rel,
                    "scope": fixture.scope,
                    "lib": fixture.lib,
                    "dimension": fixture.dimension,
                    "case": fixture.case,
                    "semantic_classes": classes,
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
                    "semantic_classes": classes,
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
    missing_semantic_classes = [
        {"kind": "missing_semantic_class", "semantic_class": name}
        for name in REQUIRED_SEMANTIC_CLASSES
        if by_semantic_class[name]["fixtures"] == 0
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
        and not missing_semantic_classes
        and not parse_errors
        and metadata_problem_count == 0
        and runtime_problem_count == 0
        and unowned_gap_count == 0
        and not perf_pins.get("malformed")
    )
    blockers = [
        *missing_libs,
        *missing_semantic_classes,
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
        "owner_issue": "#713",
        "ready": ready,
        "status": "green" if ready else "red",
        "host": {
            "sys_platform": sys.platform,
            "platform": sys.implementation.name,
            "python": sys.version.split()[0],
        },
        "target_scopes": TARGET_SCOPES,
        "required_semantic_classes": REQUIRED_SEMANTIC_CLASSES,
        "results_db": repo_rel(db) if db.is_absolute() else str(db),
        "counts": {
            "fixtures": len(fixtures),
            "scopes": len(TARGET_SCOPES),
            "target_libs": len(TARGET_LIB_TO_SCOPE),
            "missing_target_libs": len(missing_libs),
            "semantic_classes": len(by_semantic_class),
            "missing_semantic_classes": len(missing_semantic_classes),
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
        "by_semantic_class": {
            key: dict(sorted(value.items()))
            for key, value in sorted(by_semantic_class.items())
        },
        "perf_pins": perf_pins,
        "blocker_count": len(missing_libs)
        + len(missing_semantic_classes)
        + len(parse_errors)
        + unowned_gap_count
        + len(perf_pins.get("malformed", [])),
        "blockers": blockers,
        "evidence_commands": [
            "python3.12 projects/mamba/tests/harness/cpython/tools/concurrency_readiness.py --json",
            "python3.12 projects/mamba/tests/harness/cpython/tools/results_store.py collect --dimension concurrency --limit 5 --jobs 1",
            "python3.12 projects/mamba/tests/harness/cpython/tools/verify_cpython_oracle.py --ready-only --bucket behavior",
            "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/threading/thread_tests__test_various_ops_uc3da134.py",
        ],
    }


def print_human(report: dict[str, Any]) -> None:
    print(f"concurrency readiness: {report['status']}")
    counts = report["counts"]
    print(
        "  fixtures={fixtures} pass_candidate={pass_candidate} "
        "unmeasured={unmeasured} promotion_pending={promotion_pending} "
        "runtime_failure_debt={runtime_failure_debt} parse_errors={parse_errors} "
        "missing_semantic_classes={missing_semantic_classes}".format(**counts)
    )
    print(
        "  runtime: ok={runtime_ok} fail={runtime_fail} timeout={runtime_timeout} "
        "crash={runtime_crash}".format(**counts)
    )
    for scope, scope_counts in report["by_scope"].items():
        print(f"- {scope}: {scope_counts}")
    for blocker in report["blockers"][:5]:
        label = blocker.get("path") or blocker.get("semantic_class") or blocker.get("lib")
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
    return 0 if report["ready"] else EXIT_NOT_READY


if __name__ == "__main__":
    raise SystemExit(main())
