#!/usr/bin/env python3.12
"""Single CPython replacement-readiness report.

This command is intentionally stricter than development conformance. It reports
the current replacement blockers across readiness dimensions and exits nonzero
until every dimension is proven green. Red/blocked output is valid evidence.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]
PROMOTION_GATE = TOOLS_DIR / "promotion_gate.py"
DENOMINATOR_INVENTORY = MAMBA_DIR / "tools" / "cpython_regrtest_inventory.py"
STRICT_TYPE_ACCOUNTING = TOOLS_DIR / "strict_type_accounting.py"

EXIT_NOT_READY = 70


@dataclass
class Dimension:
    id: str
    title: str
    status: str
    owner_issue: str
    summary: str
    counts: dict[str, Any] = field(default_factory=dict)
    evidence: list[str] = field(default_factory=list)
    blockers: list[dict[str, Any]] = field(default_factory=list)

    def to_json(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "title": self.title,
            "status": self.status,
            "owner_issue": self.owner_issue,
            "summary": self.summary,
            "counts": self.counts,
            "evidence": self.evidence,
            "blockers": self.blockers,
        }


def run_json(argv: list[str], *, accepted: set[int]) -> tuple[int, dict[str, Any]]:
    proc = subprocess.run(argv, text=True, capture_output=True)
    if proc.returncode not in accepted:
        raise RuntimeError(
            f"{' '.join(argv)} exited {proc.returncode}\n"
            f"stdout={proc.stdout}\nstderr={proc.stderr}"
        )
    try:
        return proc.returncode, json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(
            f"{' '.join(argv)} did not emit JSON: {exc}\nstdout={proc.stdout}"
        ) from exc


def promotion_dimension(show: int) -> Dimension:
    code, payload = run_json(
        [
            sys.executable,
            str(PROMOTION_GATE),
            "--profile",
            "replacement",
            "--json",
            "--show",
            str(show),
        ],
        accepted={0, EXIT_NOT_READY},
    )
    total = int(payload["promotion_debt_total"])
    status = "green" if code == 0 and total == 0 else "red"
    blockers = []
    for key in ("unowned_debt", "promotion_pending_debt", "xfail_debt", "skip_debt"):
        for item in payload.get(key, [])[:show]:
            blockers.append(
                {
                    "kind": item["kind"],
                    "path": item["path"],
                    "owner_refs": item["owner_refs"],
                    "reason": item["reason"],
                    "source": key,
                }
            )
    return Dimension(
        id="promotion_debt",
        title="No xfail/skip/optional promotion debt",
        status=status,
        owner_issue="#703",
        summary=(
            "replacement profile has no xfail/skip/optional debt"
            if status == "green"
            else f"replacement profile has {total} promotion blockers"
        ),
        counts={
            "scanned_files": payload["scanned_files"],
            "xfail": payload["xfail_count"],
            "skip": payload["skip_count"],
            "optional": payload["optional_count"],
            "total": total,
            "owned": payload["owned_count"],
            "unowned": payload["unowned_count"],
            "promotion_pending": payload["promotion_pending_count"],
            "parse_errors": payload["parse_error_count"],
        },
        evidence=[
            "python3.12 projects/mamba/tests/harness/cpython/tools/promotion_gate.py --profile replacement --json"
        ],
        blockers=blockers,
    )


def denominator_dimension(show: int) -> Dimension:
    _, payload = run_json(
        [
            sys.executable,
            str(DENOMINATOR_INVENTORY),
            "--json",
            "--top",
            str(show),
        ],
        accepted={0},
    )
    ownership = payload["denominator_ownership"]
    status = "green" if ownership["pass"] else "uncovered"
    blockers = [
        {
            "kind": "unowned_denominator_module",
            "module": item["module"],
            "key": item["key"],
            "static_test_defs": item["static_test_defs"],
        }
        for item in payload["top_no_fixture_lib_or_source_match"][:show]
    ]
    return Dimension(
        id="cpython_denominator",
        title="CPython 3.12 Lib/test denominator ownership",
        status=status,
        owner_issue="#706",
        summary=(
            "all CPython regrtest modules have fixture/source ownership"
            if status == "green"
            else f"{ownership['unowned_modules']} CPython regrtest modules lack fixture/source ownership"
        ),
        counts={
            "python": payload["python"],
            "test_py_files": payload["cpython_test_case_candidates"]["test_py_files"],
            "regrtest_modules": payload["cpython_regrtest_modules"],
            "static_test_defs_in_regrtest_modules": payload[
                "static_test_defs_in_regrtest_modules"
            ],
            "owned_modules": ownership["owned_modules"],
            "unowned_modules": ownership["unowned_modules"],
            "owned_static_test_defs": ownership["owned_static_test_defs"],
            "unowned_static_test_defs": ownership["unowned_static_test_defs"],
            "invalid_metadata_files": ownership["invalid_metadata_files"],
            "parse_errors": ownership["parse_errors"],
        },
        evidence=["python3.12 projects/mamba/tools/cpython_regrtest_inventory.py --json"],
        blockers=blockers,
    )


def strict_type_dimension(show: int, type_limit: int) -> Dimension:
    argv = [
        sys.executable,
        str(STRICT_TYPE_ACCOUNTING),
        "--json",
        "--show",
        str(show),
    ]
    if type_limit:
        argv.extend(["--limit", str(type_limit)])
    code, payload = run_json(argv, accepted={0, EXIT_NOT_READY})
    status = "green" if code == 0 and payload["ready"] else "red"
    typeshed = payload["typeshed"]
    enforcement = payload["enforcement"]
    soundness = payload["soundness"]
    divergences = payload["divergences"]
    blockers = payload.get("blockers", [])[:show]
    return Dimension(
        id="strict_type_accounting",
        title="Strict-type divergence and typeshed accounting",
        status=status,
        owner_issue="#704",
        summary=(
            "strict-type denominator, enforcement, soundness, and divergence accounting are green"
            if status == "green"
            else (
                "strict-type accounting is not replacement-ready: "
                f"enforced={enforcement['enforced']}/{enforcement['gradable']} "
                f"gradable, soundness={soundness['passed']}/{soundness['gradable']} "
                f"gradable, divergences valid={divergences['valid']}/{divergences['declared']}"
            )
        ),
        counts={
            "typeshed_rows": typeshed["rows"],
            "typeshed_enforceable": typeshed["enforceable"],
            "typeshed_unknown_skipped": typeshed["unknown_skipped"],
            "type_fixture_wall": typeshed["type_fixture_wall"],
            "type_measured": enforcement["measured"],
            "type_gradable": enforcement["gradable"],
            "type_enforced": enforcement["enforced"],
            "type_leaked": enforcement["leaked"],
            "type_ungradable": enforcement["ungradable"],
            "soundness_measured": soundness["measured"],
            "soundness_gradable": soundness["gradable"],
            "soundness_passed": soundness["passed"],
            "soundness_failed": soundness["failed"],
            "soundness_oracle_skip": soundness["oracle_skip"],
            "declared_divergences": divergences["declared"],
            "valid_divergences": divergences["valid"],
            "invalid_divergences": divergences["invalid"],
            "missing_divergence_owner": divergences["missing_owner"],
            "sampled": payload["sampled"],
        },
        evidence=[
            "python3.12 projects/mamba/tests/harness/cpython/tools/strict_type_accounting.py --json",
            "python3.12 projects/mamba/tests/harness/cpython/tools/fixture_lint.py --bucket type",
        ],
        blockers=blockers,
    )


def blocked_dimension(
    *,
    id: str,
    title: str,
    owner_issue: str,
    summary: str,
    evidence: list[str],
) -> Dimension:
    return Dimension(
        id=id,
        title=title,
        status="blocked",
        owner_issue=owner_issue,
        summary=summary,
        evidence=evidence,
        blockers=[
            {
                "kind": "readiness_dimension_not_integrated",
                "owner_issue": owner_issue,
                "reason": summary,
            }
        ],
    )


def dimensions(show: int, type_limit: int) -> list[Dimension]:
    return [
        denominator_dimension(show),
        promotion_dimension(show),
        strict_type_dimension(show, type_limit),
        blocked_dimension(
            id="perf_rss_baselines",
            title="Performance and peak-RSS baseline gate",
            owner_issue="#707",
            summary="perf/RSS baseline completeness is still owned by cpython_status/perf pins and not proven green",
            evidence=["cargo test -p mamba --test cpython_status -- --json"],
        ),
        blocked_dimension(
            id="safety_stability_security",
            title="Safety, stability, leak, crash, and secret-leak gates",
            owner_issue="#709",
            summary="safety/stability/security readiness dimensions require dedicated leak/crash/secret-leak proof",
            evidence=["python3.12 projects/mamba/tests/harness/cpython/tools/gate_check.py --json"],
        ),
        blocked_dimension(
            id="platform_os_process_network_tls",
            title="Platform, OS, process, network, and TLS coverage",
            owner_issue="#710",
            summary="platform/OS/process/network/TLS coverage is not yet proven as a replacement gate",
            evidence=["cargo test -p mamba --test cpython_status -- --json"],
        ),
        blocked_dimension(
            id="import_package_module_system",
            title="Import, package, and module-system semantics",
            owner_issue="#708",
            summary="import/package/module-system semantics need dedicated denominator and runtime evidence",
            evidence=["python3.12 projects/mamba/tests/harness/cpython/tools/gate_status.py --help"],
        ),
        blocked_dimension(
            id="third_party_c_extension_strategy",
            title="Third-party and C-extension compatibility strategy",
            owner_issue="#711",
            summary="third-party and C-extension strategy is not yet green for replacement readiness",
            evidence=["cargo test -p mamba --test cpython_status -- --json"],
        ),
        blocked_dimension(
            id="debugger_introspection_profiling",
            title="Debugger, introspection, profiling, and tracing surfaces",
            owner_issue="#712",
            summary="debugger/introspection/profiling/tracing surfaces are not yet replacement-ready",
            evidence=["cargo test -p mamba --test cpython_status"],
        ),
        blocked_dimension(
            id="concurrency_free_threaded",
            title="Concurrency and free-threaded semantics",
            owner_issue="#713",
            summary="concurrency and free-threaded semantics still need strict replacement gates",
            evidence=["cargo test -p mamba --test cpython_status"],
        ),
    ]


def build_report(show: int, type_limit: int = 0) -> dict[str, Any]:
    dims = dimensions(show, type_limit)
    counts = {
        "green": sum(1 for item in dims if item.status == "green"),
        "red": sum(1 for item in dims if item.status == "red"),
        "blocked": sum(1 for item in dims if item.status == "blocked"),
        "uncovered": sum(1 for item in dims if item.status == "uncovered"),
    }
    ready = counts["red"] == 0 and counts["blocked"] == 0 and counts["uncovered"] == 0
    return {
        "schema_version": 1,
        "profile": "replacement-readiness",
        "development_build_policy": "debug",
        "ready": ready,
        "status": "green" if ready else "red",
        "counts": counts,
        "dimensions": [item.to_json() for item in dims],
    }


def print_human(report: dict[str, Any]) -> None:
    print(f"replacement readiness: {report['status']}")
    print(f"  profile: {report['profile']}")
    print(f"  development build policy: {report['development_build_policy']}")
    counts = report["counts"]
    print(
        "  dimensions: "
        f"green={counts['green']} red={counts['red']} "
        f"blocked={counts['blocked']} uncovered={counts['uncovered']}"
    )
    for item in report["dimensions"]:
        print(f"- {item['status']}: {item['id']} ({item['owner_issue']})")
        print(f"  {item['summary']}")
        if item["counts"]:
            counts_text = ", ".join(
                f"{key}={value}" for key, value in sorted(item["counts"].items())
            )
            print(f"  counts: {counts_text}")
        for blocker in item["blockers"][:3]:
            label = blocker.get("path") or blocker.get("module") or blocker.get("kind")
            reason = blocker.get("reason") or blocker.get("kind")
            print(f"  blocker: {label} - {reason}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--show", type=int, default=10)
    parser.add_argument(
        "--type-limit",
        type=int,
        default=0,
        help="sample N strict-type fixtures for development validation; sampled reports stay red",
    )
    args = parser.parse_args(argv)

    report = build_report(args.show, args.type_limit)
    if args.json:
        print(json.dumps(report, sort_keys=True))
    else:
        print_human(report)
    return 0 if report["ready"] else EXIT_NOT_READY


if __name__ == "__main__":
    raise SystemExit(main())
