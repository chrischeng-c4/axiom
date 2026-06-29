#!/usr/bin/env python3.12
"""Runtime-gap ownership bridge for #715.

The bridge keeps legacy runtime epics visible while the replacement-readiness
taxonomy (#702) becomes the primary planning surface. It is intentionally local
and deterministic: GitHub is the live tracker, but this report can be validated
without network access by using the current sweep failure cache and an explicit
inventory of old open runtime epics.
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


TOOLS_DIR = Path(__file__).resolve().parent
CPYTHON_DIR = TOOLS_DIR.parents[2] / "cpython"
MAMBA_DIR = CPYTHON_DIR.parent.parent
FAILURE_CACHE = CPYTHON_DIR / ".cache" / "sweep" / "failures.txt"

CLASSIFICATIONS = (
    "closed_stale_candidate",
    "still_failing_runtime_work",
    "readiness_test_hardening",
    "blocked",
)


@dataclass(frozen=True)
class LegacyEpic:
    number: int
    title: str
    classification: str
    readiness_dimensions: tuple[str, ...]
    selectors: tuple[str, ...]
    reproduction: str
    no_hit_classification: str = "closed_stale_candidate"


LEGACY_EPICS: tuple[LegacyEpic, ...] = (
    LegacyEpic(8, "pass-all-tests cluster repair loop", "still_failing_runtime_work", ("promotion_debt", "safety_stability_security"), (), "python3.12 projects/mamba/tests/harness/cpython/tools/sweep.py --help"),
    LegacyEpic(15, "non-3p stdlib/PEP long-tail sweep", "still_failing_runtime_work", ("platform_os_process_network_tls", "import_package_module_system"), ("_regression/core/", "_regression/builtin-libs/", "_regression/std-libs/"), "python3.12 projects/mamba/tests/harness/cpython/tools/sweep.py _regression/core _regression/std-libs --jobs 6"),
    LegacyEpic(16, "deep runtime architecture cross-cutting root causes", "still_failing_runtime_work", ("safety_stability_security", "concurrency_free_threaded"), (), "cargo test -p mamba --lib runtime -- --nocapture"),
    LegacyEpic(17, "ssl MemoryBIO and SSLSocket TLS state machine", "still_failing_runtime_work", ("platform_os_process_network_tls",), ("ssl",), "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/ssl/"),
    LegacyEpic(18, "harness oracle-env and results cache maintenance", "readiness_test_hardening", ("cpython_denominator", "perf_rss_baselines"), ("oracle-env", "results_store", "sweep"), "python3.12 projects/mamba/tests/harness/cpython/tools/results_store.py stats"),
    LegacyEpic(19, "third-libs mamba-side gaps", "blocked", ("third_party_c_extension_strategy",), ("_regression/3rd-libs/", "3rd-libs"), "python3.12 projects/mamba/tests/harness/cpython/tools/third_party_readiness.py --json"),
    LegacyEpic(22, "PEP 484 typing semantics", "still_failing_runtime_work", ("strict_type_accounting",), ("typing", "type-strict"), "python3.12 projects/mamba/tests/harness/cpython/tools/strict_type_accounting.py --json --limit 20"),
    LegacyEpic(23, "socket module", "still_failing_runtime_work", ("platform_os_process_network_tls",), ("socket",), "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/socket/"),
    LegacyEpic(24, "http family cookiejar server client", "still_failing_runtime_work", ("platform_os_process_network_tls",), ("http_", "http_cookiejar", "http_cookies", "urllib"), "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/http_client/"),
    LegacyEpic(26, "medium stdlib batch", "still_failing_runtime_work", ("platform_os_process_network_tls", "debugger_introspection_profiling", "concurrency_free_threaded"), ("platform", "weakref", "types", "ipaddress", "sqlite3", "lzma", "bdb", "tempfile", "subprocess", "enum"), "python3.12 projects/mamba/tests/harness/cpython/tools/platform_readiness.py --json"),
    LegacyEpic(27, "non-3p long-tail directories under ten failures", "still_failing_runtime_work", ("promotion_debt",), ("_regression/core/", "_regression/std-libs/", "_regression/builtin-libs/"), "python3.12 projects/mamba/tests/harness/cpython/tools/promotion_gate.py --profile replacement --json"),
    LegacyEpic(30, "runtime dynamic class construction", "still_failing_runtime_work", ("promotion_debt",), ("type", "dataclasses", "namedtuple", "enum"), "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/enum/"),
    LegacyEpic(
        31,
        "collections.abc mixin synthesis",
        "still_failing_runtime_work",
        ("promotion_debt",),
        ("collections", "collections_abc"),
        "env MAMBA_BIN=target/debug/mamba python3.12 projects/mamba/tests/harness/cpython/tools/sweep.py behavior/std-libs/collections/abc_mapping_views_are_set_like.py behavior/std-libs/collections/abc_mutablesequence_mixins.py behavior/std-libs/collections/abc_mutableset_mixins_mutate.py behavior/std-libs/collections/abc_set_mixins_provide_algebra.py behavior/std-libs/collections/abc_subclass_and_register.py --timeout 10 --jobs 6",
        no_hit_classification="still_failing_runtime_work",
    ),
    LegacyEpic(34, "language core miscellaneous batch", "still_failing_runtime_work", ("promotion_debt",), ("builtin", "ClassDef", "NameError", "class_system"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/core/class_system/errors.py"),
    LegacyEpic(36, "oracle-env ensure-script interpreter pin", "readiness_test_hardening", ("cpython_denominator",), ("oracle-env", "verify_cpython_oracle"), "python3.12 projects/mamba/tests/harness/cpython/tools/verify_cpython_oracle.py --ready-only --help"),
    LegacyEpic(37, "D5.3 oracle cache and D5.4 collector", "readiness_test_hardening", ("perf_rss_baselines", "safety_stability_security"), ("results_store", "sweep"), "python3.12 projects/mamba/tests/harness/cpython/tools/results_store.py summary --json"),
    LegacyEpic(89, "kwargs binding runtime", "still_failing_runtime_work", ("promotion_debt",), ("kwargs", "function_machinery", "call"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/core/function_machinery/"),
    LegacyEpic(91, "scope closure epic", "still_failing_runtime_work", ("promotion_debt", "import_package_module_system"), ("closure", "scope", "language/closures"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/core/language/closures/surface.py"),
    LegacyEpic(92, "str bytes surrogate representation", "still_failing_runtime_work", ("promotion_debt",), ("surrogate", "string", "bytes", "codecs"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/builtin-libs/bytes/construction.py"),
    LegacyEpic(93, "exec eval execution engine", "still_failing_runtime_work", ("promotion_debt",), ("eval_exec_compile", "exec", "eval"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/builtin-libs/builtins/eval_exec_compile.py"),
    LegacyEpic(95, "module-level IntEnum constants", "still_failing_runtime_work", ("platform_os_process_network_tls",), ("HTTPStatus", "Signals", "socket", "signal"), "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/signal/"),
    LegacyEpic(219, "builtin core contracts", "still_failing_runtime_work", ("promotion_debt",), ("_regression/builtin-libs/builtins", "builtins"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/builtin-libs/builtins/eval_exec_compile.py"),
    LegacyEpic(220, "list and sort runtime conformance", "still_failing_runtime_work", ("promotion_debt", "concurrency_free_threaded"), ("list", "sort"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/builtin-libs/list_methods/behavior.py"),
    LegacyEpic(221, "set and frozenset runtime conformance", "still_failing_runtime_work", ("promotion_debt", "concurrency_free_threaded"), ("set", "frozenset"), "target/debug/mamba run projects/mamba/tests/cpython/concurrency/atomicity/set/add_distinct_no_corruption.py"),
    LegacyEpic(222, "range and slice runtime conformance", "still_failing_runtime_work", ("promotion_debt",), ("range", "slice"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/builtin-libs/range/behavior.py"),
    LegacyEpic(223, "string Unicode and formatting", "still_failing_runtime_work", ("promotion_debt",), ("string", "unicode", "format", "codecs"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/builtin-libs/string_methods/errors.py"),
    LegacyEpic(224, "async and coroutine runtime conformance", "still_failing_runtime_work", ("concurrency_free_threaded",), ("async", "asyncio", "coroutine", "async_await"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/core/async_await/async_await_task.py"),
    LegacyEpic(225, "args kwargs and unpacking conformance", "still_failing_runtime_work", ("promotion_debt",), ("args", "kwargs", "unpack", "function_machinery"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/core/function_machinery/"),
    LegacyEpic(226, "class descriptor MRO and super", "still_failing_runtime_work", ("promotion_debt",), ("class_system", "descriptor", "mro", "super"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/core/class_system/errors.py"),
    LegacyEpic(227, "exception and traceback conformance", "still_failing_runtime_work", ("debugger_introspection_profiling", "safety_stability_security"), ("exception", "traceback"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/core/exceptions/generator_exceptions.py"),
    LegacyEpic(228, "decorator metaclass and slots", "still_failing_runtime_work", ("promotion_debt",), ("decorator", "metaclass", "slots", "class_system"), "target/debug/mamba run projects/mamba/tests/cpython/_regression/core/class_system/"),
    LegacyEpic(229, "closure scope and import", "still_failing_runtime_work", ("import_package_module_system",), ("closure", "scope", "import", "circular_import"), "python3.12 projects/mamba/tests/harness/cpython/tools/import_readiness.py --json"),
    LegacyEpic(231, "PEP 484 572 634 695", "still_failing_runtime_work", ("strict_type_accounting", "promotion_debt"), ("typing", "walrus", "pattern", "pep695"), "cargo test -p mamba --lib pep695 -- --nocapture"),
    LegacyEpic(232, "asyncio contextvars copy", "still_failing_runtime_work", ("concurrency_free_threaded",), ("asyncio", "contextvars", "copy"), "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/contextvars/"),
    LegacyEpic(233, "weakref weakset uuid GC", "still_failing_runtime_work", ("debugger_introspection_profiling",), ("weakref", "weakset", "uuid", "gc"), "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/weakref/"),
    LegacyEpic(234, "process threading queue subprocess", "still_failing_runtime_work", ("concurrency_free_threaded", "platform_os_process_network_tls"), ("threading", "queue", "subprocess", "multiprocessing"), "python3.12 projects/mamba/tests/harness/cpython/tools/concurrency_readiness.py --json"),
    LegacyEpic(235, "advanced re engine contracts", "still_failing_runtime_work", ("promotion_debt",), ("re/", "regex"), "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/re/"),
    LegacyEpic(236, "enum conversion and scaffolding", "still_failing_runtime_work", ("promotion_debt",), ("enum",), "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/enum/"),
    LegacyEpic(237, "functools dispatch and API surface", "still_failing_runtime_work", ("promotion_debt",), ("functools",), "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/functools/"),
    LegacyEpic(238, "codec callback and surrogate handlers", "still_failing_runtime_work", ("promotion_debt",), ("codecs", "surrogate", "codeccallbacks"), "target/debug/mamba run projects/mamba/tests/cpython/behavior/std-libs/codecs/"),
    LegacyEpic(239, "selector signal socket ssl live OS", "still_failing_runtime_work", ("platform_os_process_network_tls", "concurrency_free_threaded"), ("selectors", "signal", "socket", "ssl"), "python3.12 projects/mamba/tests/harness/cpython/tools/platform_readiness.py --json"),
    LegacyEpic(240, "remaining stdlib real-world and security long-tail", "still_failing_runtime_work", ("safety_stability_security", "platform_os_process_network_tls"), ("real_world", "security"), "python3.12 projects/mamba/tests/harness/cpython/tools/gate_check.py --json"),
    LegacyEpic(241, "inspect frame traceback tracemalloc", "still_failing_runtime_work", ("debugger_introspection_profiling",), ("inspect", "frame", "traceback", "tracemalloc"), "python3.12 projects/mamba/tests/harness/cpython/tools/debugger_readiness.py --json"),
    LegacyEpic(458, "native h2c HTTP client", "still_failing_runtime_work", ("third_party_c_extension_strategy", "platform_os_process_network_tls"), ("http", "h2", "httpx"), "cargo test -p mamba --test schema_gates third_party_readiness_gate_711 -- --nocapture"),
)


def repo_rel(path: Path) -> str:
    return path.resolve().relative_to(MAMBA_DIR.resolve()).as_posix()


def load_failure_cache(path: Path) -> list[str]:
    if not path.exists():
        return []
    return [
        line.strip()
        for line in path.read_text(encoding="utf-8", errors="replace").splitlines()
        if line.strip()
    ]


def selector_hits(entries: list[str], selectors: tuple[str, ...]) -> list[str]:
    if not selectors:
        return entries
    lowered = [(entry, entry.lower()) for entry in entries]
    needles = tuple(selector.lower() for selector in selectors)
    return [entry for entry, low in lowered if any(needle in low for needle in needles)]


def classify(epic: LegacyEpic, hits: list[str]) -> str:
    if epic.classification in {"blocked", "readiness_test_hardening"}:
        return epic.classification
    if hits:
        return "still_failing_runtime_work"
    return epic.no_hit_classification


def build_report(show: int, failure_cache: Path) -> dict[str, Any]:
    failures = load_failure_cache(failure_cache)
    epics: list[dict[str, Any]] = []
    classification_counts: dict[str, int] = {name: 0 for name in CLASSIFICATIONS}
    readiness_counts: dict[str, int] = {}
    unclassified: list[int] = []

    for epic in LEGACY_EPICS:
        hits = selector_hits(failures, epic.selectors)
        classification = classify(epic, hits)
        if classification not in CLASSIFICATIONS:
            unclassified.append(epic.number)
        classification_counts[classification] += 1
        for dimension in epic.readiness_dimensions:
            readiness_counts[dimension] = readiness_counts.get(dimension, 0) + 1
        epics.append(
            {
                "number": epic.number,
                "title": epic.title,
                "classification": classification,
                "readiness_dimensions": epic.readiness_dimensions,
                "selectors": epic.selectors,
                "failure_cache_matches": len(hits),
                "sample_failures": hits[:show],
                "reproduction": epic.reproduction,
            }
        )

    stale_candidates = [
        epic for epic in epics if epic["classification"] == "closed_stale_candidate"
    ]
    ready = not unclassified and not stale_candidates
    return {
        "schema_version": 1,
        "owner_issue": "#715",
        "ready": ready,
        "status": "green" if ready else "red",
        "failure_cache": repo_rel(failure_cache),
        "counts": {
            "legacy_epics": len(LEGACY_EPICS),
            "failure_cache_rows": len(failures),
            "unclassified": len(unclassified),
            "closed_stale_candidates": len(stale_candidates),
            **classification_counts,
        },
        "classification_vocabulary": CLASSIFICATIONS,
        "readiness_taxonomy": dict(sorted(readiness_counts.items())),
        "epics": epics,
        "blockers": [
            {
                "kind": "closed_stale_candidate_needs_current_evidence",
                "number": epic["number"],
                "title": epic["title"],
                "reason": "no matching failure-cache row; close only after focused debug-build proof",
            }
            for epic in stale_candidates[:show]
        ],
        "evidence_commands": [
            "gh issue list --label project:mamba --state open --limit 200",
            "python3.12 projects/mamba/tests/harness/cpython/tools/runtime_gap_bridge.py --json",
            "python3.12 projects/mamba/tests/harness/cpython/tools/replacement_readiness.py --json --show 1 --type-limit 1",
            "cargo build -p mamba",
        ],
    }


def print_human(report: dict[str, Any]) -> None:
    counts = report["counts"]
    print(f"runtime gap bridge: {report['status']}")
    print(
        "  epics={legacy_epics} failures={failure_cache_rows} "
        "still_failing={still_failing_runtime_work} readiness={readiness_test_hardening} "
        "blocked={blocked} stale_candidates={closed_stale_candidates}".format(**counts)
    )
    for epic in report["epics"]:
        print(
            f"- #{epic['number']} {epic['classification']} "
            f"matches={epic['failure_cache_matches']} dimensions={','.join(epic['readiness_dimensions'])}"
        )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--show", type=int, default=5)
    parser.add_argument("--failure-cache", type=Path, default=FAILURE_CACHE)
    args = parser.parse_args(argv)

    report = build_report(args.show, args.failure_cache)
    if args.json:
        print(json.dumps(report, sort_keys=True))
    else:
        print_human(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
